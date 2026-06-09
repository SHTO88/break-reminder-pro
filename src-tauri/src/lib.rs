use log::{error, info};
use serde::{Deserialize, Serialize};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs;
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

// For autostart plugin
use tauri_plugin_autostart::ManagerExt;

// Import our window manager module
mod window_manager;
use window_manager::{WindowConfig, WindowManager};

/// Shared flag: was media playing when the break started?
/// Written by main window before break, read by break windows on close.
/// Using AtomicBool so it's safe to access from any thread/webview.
static MEDIA_WAS_PLAYING: AtomicBool = AtomicBool::new(false);

/// Was VLC playing when we sent the pause command? Set by Core Audio session check.
static VLC_WAS_PLAYING: AtomicBool = AtomicBool::new(false);

/// SMTC sources that were paused by us at break start.
static SMTC_PAUSED_SOURCES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn smtc_paused_sources() -> &'static Mutex<Vec<String>> {
    SMTC_PAUSED_SOURCES.get_or_init(|| Mutex::new(Vec::new()))
}

#[tauri::command]
fn set_media_was_playing(was_playing: bool) {
    MEDIA_WAS_PLAYING.store(was_playing, Ordering::SeqCst);
}

#[tauri::command]
fn get_media_was_playing() -> bool {
    MEDIA_WAS_PLAYING.load(Ordering::SeqCst)
}

#[tauri::command]
fn clear_media_was_playing() {
    MEDIA_WAS_PLAYING.store(false, Ordering::SeqCst);
    VLC_WAS_PLAYING.store(false, Ordering::SeqCst);
    if let Ok(mut guard) = smtc_paused_sources().lock() {
        guard.clear();
    }
}

/// Initialise file + terminal logging.
///
/// Log files go to: <AppData>\Break Reminder Pro\
///   app.log      — current session
///   app.log.bak  — previous session (one rotation kept for crash diagnosis)
///
/// On every startup the current log is rotated to .bak so each session starts
/// fresh. This keeps the total log footprint to at most two small files.
fn init_logging() {
    // Determine log directory and file paths
    let (log_path, bak_path) = {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());
        let dir = std::path::PathBuf::from(appdata).join("Break Reminder Pro");
        let _ = fs::create_dir_all(&dir);
        (dir.join("app.log"), dir.join("app.log.bak"))
    };

    // Rotate: move current log → .bak (overwrites previous .bak)
    // This bounds total log storage to two session files.
    if log_path.exists() {
        let _ = fs::rename(&log_path, &bak_path);
    }

    // Open (or create) the log file — always starts empty after rotation
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .expect("Failed to open log file");

    // Terminal: Debug in dev builds, Info in release
    // File: Info in release (avoids flooding the log with debug noise),
    //       Debug in dev (full visibility during development)
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    CombinedLogger::init(vec![
        TermLogger::new(log_level, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(log_level, Config::default(), log_file),
    ])
    .unwrap_or_else(|_| {
        // Already initialised (e.g. in tests) — silently continue
    });

    info!(
        "=== Break Reminder Pro v{} starting ===",
        env!("CARGO_PKG_VERSION")
    );
    info!("Log file: {}", log_path.display());
}


#[derive(Serialize, Deserialize)]
struct AppSettings {
    break_minutes: u32,
    break_seconds: u32,
    break_duration_minutes: u32,
    break_duration_seconds: u32,
    break_mode: String,
    auto_pause: bool,
    meeting_detect: bool,
    pre_break: bool,
    pre_break_minutes: u32,
    pre_break_seconds: u32,
    break_chime: bool,
    recurring: bool,
    autostart: bool,
    auto_start_timer: bool,
}

#[tauri::command]
fn force_break_window(app_handle: tauri::AppHandle, duration: Option<u32>) -> Result<(), String> {
    let break_duration = duration.unwrap_or(300);
    println!(
        "💥 Creating force break window with duration: {} seconds",
        break_duration
    );

    WindowManager::close_existing_window(&app_handle, "force_break");
    let config = WindowConfig::force_break(break_duration);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
fn close_window(app_handle: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&label) {
        window
            .close()
            .map_err(|e| format!("Failed to close window {}: {}", label, e))?;
    }
    Ok(())
}

#[tauri::command]
fn notify_window(app_handle: tauri::AppHandle, duration: Option<u32>) -> Result<(), String> {
    let break_duration = duration.unwrap_or(600);
    println!(
        "🔔 Creating notify window with duration: {} seconds",
        break_duration
    );

    WindowManager::close_existing_window(&app_handle, "notify");
    let config = WindowConfig::notify(&app_handle, break_duration);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
fn pre_break_notification_window(
    app_handle: tauri::AppHandle,
    remaining_seconds: Option<u32>,
) -> Result<(), String> {
    let seconds = remaining_seconds.unwrap_or(30);
    println!(
        "⏰ Creating pre-break window with {} seconds remaining...",
        seconds
    );

    WindowManager::close_existing_window(&app_handle, "pre_break");
    let config = WindowConfig::pre_break(&app_handle, seconds);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
async fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager
        .enable()
        .map_err(|e| format!("Failed to enable autostart: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager
        .disable()
        .map_err(|e| format!("Failed to disable autostart: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager
        .is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
}

#[tauri::command]
async fn hide_to_tray(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
        println!("🫥 Main window hidden to system tray");
    }
    Ok(())
}

#[tauri::command]
async fn show_from_tray(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus window: {}", e))?;
        println!("👁️ Main window restored from system tray");
    }
    Ok(())
}

#[tauri::command]
async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🚪 Quitting application completely");
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn open_url(url: String) -> Result<(), String> {
    println!("🌐 Opening URL: {}", url);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
fn show_update_notification(
    app_handle: tauri::AppHandle,
    version: String,
    notes: String,
    download_url: String,
    published_at: String,
) -> Result<(), String> {
    println!("🔔 Showing update notification for version: {}", version);

    WindowManager::close_existing_window(&app_handle, "update_notification");
    let config =
        WindowConfig::update_notification(&app_handle, version, notes, download_url, published_at);
    WindowManager::create_window(app_handle, config)
}


#[tauri::command]
fn lock_screen() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("rundll32.exe")
            .arg("user32.dll,LockWorkStation")
            .spawn()
            .map_err(|e| format!("Failed to lock screen: {}", e))?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Lock screen only supported on Windows".to_string())
    }
}

/// Check if VLC is currently playing by inspecting its Windows Core Audio session state.
/// A playing VLC has an Active audio session; a paused VLC has an Inactive one.
/// Returns false if VLC is not running or has no audio session.
#[cfg(target_os = "windows")]
fn is_vlc_playing_via_audio(vlc_pid: u32) -> bool {
    use windows::Win32::Media::Audio::{
        eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
        IAudioSessionControl2, IAudioSessionManager2,
        AudioSessionStateActive,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};
    use windows::core::ComInterface;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

        let enumerator: IMMDeviceEnumerator =
            match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(e) => { println!("⚠️ CoCreateInstance failed: {:?}", e); return false; }
            };

        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
            Ok(d) => d,
            Err(e) => { println!("⚠️ GetDefaultAudioEndpoint failed: {:?}", e); return false; }
        };

        // IMMDevice::Activate is generic in windows 0.52 — type inferred from return type
        let session_manager: IAudioSessionManager2 =
            match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(e) => { println!("⚠️ Activate IAudioSessionManager2 failed: {:?}", e); return false; }
            };

        let session_enum = match session_manager.GetSessionEnumerator() {
            Ok(e) => e,
            Err(e) => { println!("⚠️ GetSessionEnumerator failed: {:?}", e); return false; }
        };

        let count = match session_enum.GetCount() {
            Ok(c) => c,
            Err(_) => return false,
        };

        for i in 0..count {
            let session = match session_enum.GetSession(i) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let session2: IAudioSessionControl2 = match session.cast() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let pid = match session2.GetProcessId() {
                Ok(p) => p,
                Err(_) => continue,
            };
            if pid == vlc_pid {
                let state = match session2.GetState() {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let is_active = state == AudioSessionStateActive;
                println!("  VLC audio session state: {:?} → playing={}", state, is_active);
                return is_active;
            }
        }
        println!("  No audio session found for VLC PID {} (no audio or muted)", vlc_pid);
        false
    }
}

/// Detects the lock screen by checking if LogonUI.exe is running — Windows always
/// launches this process when the workstation is locked, regardless of desktop access.
#[tauri::command]
fn is_screen_locked() -> bool {
    #[cfg(target_os = "windows")]
    {
        use sysinfo::System;
        let sys = System::new_all();
        let locked = sys.processes().values().any(|proc| {
            proc.name().to_lowercase() == "logonui.exe"
        });
        if locked {
            println!("🔒 Screen is locked (LogonUI.exe detected)");
        } else {
            println!("🔓 Screen is not locked");
        }
        locked
    }
    #[cfg(not(target_os = "windows"))]
    false
}

#[tauri::command]
async fn control_media(action: String) -> Result<(), String> {
    println!("🎵 Media control requested: {}", action);

    #[cfg(target_os = "windows")]
    {
        // ── VLC: state-aware WM_APPCOMMAND ──────────────────────────────────────
        // VLC never registers with SMTC. We check its actual playing state first
        // so we only toggle it when needed (pause only if playing, play only if paused).
        {
            use winapi::um::winuser::{
                EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
                SendMessageTimeoutW,
                WM_APPCOMMAND, SMTO_ABORTIFHUNG,
            };
            use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
            use winapi::shared::windef::HWND;
            use sysinfo::System;

            let sys = System::new_all();
            let vlc_pid: Option<u32> = sys.processes().values()
                .find(|p| p.name().to_lowercase() == "vlc.exe")
                .map(|p| p.pid().as_u32());

            if let Some(pid) = vlc_pid {
                struct SearchData { pid: u32, hwnd: HWND }
                let mut data = SearchData { pid, hwnd: std::ptr::null_mut() };

                unsafe extern "system" fn find_vlc_visible_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
                    let data = &mut *(lparam as *mut SearchData);
                    let mut wpid: u32 = 0;
                    GetWindowThreadProcessId(hwnd, &mut wpid);
                    if wpid == data.pid && IsWindowVisible(hwnd) != 0 {
                        data.hwnd = hwnd;
                        return 0;
                    }
                    TRUE
                }
                unsafe { EnumWindows(Some(find_vlc_visible_window), &mut data as *mut SearchData as LPARAM); }

                if !data.hwnd.is_null() {
                    // Use explicit PAUSE/PLAY commands — but only act if VLC is in
                    // the expected state, detected via its Core Audio session.
                    // Active audio session = playing; Inactive = paused.
                    use winapi::um::winuser::{
                        APPCOMMAND_MEDIA_PAUSE, APPCOMMAND_MEDIA_PLAY,
                        APPCOMMAND_MEDIA_PLAY_PAUSE,
                    };

                    let should_send = match action.as_str() {
                        "pause" => {
                            let playing = is_vlc_playing_via_audio(pid);
                            VLC_WAS_PLAYING.store(playing, Ordering::SeqCst);
                            println!("  VLC audio state → playing={}, will_pause={}", playing, playing);
                            playing
                        }
                        "play" => {
                            let was = VLC_WAS_PLAYING.load(Ordering::SeqCst);
                            println!("  VLC was_playing={}, will_resume={}", was, was);
                            was
                        }
                        _ => true,
                    };

                    if should_send {
                        let appcommand: i16 = match action.as_str() {
                            "pause" => APPCOMMAND_MEDIA_PAUSE,
                            "play"  => APPCOMMAND_MEDIA_PLAY,
                            _       => APPCOMMAND_MEDIA_PLAY_PAUSE,
                        };
                        let lparam_val = ((appcommand as isize) << 16) as isize;
                        let mut result: usize = 0;
                        let ret = unsafe {
                            SendMessageTimeoutW(
                                data.hwnd, WM_APPCOMMAND, data.hwnd as usize,
                                lparam_val, SMTO_ABORTIFHUNG, 1000, &mut result,
                            )
                        };
                        println!("✅ WM_APPCOMMAND {} sent to VLC (ret={} result={})", action, ret, result);
                    } else {
                        println!("⏭️ Skipping VLC WM_APPCOMMAND for action '{}' (wrong state)", action);
                    }
                } else {
                    println!("⚠️ VLC running but no visible window found");
                }
            } else {
                println!("ℹ️ VLC not running");
            }
        }

        // ── SMTC: track which sessions we pause, only resume those ──────────────
        // On pause: record which sources were Playing → pause them → store their IDs.
        // On play:  only resume the sources we recorded, not all paused sessions.
        let smtc_action = action.clone();
        let smtc_thread = std::thread::spawn(move || {
            use windows::Media::Control::{
                GlobalSystemMediaTransportControlsSessionManager,
                GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            };

            let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|op| op.get())
            {
                Ok(m) => m,
                Err(e) => { println!("⚠️ SMTC manager failed: {:?}", e); return; }
            };

            let sessions_view = match manager.GetSessions() {
                Ok(s) => s,
                Err(e) => { println!("⚠️ GetSessions failed: {:?}", e); return; }
            };
            let count = sessions_view.Size().unwrap_or(0);
            println!("🎵 SMTC sessions: {}", count);

            match smtc_action.as_str() {
                "pause" => {
                    // Pause only currently-playing sessions and record their IDs
                    let mut paused_sources: Vec<String> = Vec::new();
                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) { Ok(s) => s, Err(_) => continue };
                        let source = session.SourceAppUserModelId()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|_| format!("session_{}", i));
                        let is_playing = session.GetPlaybackInfo()
                            .and_then(|info| info.PlaybackStatus())
                            .map(|st| st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                            .unwrap_or(false);

                        if is_playing {
                            println!("  ⏸ Pausing SMTC: {}", source);
                            let _ = session.TryPauseAsync().and_then(|op| op.get());
                            paused_sources.push(source);
                        } else {
                            println!("  ⏭️ Skipped (not playing): {}", source);
                        }
                    }
                    // Store the list of sources we paused
                    if let Ok(mut guard) = smtc_paused_sources().lock() {
                        *guard = paused_sources.clone();
                        println!("💾 Stored {} paused SMTC sources: {:?}", paused_sources.len(), paused_sources);
                    }
                }
                "play" => {
                    // Resume only the sessions we previously paused
                    let paused_sources = if let Ok(guard) = smtc_paused_sources().lock() {
                        guard.clone()
                    } else {
                        Vec::new()
                    };
                    println!("  Resuming {} recorded SMTC sources: {:?}", paused_sources.len(), paused_sources);

                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) { Ok(s) => s, Err(_) => continue };
                        let source = session.SourceAppUserModelId()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|_| format!("session_{}", i));

                        if paused_sources.contains(&source) {
                            println!("  ▶ Resuming SMTC: {}", source);
                            let _ = session.TryPlayAsync().and_then(|op| op.get());
                        } else {
                            println!("  ⏭️ Not our session, skipping: {}", source);
                        }
                    }
                }
                _ => {
                    // playpause toggle: act on all playing sessions
                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) { Ok(s) => s, Err(_) => continue };
                        let _ = session.TryTogglePlayPauseAsync().and_then(|op| op.get());
                    }
                }
            }
        });

        let _ = smtc_thread.join();
        return Ok(());
    }

    // Non-Windows fallback
    #[cfg(not(target_os = "windows"))]
    {
        use enigo::{Enigo, Key, KeyboardControllable};
        let mut enigo = Enigo::new();
        enigo.key_click(Key::MediaPlayPause);
        Ok(())
    }
}

/// Returns true if ANY SMTC session is currently Playing.
/// Falls back to false (assume not playing) if SMTC is unavailable.
#[tauri::command]
async fn is_media_playing() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Media::Control::{
            GlobalSystemMediaTransportControlsSessionManager,
            GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        };

        let result = async {
            let manager =
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
            let sessions = manager.GetSessions()?;
            let count = sessions.Size().unwrap_or(0);
            for i in 0..count {
                let session = match sessions.GetAt(i) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let is_playing = session
                    .GetPlaybackInfo()
                    .and_then(|info| info.PlaybackStatus())
                    .map(|st| {
                        st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
                    })
                    .unwrap_or(false);
                if is_playing {
                    return Ok::<bool, windows::core::Error>(true);
                }
            }
            Ok::<bool, windows::core::Error>(false)
        }
        .await;

        match result {
            Ok(playing) => {
                println!("🎵 Media playing state: {}", playing);
                return playing;
            }
            Err(e) => {
                println!("⚠️ Could not read SMTC playback state: {:?}", e);
                return false;
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    false
}

#[tauri::command]
fn play_chime() -> Result<(), String> {
    println!("🔔 Playing chime sound...");

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // Try to play the Windows default notification sound
        match Command::new("powershell")
            .arg("-Command")
            .arg("[System.Media.SystemSounds]::Beep.Play()")
            .output()
        {
            Ok(_) => {
                println!("✅ Chime played successfully using SystemSounds");
                Ok(())
            }
            Err(e) => {
                println!("⚠️ SystemSounds failed, trying alternative: {}", e);

                // Fallback: Use rundll32 to play default system sound
                match Command::new("rundll32")
                    .arg("user32.dll,MessageBeep")
                    .arg("0")
                    .output()
                {
                    Ok(_) => {
                        println!("✅ Chime played successfully using MessageBeep");
                        Ok(())
                    }
                    Err(e2) => {
                        println!("❌ Both chime methods failed");
                        Err(format!(
                            "Failed to play chime: SystemSounds error: {}, MessageBeep error: {}",
                            e, e2
                        ))
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Chime playback only supported on Windows".to_string())
    }
}

#[tauri::command]
fn is_meeting_active() -> Result<bool, String> {
    use sysinfo::System;

    // Check for desktop meeting applications
    let meeting_processes = [
        "zoom.exe",
        "teams.exe",
        "skype.exe",
        "webex.exe",
        "meet.exe",
    ];

    let sys = System::new_all();
    let desktop_meeting_found = sys.processes().values().any(|proc| {
        let name = proc.name().to_lowercase();
        meeting_processes.iter().any(|mp| name.contains(mp))
    });

    if desktop_meeting_found {
        return Ok(true);
    }

    // Check for browser-based meetings (Windows only)
    #[cfg(target_os = "windows")]
    {
        if let Ok(browser_meeting_found) = check_browser_meetings() {
            return Ok(browser_meeting_found);
        }
    }

    Ok(false)
}

#[cfg(target_os = "windows")]
fn check_browser_meetings() -> Result<bool, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::shared::minwindef::{BOOL, FALSE, LPARAM, TRUE};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowTextW, IsWindowVisible};

    // Meeting indicators to look for in browser window titles
    let meeting_indicators = [
        "google meet",
        "meet.google.com",
        "zoom meeting",
        "microsoft teams",
        "teams.microsoft.com",
        "webex meeting",
        "webex.com",
        "gotomeeting",
        "join.me",
        "bluejeans",
        "whereby.com",
        "discord",
        "slack call",
        "skype",
        "hangouts",
        "jitsi meet",
        "bigbluebutton",
        "8x8.vc",
        "ringcentral meetings",
        "cisco webex",
        "amazon chime",
        "facebook messenger rooms",
        "whatsapp web",
    ];

    // Browser process names to check
    let browser_processes = [
        "chrome.exe",
        "firefox.exe",
        "msedge.exe",
        "opera.exe",
        "brave.exe",
        "vivaldi.exe",
        "iexplore.exe",
    ];

    // First check if any browsers are running
    let sys = sysinfo::System::new_all();
    let browser_running = sys.processes().values().any(|proc| {
        let name = proc.name().to_lowercase();
        browser_processes.iter().any(|bp| name.contains(bp))
    });

    if !browser_running {
        return Ok(false);
    }

    // Structure to pass data to the callback
    struct CallbackData {
        meeting_indicators: Vec<String>,
        found_meeting: bool,
    }

    let mut callback_data = CallbackData {
        meeting_indicators: meeting_indicators
            .iter()
            .map(|s| s.to_lowercase())
            .collect(),
        found_meeting: false,
    };

    // Callback function for EnumWindows
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let callback_data = &mut *(lparam as *mut CallbackData);

        // Only check visible windows
        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }

        // Get window title
        let mut title: [u16; 512] = [0; 512];
        let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

        if title_len > 0 {
            let title_os_string = OsString::from_wide(&title[..title_len as usize]);
            if let Ok(title_string) = title_os_string.into_string() {
                let title_lower = title_string.to_lowercase();

                // Check if the window title contains any meeting indicators
                for indicator in &callback_data.meeting_indicators {
                    if title_lower.contains(indicator) {
                        println!("🔍 Meeting detected in browser window: {}", title_string);
                        callback_data.found_meeting = true;
                        return FALSE; // Stop enumeration
                    }
                }
            }
        }

        TRUE // Continue enumeration
    }

    // Enumerate all windows
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut callback_data as *mut CallbackData as LPARAM,
        );
    }

    Ok(callback_data.found_meeting)
}

#[tauri::command]
fn check_browser_meeting_debug() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        match check_browser_meetings() {
            Ok(found) => {
                if found {
                    Ok("Browser meeting detected".to_string())
                } else {
                    Ok("No browser meeting detected".to_string())
                }
            }
            Err(e) => Ok(format!("Error checking browser meetings: {}", e)),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Browser meeting detection only supported on Windows".to_string())
    }
}

#[tauri::command]
fn save_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create the directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let settings_path = app_data_dir.join("settings.json");
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(settings_path, settings_json)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

#[tauri::command]
fn load_settings(app_handle: tauri::AppHandle) -> Result<Option<AppSettings>, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let settings_path = app_data_dir.join("settings.json");

    if !settings_path.exists() {
        return Ok(None);
    }

    let settings_json = fs::read_to_string(settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let settings: AppSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(Some(settings))
}

#[tauri::command]
fn debug_test_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🧪 Creating debug test window...");

    // Close existing window if it exists
    if let Some(existing) = app_handle.get_webview_window("debug_test") {
        println!("📄 Closing existing debug test window");
        let _ = existing.close();
    }

    // Create window in separate thread as recommended by Tauri docs
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        println!("📂 Attempting to load: test.html");
        match WebviewWindowBuilder::new(&handle, "debug_test", WebviewUrl::App("test.html".into()))
            .title("Debug Test Window - Using test.html")
            .inner_size(500.0, 400.0)
            .position(200.0, 200.0)
            .resizable(true)
            .decorations(true)
            .always_on_top(true)
            .focused(true)
            .visible(true)
            .build()
        {
            Ok(window) => {
                println!("✅ Debug test window created successfully!");
                println!("🎯 Window label: {}", window.label());
                println!("📋 Expected content: Colorful gradient with TEST SUCCESS message");

                // Try to inject some debugging JavaScript after a delay
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window.eval("console.log('🔥 Debug test window JavaScript executed!'); document.title = 'TEST WINDOW LOADED';");
            }
            Err(e) => {
                println!("❌ Failed to create debug test window: {}", e);
            }
        }
    });

    println!("🚀 Debug test window creation initiated in separate thread");
    Ok(())
}

#[tauri::command]
async fn get_primary_monitor_size(app_handle: tauri::AppHandle) -> Result<(u32, u32), String> {
    match app_handle.primary_monitor() {
        Ok(Some(monitor)) => {
            let size = monitor.size();
            Ok((size.width, size.height))
        }
        Ok(None) => {
            // No monitor found, try to get screen size from system
            println!("No primary monitor found, attempting to get screen size from system");
            // Return a reasonable default that will be overridden by JavaScript
            Ok((800, 600))
        }
        Err(e) => {
            println!("Error getting monitor info: {}, using default size", e);
            // Return a reasonable default that will be overridden by JavaScript
            Ok((800, 600))
        }
    }
}

#[tauri::command]
fn meeting_detected_notification(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🤝 Creating meeting detected notification window...");

    WindowManager::close_existing_window(&app_handle, "meeting_notification");
    let config = WindowConfig::meeting_notification(&app_handle);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
fn break_ended_early(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🏃 Break ended early - user returned");

    // Try to notify the main window about early return
    if let Some(main_window) = app_handle.get_webview_window("main") {
        println!("📱 Found main window, calling handleEarlyBreakReturn");
        match main_window.eval("if (window.handleEarlyBreakReturn) { window.handleEarlyBreakReturn(); } else { console.error('handleEarlyBreakReturn function not found on window!'); }") {
            Ok(_) => println!("✅ Successfully called handleEarlyBreakReturn"),
            Err(e) => println!("❌ Error calling handleEarlyBreakReturn: {}", e),
        }
    } else {
        println!("❌ Main window not found!");
    }

    Ok(())
}

#[tauri::command]
fn skip_break(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("⏭️ Skip break requested");

    // Close pre-break window if it exists
    if let Some(window) = app_handle.get_webview_window("pre_break") {
        let _ = window.close();
    }

    // Close any active break windows
    if let Some(window) = app_handle.get_webview_window("force_break") {
        let _ = window.close();
    }
    if let Some(window) = app_handle.get_webview_window("notify") {
        let _ = window.close();
    }

    // Notify main window that break was skipped
    if let Some(main_window) = app_handle.get_webview_window("main") {
        println!("📱 Found main window, calling handleBreakSkipped");
        match main_window.eval("if (window.handleBreakSkipped) { window.handleBreakSkipped(); } else { console.error('handleBreakSkipped function not found on window!'); }") {
            Ok(_) => println!("✅ Successfully called handleBreakSkipped"),
            Err(e) => println!("❌ Error calling handleBreakSkipped: {}", e),
        }
    } else {
        println!("❌ Main window not found!");
    }

    println!("✅ Break skipped successfully");
    Ok(())
}

#[tauri::command]
fn show_index_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("Showing index window...");
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        WebviewWindowBuilder::new(&app_handle, "main", WebviewUrl::App("index.html".into()))
            .title("Break Reminder Pro")
            .build()
            .map_err(|e| format!("Failed to create main window: {}", e))?;
    }
    Ok(())
}

fn setup_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    info!("Tray: creating menu items...");
    let quit_item = MenuItem::with_id(app, "quit", "Quit Break Reminder Pro", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
    info!("Tray: assembling menu...");
    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

    info!("Tray: loading icon...");
    let icon = app
        .default_window_icon()
        .ok_or("No default window icon found")?
        .clone();

    info!("Tray: building tray icon...");
    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Break Reminder Pro - Click to toggle window")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                println!("🚪 Quit selected from tray menu");
                app.exit(0);
            }
            "show" => {
                println!("👁️ Show selected from tray menu");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                println!("🫥 Hide selected from tray menu");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                        println!("🫥 Main window hidden via tray click");
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        println!("👁️ Main window shown via tray click");
                    }
                }
            }
        })
        .build(app)?;

    info!("✅ System tray built successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialise logging before anything else so crashes are captured
    init_logging();

    // Install a panic hook that writes to the log file before the process dies
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "unknown panic payload",
            },
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        error!("💥 PANIC at {}: {}", location, msg);
    }));

    info!("Initialising Tauri application...");

    info!("Step 1: Building Tauri app...");
    tauri::Builder::default()
        .plugin({
            info!("Step 2: Loading opener plugin...");
            tauri_plugin_opener::init()
        })
        .plugin({
            info!("Step 3: Loading autostart plugin...");
            tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None)
        })
        .setup(|app| {
            info!("Step 4: Setup callback started");

            // Setup system tray
            if let Err(e) = setup_system_tray(app.handle()) {
                error!("❌ Failed to setup system tray: {}", e);
            } else {
                info!("✅ System tray initialized");
            }

            info!("Step 4a: Looking for main window...");
            // Handle window close events to hide to tray instead of closing
            if let Some(window) = app.get_webview_window("main") {
                info!("✅ Main window found, attaching close handler");
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                            info!("🫥 Main window hidden to tray instead of closing");
                        }
                    }
                });
            } else {
                error!("❌ Main window not found during setup!");
            }

            info!("Step 4: Setup callback complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lock_screen,
            is_screen_locked,
            control_media,
            is_media_playing,
            play_chime,
            is_meeting_active,
            check_browser_meeting_debug,
            force_break_window,
            close_window,
            notify_window,
            pre_break_notification_window,
            meeting_detected_notification,
            get_primary_monitor_size,
            break_ended_early,
            skip_break,
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,
            hide_to_tray,
            show_from_tray,
            quit_app,
            save_settings,
            load_settings,
            debug_test_window,
            show_index_window,
            get_app_version,
            open_url,
            show_update_notification,
            set_media_was_playing,
            get_media_was_playing,
            clear_media_was_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
