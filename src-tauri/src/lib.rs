use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

// For autostart and opener plugins
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

// Import our window manager module
mod window_manager;
use window_manager::{WindowConfig, WindowManager};

/// Query process information efficiently without full hardware/network scans.
/// Reuses an allocated `sysinfo::System` instance across queries.
fn with_refreshed_processes<F, R>(f: F) -> R
where
    F: FnOnce(&sysinfo::System) -> R,
{
    static SYSTEM_INFO: OnceLock<Mutex<sysinfo::System>> = OnceLock::new();
    let mut sys = SYSTEM_INFO
        .get_or_init(|| Mutex::new(sysinfo::System::new()))
        .lock()
        .unwrap();
    sys.refresh_processes_specifics(sysinfo::ProcessRefreshKind::new());
    f(&sys)
}

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
#[serde(default)]
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

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            break_minutes: 20,
            break_seconds: 0,
            break_duration_minutes: 0,
            break_duration_seconds: 20,
            break_mode: "force".to_string(),
            auto_pause: false,
            meeting_detect: false,
            pre_break: false,
            pre_break_minutes: 0,
            pre_break_seconds: 30,
            break_chime: false,
            recurring: false,
            autostart: false,
            auto_start_timer: false,
        }
    }
}

#[tauri::command]
fn force_break_window(app_handle: tauri::AppHandle, duration: Option<u32>) -> Result<(), String> {
    let break_duration = duration.unwrap_or(300);
    info!(
        "💥 Creating force break window with duration: {} seconds",
        break_duration
    );

    WindowManager::create_force_break_windows(&app_handle, break_duration)
}

#[tauri::command]
fn close_window(app_handle: tauri::AppHandle, label: String) -> Result<(), String> {
    if label == "force_break" || label.starts_with("force_break") {
        WindowManager::close_all_force_break_windows(&app_handle);
        return Ok(());
    }
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
    info!(
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
    info!(
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
        info!("🫥 Main window hidden to system tray");
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
        info!("👁️ Main window restored from system tray");
    }
    Ok(())
}

#[tauri::command]
async fn quit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("🚪 Quitting application completely");
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
async fn open_url(app_handle: tauri::AppHandle, url: String) -> Result<(), String> {
    info!("🌐 Opening URL: {}", url);
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
fn show_update_notification(
    app_handle: tauri::AppHandle,
    version: String,
    notes: String,
    download_url: String,
    release_url: Option<String>,
    published_at: String,
) -> Result<(), String> {
    info!("🔔 Showing update notification for version: {}", version);

    let rel_url = release_url.unwrap_or_else(|| download_url.clone());
    WindowManager::close_existing_window(&app_handle, "update_notification");
    let config =
        WindowConfig::update_notification(&app_handle, version, notes, download_url, rel_url, published_at);
    WindowManager::create_window(app_handle, config)
}

#[derive(Clone, Serialize)]
struct DownloadProgressPayload {
    downloaded: u64,
    total: u64,
    percent: f64,
}

#[tauri::command]
async fn download_and_install_update(
    app_handle: tauri::AppHandle,
    download_url: String,
) -> Result<(), String> {
    info!("📥 Direct update download initiated: {}", download_url);

    let handle = app_handle.clone();
    let temp_installer_path = tauri::async_runtime::spawn_blocking(move || -> Result<std::path::PathBuf, String> {
        let temp_dir = std::env::temp_dir();
        let installer_path = temp_dir.join("BreakReminderPro-Update.exe");

        if installer_path.exists() {
            let _ = fs::remove_file(&installer_path);
        }

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(180))
            .redirects(5)
            .build();

        let response = agent
            .get(&download_url)
            .call()
            .map_err(|e| format!("Download request failed: {}", e))?;

        let total_size: u64 = response
            .header("content-length")
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(0);

        let mut reader = response.into_reader();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&installer_path)
            .map_err(|e| format!("Failed to create temp installer file: {}", e))?;

        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
        let mut downloaded: u64 = 0;
        let mut last_emit_percent = -1.0;

        loop {
            let bytes_read = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("Error reading download stream: {}", e)),
            };

            file.write_all(&buffer[..bytes_read])
                .map_err(|e| format!("Error writing installer to disk: {}", e))?;

            downloaded += bytes_read as u64;

            let percent = if total_size > 0 {
                (downloaded as f64 / total_size as f64 * 100.0).min(100.0)
            } else {
                0.0
            };

            if (percent - last_emit_percent) >= 1.0 || percent >= 100.0 {
                last_emit_percent = percent;
                let _ = handle.emit(
                    "update-download-progress",
                    DownloadProgressPayload {
                        downloaded,
                        total: total_size,
                        percent,
                    },
                );
            }
        }

        file.flush().map_err(|e| format!("Failed to flush installer file: {}", e))?;
        info!("✅ Update downloaded to: {}", installer_path.display());
        Ok(installer_path)
    })
    .await
    .map_err(|e| format!("Download worker panicked: {}", e))??;

    info!("🚀 Executing update installer with /UPDATE /R: {}", temp_installer_path.display());

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new(&temp_installer_path);
        cmd.arg("/UPDATE");
        cmd.arg("/R");
        if let Err(e) = cmd.spawn() {
            return Err(format!("Failed to start installer: {}", e));
        }

        info!("🚪 Exiting Break Reminder Pro to allow in-place upgrade...");
        app_handle.exit(0);
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("In-app update is only supported on Windows".to_string())
    }
}


#[tauri::command]
fn lock_screen() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::LockWorkStation;
        let success = unsafe { LockWorkStation() };
        if success != 0 {
            info!("🔒 Screen locked successfully via LockWorkStation");
            Ok(())
        } else {
            Err("Failed to lock screen".to_string())
        }
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
                Err(e) => { info!("⚠️ CoCreateInstance failed: {:?}", e); return false; }
            };

        let device = match enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
            Ok(d) => d,
            Err(e) => { info!("⚠️ GetDefaultAudioEndpoint failed: {:?}", e); return false; }
        };

        // IMMDevice::Activate is generic in windows 0.52 — type inferred from return type
        let session_manager: IAudioSessionManager2 =
            match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(e) => { info!("⚠️ Activate IAudioSessionManager2 failed: {:?}", e); return false; }
            };

        let session_enum = match session_manager.GetSessionEnumerator() {
            Ok(e) => e,
            Err(e) => { info!("⚠️ GetSessionEnumerator failed: {:?}", e); return false; }
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
                info!("  VLC audio session state: {:?} → playing={}", state, is_active);
                return is_active;
            }
        }
        info!("  No audio session for VLC pid={} (no audio or muted)", vlc_pid);
        false
    }
}

/// Detects the lock screen by checking if LogonUI.exe is running — Windows always
/// launches this process when the workstation is locked, regardless of desktop access.
#[tauri::command]
fn is_screen_locked() -> bool {
    #[cfg(target_os = "windows")]
    {
        let locked = with_refreshed_processes(|sys| {
            sys.processes().values().any(|proc| {
                proc.name().to_lowercase() == "logonui.exe"
            })
        });
        if locked {
            info!("🔒 Screen is locked (LogonUI.exe detected)");
        } else {
            info!("🔓 Screen is not locked");
        }
        locked
    }
    #[cfg(not(target_os = "windows"))]
    false
}

#[tauri::command]
async fn control_media(action: String) -> Result<(), String> {
    info!("🎵 Media control requested: {}", action);

    #[cfg(target_os = "windows")]
    {
        // ── VLC: state-aware WM_APPCOMMAND ──────────────────────────────────────
        // VLC never registers with SMTC. We find its window and send WM_APPCOMMAND.
        {
            use winapi::um::winuser::{
                EnumWindows,
                SendMessageTimeoutW,
                WM_APPCOMMAND, SMTO_ABORTIFHUNG,
            };
            use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
            use winapi::shared::windef::HWND;

            let vlc_pid: Option<u32> = with_refreshed_processes(|sys| {
                sys.processes().values()
                    .find(|p| p.name().to_lowercase().contains("vlc"))
                    .map(|p| {
                        info!("  VLC process found: '{}' pid={}", p.name(), p.pid().as_u32());
                        p.pid().as_u32()
                    })
            });

            if let Some(pid) = vlc_pid {
                struct SearchData { pid: u32, hwnd: HWND }
                let mut data = SearchData { pid, hwnd: std::ptr::null_mut() };

                unsafe extern "system" fn find_vlc_visible_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
                    use winapi::um::winuser::{GetWindowTextLengthW, IsWindowVisible, GetWindowThreadProcessId};
                    let data = &mut *(lparam as *mut SearchData);
                    let mut wpid: u32 = 0;
                    GetWindowThreadProcessId(hwnd, &mut wpid);
                    if wpid != data.pid || IsWindowVisible(hwnd) == 0 {
                        return TRUE;
                    }
                    // Prefer windows that have a title (the main VLC window)
                    if GetWindowTextLengthW(hwnd) > 0 && data.hwnd.is_null() {
                        data.hwnd = hwnd;
                    }
                    TRUE
                }
                unsafe { EnumWindows(Some(find_vlc_visible_window), &mut data as *mut SearchData as LPARAM); }
                info!("  VLC window search result: hwnd={:?}", data.hwnd);

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
                            info!("  VLC audio state → playing={}, will_pause={}", playing, playing);
                            playing
                        }
                        "play" => {
                            let was = VLC_WAS_PLAYING.load(Ordering::SeqCst);
                            info!("  VLC was_playing={} will_resume={}", was, was);
                            was
                        }
                        _ => true,
                    };

                    if should_send {
                        let appcommand: i32 = match action.as_str() {
                            "pause" => APPCOMMAND_MEDIA_PAUSE as i32,
                            "play"  => APPCOMMAND_MEDIA_PLAY as i32,
                            _       => APPCOMMAND_MEDIA_PLAY_PAUSE as i32,
                        };
                        let lparam_val = (appcommand << 16) as isize;
                        let mut result: usize = 0;
                        let ret = unsafe {
                            SendMessageTimeoutW(
                                data.hwnd, WM_APPCOMMAND, data.hwnd as usize,
                                lparam_val, SMTO_ABORTIFHUNG, 1000, &mut result,
                            )
                        };
                        info!("✅ WM_APPCOMMAND {} sent to VLC (ret={} result={})", action, ret, result);
                    } else {
                        info!("⏭️ Skipping VLC WM_APPCOMMAND for action '{}' (wrong state)", action);
                    }
                } else {
                    info!("⚠️ VLC running but no visible window found");
                }
            } else {
                info!("ℹ️ VLC not running");
            }
        }

        // ── SMTC: selective pause/resume ────────────────────────────────────────
        // On pause: record which sources were Playing, pause only those.
        // On play:  resume only the sources we recorded — anything the user
        //           manually paused during the break is left alone.
        let smtc_action = action.clone();
        let smtc_thread = std::thread::spawn(move || {
            use windows::Media::Control::{
                GlobalSystemMediaTransportControlsSessionManager,
                GlobalSystemMediaTransportControlsSessionPlaybackStatus,
            };

            info!("🎵 [SMTC] Starting '{}' operation", smtc_action);

            let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .and_then(|op| op.get())
            {
                Ok(m) => m,
                Err(e) => { info!("⚠️ [SMTC] Manager failed: {:?}", e); return; }
            };

            let sessions_view = match manager.GetSessions() {
                Ok(s) => s,
                Err(e) => { info!("⚠️ [SMTC] GetSessions failed: {:?}", e); return; }
            };

            let count = sessions_view.Size().unwrap_or(0);
            info!("🎵 [SMTC] Found {} session(s)", count);

            if count == 0 {
                info!("⚠️ [SMTC] No sessions — nothing to '{}'", smtc_action);
                return;
            }

            match smtc_action.as_str() {
                "pause" => {
                    let mut paused_sources: Vec<String> = Vec::new();
                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) {
                            Ok(s) => s,
                            Err(e) => { info!("  ⚠️ GetAt({}) failed: {:?}", i, e); continue }
                        };
                        let source = session.SourceAppUserModelId()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|_| format!("session_{}", i));
                        let is_playing = session.GetPlaybackInfo()
                            .and_then(|info| info.PlaybackStatus())
                            .map(|st| st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                            .unwrap_or(false);

                        info!("  SMTC[{}] source={} playing={}", i, source, is_playing);

                        if is_playing {
                            match session.TryPauseAsync().and_then(|op| op.get()) {
                                Ok(_)  => { info!("  ⏸ Paused '{}'", source); paused_sources.push(source); }
                                Err(e) => info!("  ❌ Pause failed for '{}': {:?}", source, e),
                            }
                        } else {
                            info!("  ⏭️ Skipping '{}' (not playing)", source);
                        }
                    }
                    // Store which sources we paused so resume is selective
                    if let Ok(mut guard) = smtc_paused_sources().lock() {
                        *guard = paused_sources.clone();
                        info!("💾 [SMTC] Stored {} paused source(s): {:?}", paused_sources.len(), paused_sources);
                    }
                }
                "play" => {
                    // Only resume the exact sources we paused — leave anything the
                    // user manually paused during the break untouched.
                    let recorded = if let Ok(guard) = smtc_paused_sources().lock() {
                        guard.clone()
                    } else {
                        Vec::new()
                    };
                    info!("▶ [SMTC] Recorded sources to resume ({} total): {:?}", recorded.len(), recorded);

                    if recorded.is_empty() {
                        info!("⚠️ [SMTC] No recorded sources — nothing to resume");
                        return;
                    }

                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) {
                            Ok(s) => s,
                            Err(e) => { info!("  ⚠️ GetAt({}) failed: {:?}", i, e); continue }
                        };
                        let source = session.SourceAppUserModelId()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|_| format!("session_{}", i));

                        if recorded.contains(&source) {
                            info!("  ▶ Resuming '{}'", source);
                            match session.TryPlayAsync().and_then(|op| op.get()) {
                                Ok(_)  => info!("  ✅ Resumed '{}'", source),
                                Err(e) => info!("  ❌ Resume failed for '{}': {:?}", source, e),
                            }
                        } else {
                            info!("  ⏭️ Not our session, skipping '{}'", source);
                        }
                    }
                }
                _ => {
                    for i in 0..count {
                        let session = match sessions_view.GetAt(i) { Ok(s) => s, Err(_) => continue };
                        let _ = session.TryTogglePlayPauseAsync().and_then(|op| op.get());
                    }
                }
            }

            info!("🎵 [SMTC] '{}' operation complete", smtc_action);
        });

        // For 'pause': wait up to 5s — must finish before the break window opens.
        // For 'play':  wait up to 3s — slow resume is cosmetic, not critical.
        let timeout = if action == "pause" {
            std::time::Duration::from_secs(5)
        } else {
            std::time::Duration::from_secs(3)
        };
        let start = std::time::Instant::now();
        loop {
            if smtc_thread.is_finished() {
                let _ = smtc_thread.join();
                break;
            }
            if start.elapsed() >= timeout {
                info!("⚠️ [SMTC] Thread timed out after {}s", timeout.as_secs());
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        info!("🎵 [SMTC] control_media('{}') returning Ok", action);
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

/// Returns true if ANY media is currently playing — checks both SMTC sessions
/// (Spotify, browsers, etc.) AND VLC via its Core Audio session (VLC never
/// registers with SMTC so it must be checked separately).
#[tauri::command]
async fn is_media_playing() -> bool {
    #[cfg(target_os = "windows")]
    {
        // ── Check VLC via Core Audio session ──
        {
            let vlc_pid = with_refreshed_processes(|sys| {
                sys.processes().values()
                    .find(|p| p.name().to_lowercase().contains("vlc"))
                    .map(|p| p.pid().as_u32())
            });

            if let Some(pid) = vlc_pid {
                info!("🎵 VLC running (pid={}), checking audio session...", pid);
                if is_vlc_playing_via_audio(pid) {
                    info!("🎵 Media playing state: true (VLC active audio session)");
                    return true;
                }
                info!("🎵 VLC found but audio session inactive (paused/stopped)");
            }
        }

        // ── Check SMTC sessions (Spotify, YouTube, etc.) ──
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
                info!("🎵 SMTC sessions found: {}", count);
                for i in 0..count {
                    let session = match sessions.GetAt(i) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };
                    let source = session.SourceAppUserModelId()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| format!("session_{}", i));
                    let is_playing = session
                        .GetPlaybackInfo()
                        .and_then(|info| info.PlaybackStatus())
                        .map(|st| st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                        .unwrap_or(false);
                    info!("  SMTC [{}] '{}' playing={}", i, source, is_playing);
                    if is_playing {
                        return Ok::<bool, windows::core::Error>(true);
                    }
                }
                Ok::<bool, windows::core::Error>(false)
            }
            .await;

            match result {
                Ok(playing) => {
                    info!("🎵 Media playing state: {} (SMTC)", playing);
                    return playing;
                }
                Err(e) => {
                    info!("⚠️ Could not read SMTC playback state: {:?}", e);
                }
            }
        }

        false
    }

    #[cfg(not(target_os = "windows"))]
    false
}

#[tauri::command]
fn play_chime() -> Result<(), String> {
    info!("🔔 Playing chime sound...");

    #[cfg(target_os = "windows")]
    {
        use winapi::um::winuser::{MessageBeep, MB_ICONASTERISK};
        let success = unsafe { MessageBeep(MB_ICONASTERISK) };
        if success != 0 {
            info!("✅ Native chime played via MessageBeep");
            Ok(())
        } else {
            // Fallback with default beep
            unsafe { MessageBeep(0xFFFFFFFF); }
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Chime playback only supported on Windows".to_string())
    }
}

#[tauri::command]
fn is_meeting_active() -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Hardware check: Is microphone or webcam actively in use by a communication/interview app or browser?
        match is_hardware_in_use_by_meeting_app() {
            Ok(Some(info)) => {
                info!("🔍 Meeting / Interview detected via active hardware: {}", info);
                return Ok(Some(info));
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Failed to check hardware usage: {}", e);
            }
        }

        // 2. Presentation & Full-screen check: Screen-sharing, presentation, or full-screen assessment
        match is_presentation_or_fullscreen_active() {
            Ok(Some(info)) => {
                info!("🔍 Meeting / Interview detected via presentation or full-screen mode: {}", info);
                return Ok(Some(info));
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Failed to check presentation/full-screen mode: {}", e);
            }
        }

        // 3. Desktop window check: Specific meeting window signatures (Zoom, Teams, Webex)
        match check_meeting_windows() {
            Ok(Some(meeting_info)) => {
                info!("🔍 Meeting detected via active window: {}", meeting_info);
                return Ok(Some(meeting_info));
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Failed to check meeting windows: {}", e);
            }
        }

        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn is_hardware_in_use_by_meeting_app() -> Result<Option<String>, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use winapi::shared::minwindef::{DWORD, HKEY};
    use winapi::shared::winerror::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
    use winapi::um::winnt::KEY_READ;
    use winapi::um::winreg::{
        HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW,
    };

    const SUCCESS: i32 = ERROR_SUCCESS as i32;
    const NO_MORE_ITEMS: i32 = ERROR_NO_MORE_ITEMS as i32;

    fn to_wide(s: &str) -> Vec<u16> {
        use std::ffi::OsStr;
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let meeting_keywords = [
        "teams",
        "zoom",
        "webex",
        "skype",
        "meet",
        "slack",
        "discord",
        "chrome",
        "msedge",
        "firefox",
        "brave",
        "opera",
        "vivaldi",
    ];

    unsafe fn check_consent_store(
        root_key: HKEY,
        capability: &str,
        meeting_keywords: &[&str],
    ) -> Option<String> {
        // Check traditional NonPackaged desktop applications
        let non_packaged_str = format!(
            r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\{}\NonPackaged",
            capability
        );
        let non_packaged_path = to_wide(&non_packaged_str);
        let mut h_non_packaged: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(
            root_key,
            non_packaged_path.as_ptr(),
            0,
            KEY_READ,
            &mut h_non_packaged,
        ) == SUCCESS {
            let mut index = 0;
            let mut name_buf: [u16; 512] = [0; 512];

            loop {
                let mut name_len: DWORD = name_buf.len() as DWORD;
                let status = RegEnumKeyExW(
                    h_non_packaged,
                    index,
                    name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );

                if status == NO_MORE_ITEMS {
                    break;
                }
                if status != SUCCESS {
                    index += 1;
                    continue;
                }

                let subkey_name = OsString::from_wide(&name_buf[..name_len as usize])
                    .to_string_lossy()
                    .to_string();
                let subkey_lower = subkey_name.to_lowercase();

                if meeting_keywords.iter().any(|k| subkey_lower.contains(k)) {
                    let subkey_wide = to_wide(&subkey_name);
                    let mut h_app: HKEY = std::ptr::null_mut();
                    if RegOpenKeyExW(
                        h_non_packaged,
                        subkey_wide.as_ptr(),
                        0,
                        KEY_READ,
                        &mut h_app,
                    ) == SUCCESS {
                        let mut stop_time: u64 = 0;
                        let mut stop_size: DWORD = std::mem::size_of::<u64>() as DWORD;
                        let mut stop_type: DWORD = 0;
                        let stop_name = to_wide("LastUsedTimeStop");
                        let res_stop = RegQueryValueExW(
                            h_app,
                            stop_name.as_ptr(),
                            std::ptr::null_mut(),
                            &mut stop_type,
                            &mut stop_time as *mut u64 as *mut u8,
                            &mut stop_size,
                        );

                        let mut start_time: u64 = 0;
                        let mut start_size: DWORD = std::mem::size_of::<u64>() as DWORD;
                        let mut start_type: DWORD = 0;
                        let start_name = to_wide("LastUsedTimeStart");
                        let res_start = RegQueryValueExW(
                            h_app,
                            start_name.as_ptr(),
                            std::ptr::null_mut(),
                            &mut start_type,
                            &mut start_time as *mut u64 as *mut u8,
                            &mut start_size,
                        );

                        RegCloseKey(h_app);

                        if res_stop == SUCCESS && res_start == SUCCESS {
                            let is_active = (stop_time == 0 && start_time > 0) || (start_time > stop_time);
                            if is_active {
                                let exe_name = subkey_name
                                    .split('#')
                                    .last()
                                    .unwrap_or(&subkey_name)
                                    .to_string();
                                let exe_lower = exe_name.to_lowercase();

                                let is_running = with_refreshed_processes(|sys| {
                                    sys.processes().values().any(|p| {
                                        let p_name = p.name().to_lowercase();
                                        p_name == exe_lower
                                            || exe_lower.starts_with(&p_name)
                                            || p_name.starts_with(&exe_lower)
                                    })
                                });

                                if is_running {
                                    RegCloseKey(h_non_packaged);
                                    let device_label = if capability == "webcam" { "Webcam" } else { "Microphone" };
                                    return Some(format!("{} active by: {}", device_label, exe_name));
                                }
                            }
                        }
                    }
                }

                index += 1;
            }
            RegCloseKey(h_non_packaged);
        }

        // Check Packaged apps directly under ConsentStore\<capability>
        let packaged_str = format!(
            r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\{}",
            capability
        );
        let packaged_path = to_wide(&packaged_str);
        let mut h_packaged: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(
            root_key,
            packaged_path.as_ptr(),
            0,
            KEY_READ,
            &mut h_packaged,
        ) == SUCCESS {
            let mut index = 0;
            let mut name_buf: [u16; 512] = [0; 512];

            loop {
                let mut name_len: DWORD = name_buf.len() as DWORD;
                let status = RegEnumKeyExW(
                    h_packaged,
                    index,
                    name_buf.as_mut_ptr(),
                    &mut name_len,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );

                if status == NO_MORE_ITEMS {
                    break;
                }
                if status != SUCCESS {
                    index += 1;
                    continue;
                }

                let subkey_name = OsString::from_wide(&name_buf[..name_len as usize])
                    .to_string_lossy()
                    .to_string();
                let subkey_lower = subkey_name.to_lowercase();

                if subkey_lower != "nonpackaged"
                    && meeting_keywords.iter().any(|k| subkey_lower.contains(k))
                {
                    let subkey_wide = to_wide(&subkey_name);
                    let mut h_app: HKEY = std::ptr::null_mut();
                    if RegOpenKeyExW(
                        h_packaged,
                        subkey_wide.as_ptr(),
                        0,
                        KEY_READ,
                        &mut h_app,
                    ) == SUCCESS {
                        let mut stop_time: u64 = 0;
                        let mut stop_size: DWORD = std::mem::size_of::<u64>() as DWORD;
                        let mut stop_type: DWORD = 0;
                        let stop_name = to_wide("LastUsedTimeStop");
                        let res_stop = RegQueryValueExW(
                            h_app,
                            stop_name.as_ptr(),
                            std::ptr::null_mut(),
                            &mut stop_type,
                            &mut stop_time as *mut u64 as *mut u8,
                            &mut stop_size,
                        );

                        let mut start_time: u64 = 0;
                        let mut start_size: DWORD = std::mem::size_of::<u64>() as DWORD;
                        let mut start_type: DWORD = 0;
                        let start_name = to_wide("LastUsedTimeStart");
                        let res_start = RegQueryValueExW(
                            h_app,
                            start_name.as_ptr(),
                            std::ptr::null_mut(),
                            &mut start_type,
                            &mut start_time as *mut u64 as *mut u8,
                            &mut start_size,
                        );

                        RegCloseKey(h_app);

                        if res_stop == SUCCESS && res_start == SUCCESS {
                            let is_active = (stop_time == 0 && start_time > 0) || (start_time > stop_time);
                            if is_active {
                                let is_running = with_refreshed_processes(|sys| {
                                    sys.processes().values().any(|p| {
                                        let pname = p.name().to_lowercase();
                                        meeting_keywords.iter().any(|k| {
                                            subkey_lower.contains(k) && pname.contains(k)
                                        })
                                    })
                                });

                                if is_running {
                                    RegCloseKey(h_packaged);
                                    let device_label = if capability == "webcam" { "Webcam" } else { "Microphone" };
                                    return Some(format!("{} active by: {}", device_label, subkey_name));
                                }
                            }
                        }
                    }
                }

                index += 1;
            }
            RegCloseKey(h_packaged);
        }

        None
    }

    unsafe {
        for capability in &["microphone", "webcam"] {
            if let Some(app) = check_consent_store(HKEY_CURRENT_USER, capability, &meeting_keywords) {
                return Ok(Some(app));
            }
            if let Some(app) = check_consent_store(HKEY_LOCAL_MACHINE, capability, &meeting_keywords) {
                return Ok(Some(app));
            }
        }
    }

    Ok(None)
}

#[allow(dead_code)]
#[cfg(target_os = "windows")]
fn is_microphone_in_use_by_meeting_app() -> Result<Option<String>, String> {
    is_hardware_in_use_by_meeting_app()
}

#[cfg(target_os = "windows")]
fn classify_fullscreen_window(
    proc_name: &str,
    title: &str,
    class_name: &str,
) -> Option<String> {
    let proc_lower = proc_name.to_lowercase();
    let title_lower = title.to_lowercase();
    let class_lower = class_name.to_lowercase();

    // 1. Explicit Exclusions: Video playback, media streaming, and media players.
    // These frequently run in fullscreen, but MUST NEVER be flagged as meetings.
    let is_media_or_video = title_lower.contains("youtube")
        || title_lower.contains("netflix")
        || title_lower.contains("twitch")
        || title_lower.contains("prime video")
        || title_lower.contains("disney+")
        || title_lower.contains("hulu")
        || title_lower.contains("vimeo")
        || title_lower.contains("dailymotion")
        || title_lower.contains("crunchyroll")
        || title_lower.contains("bilibili")
        || title_lower.contains("plex")
        || title_lower.contains("stremio")
        || title_lower.contains("hotstar");

    let is_media_player = proc_lower.contains("vlc.exe")
        || proc_lower.contains("wmplayer.exe")
        || proc_lower.contains("mpv.exe")
        || proc_lower.contains("potplayer")
        || proc_lower.contains("kmplayer")
        || proc_lower.contains("gomp")
        || proc_lower.contains("foobar2000")
        || proc_lower.contains("musicbee")
        || proc_lower.contains("spotify");

    if is_media_or_video || is_media_player {
        return None;
    }

    // 2. PowerPoint Slide Show (class "screenClass" or "powerpnt.exe")
    if proc_lower.contains("powerpnt.exe") || class_lower == "screenclass" {
        return Some("PowerPoint Presentation".to_string());
    }

    // 3. Dedicated Presentation / Slide software
    if proc_lower.contains("impress.exe") || proc_lower.contains("keynote") {
        return Some("Presentation Active".to_string());
    }

    // 4. Meeting signatures in window title (Google Meet, Zoom, Teams, Webex)
    let meeting_title_keywords = [
        "google meet",
        "meet.google.com",
        "zoom meeting",
        "zoom webinar",
        "teams meeting",
        "webex meeting",
        "cisco webex",
    ];
    if meeting_title_keywords.iter().any(|k| title_lower.contains(k)) {
        return Some(format!("Meeting: {}", title));
    }

    // Also detect Google Meet tab title format ("Meet - ...")
    if title_lower.starts_with("meet - ") || title_lower.contains(" meet - ") {
        return Some(format!("Google Meet: {}", title));
    }

    // 5. Assessment / Online Interview platforms in fullscreen
    let assessment_keywords = [
        "hackerrank",
        "codility",
        "codesignal",
        "coderpad",
        "hirevue",
        "mettl",
        "testgorilla",
        "talview",
        "karat",
        "proctored exam",
        "proctor",
        "assessment test",
        "online assessment",
        "technical interview",
        "coding interview",
    ];
    if assessment_keywords.iter().any(|k| title_lower.contains(k)) {
        return Some(format!("Assessment / Interview: {}", title));
    }

    // Default: Generic full-screen windows (browsers browsing web, games, text editors) are NOT meetings
    None
}

#[cfg(target_os = "windows")]
fn is_presentation_or_fullscreen_active() -> Result<Option<String>, String> {
    use windows::Win32::UI::Shell::{
        SHQueryUserNotificationState,
        QUNS_BUSY,
        QUNS_PRESENTATION_MODE,
        QUNS_RUNNING_D3D_FULL_SCREEN,
    };
    use winapi::um::winuser::{
        GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetWindowRect, GetWindowTextW,
        GetWindowThreadProcessId,
    };
    use winapi::shared::windef::RECT;

    let res = unsafe { SHQueryUserNotificationState() };
    if let Ok(state) = res {
        // Presentation mode explicitly signaled by Windows Presentation Mode (e.g. PowerPoint or Windows Mobility Center)
        if state == QUNS_PRESENTATION_MODE {
            return Ok(Some("Presentation / Screen Share Mode".to_string()));
        }

        // When a window is running full-screen, inspect the foreground window to see if it is a
        // legitimate presentation, interview, or assessment.
        // NOTE: QUNS_RUNNING_D3D_FULL_SCREEN and QUNS_BUSY are also triggered by normal video
        // playback (e.g. YouTube in fullscreen), games, media players, and browser F11 fullscreen.
        // These MUST NOT be detected as meetings!
        if state == QUNS_BUSY || state == QUNS_RUNNING_D3D_FULL_SCREEN {
            unsafe {
                let fg_hwnd = GetForegroundWindow();
                if !fg_hwnd.is_null() {
                    let mut fg_rect: RECT = std::mem::zeroed();
                    let mut desk_rect: RECT = std::mem::zeroed();
                    GetWindowRect(fg_hwnd, &mut fg_rect);
                    GetWindowRect(GetDesktopWindow(), &mut desk_rect);

                    let covers_screen = fg_rect.left <= desk_rect.left
                        && fg_rect.top <= desk_rect.top
                        && fg_rect.right >= desk_rect.right
                        && fg_rect.bottom >= desk_rect.bottom;

                    if covers_screen {
                        let mut pid: u32 = 0;
                        GetWindowThreadProcessId(fg_hwnd, &mut pid);

                        let proc_name = with_refreshed_processes(|sys| {
                            sys.processes()
                                .values()
                                .find(|p| p.pid().as_u32() == pid)
                                .map(|p| p.name().to_string())
                        }).unwrap_or_default();

                        let mut title_buf = [0u16; 512];
                        let title_len = GetWindowTextW(fg_hwnd, title_buf.as_mut_ptr(), 512);
                        let title = String::from_utf16_lossy(&title_buf[..title_len.max(0) as usize]);

                        let mut class_buf = [0u16; 256];
                        let class_len = GetClassNameW(fg_hwnd, class_buf.as_mut_ptr(), 256);
                        let class_name = String::from_utf16_lossy(&class_buf[..class_len.max(0) as usize]);

                        if let Some(reason) = classify_fullscreen_window(&proc_name, &title, &class_name) {
                            return Ok(Some(reason));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}

#[cfg(target_os = "windows")]
fn check_meeting_windows() -> Result<Option<String>, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::shared::minwindef::{BOOL, FALSE, LPARAM, TRUE};
    use winapi::shared::windef::HWND;
    use winapi::um::winuser::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};

    // Note: Browser-based meetings (Google Meet, Teams web, etc.) are detected via active
    // microphone sessions in `is_microphone_in_use_by_meeting_app()`. Browsers keep tabs open
    // after a meeting ends (showing "You left the call"), so inspecting browser window titles
    // alone causes persistent false positives. Only check dedicated desktop meeting apps here.
    let (teams_pids, zoom_pids, webex_pids) = with_refreshed_processes(|sys| {
        let mut teams = Vec::new();
        let mut zoom = Vec::new();
        let mut webex = Vec::new();

        for (pid, proc) in sys.processes() {
            let name = proc.name().to_lowercase();
            let pid_u32 = pid.as_u32();
            if name.contains("teams.exe") || name.contains("ms-teams.exe") {
                teams.push(pid_u32);
            } else if name.contains("zoom.exe") {
                zoom.push(pid_u32);
            } else if name.contains("webex.exe") || name.contains("ciscowebexstart.exe") || name.contains("atmgr.exe") {
                webex.push(pid_u32);
            }
        }

        (teams, zoom, webex)
    });

    if teams_pids.is_empty() && zoom_pids.is_empty() && webex_pids.is_empty() {
        return Ok(None);
    }

    struct CallbackData {
        teams_pids: Vec<u32>,
        zoom_pids: Vec<u32>,
        webex_pids: Vec<u32>,
        detected_meeting: Option<String>,
    }

    let mut callback_data = CallbackData {
        teams_pids,
        zoom_pids,
        webex_pids,
        detected_meeting: None,
    };

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let callback_data = &mut *(lparam as *mut CallbackData);

        if IsWindowVisible(hwnd) == 0 {
            return TRUE;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);

        let is_teams = callback_data.teams_pids.contains(&pid);
        let is_zoom = callback_data.zoom_pids.contains(&pid);
        let is_webex = callback_data.webex_pids.contains(&pid);

        if !is_teams && !is_zoom && !is_webex {
            return TRUE;
        }

        let mut title: [u16; 512] = [0; 512];
        let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);

        if title_len > 0 {
            let title_os_string = OsString::from_wide(&title[..title_len as usize]);
            if let Ok(title_string) = title_os_string.into_string() {
                let title_lower = title_string.to_lowercase();

                // 1. Zoom active meeting window (dedicated window destroyed when meeting ends)
                if is_zoom {
                    if title_lower.contains("zoom meeting") || title_lower.contains("zoom webinar") {
                        info!("🔍 Zoom meeting window detected: {}", title_string);
                        callback_data.detected_meeting = Some(format!("Zoom Meeting: {}", title_string));
                        return FALSE;
                    }
                }

                // 2. Teams active call/meeting window (dedicated pop-out destroyed when call ends)
                if is_teams {
                    let is_meeting_window = title_lower.contains("(meeting)")
                        || title_lower.contains("meeting |")
                        || title_lower.contains("meeting with ")
                        || title_lower.contains("call with ")
                        || title_lower.contains("screen sharing")
                        || (title_lower.ends_with(" | microsoft teams")
                            && !title_lower.starts_with("chat |")
                            && !title_lower.starts_with("activity |")
                            && !title_lower.starts_with("calendar |")
                            && !title_lower.starts_with("teams |")
                            && !title_lower.starts_with("files |")
                            && title_lower != "microsoft teams");

                    if is_meeting_window {
                        info!("🔍 Teams meeting window detected: {}", title_string);
                        callback_data.detected_meeting = Some(format!("Teams Meeting: {}", title_string));
                        return FALSE;
                    }
                }

                // 3. Webex active meeting window (dedicated window destroyed when meeting ends)
                if is_webex {
                    if title_lower.contains("webex meeting") || title_lower.contains("cisco webex") {
                        info!("🔍 Webex meeting window detected: {}", title_string);
                        callback_data.detected_meeting = Some(format!("Webex Meeting: {}", title_string));
                        return FALSE;
                    }
                }
            }
        }

        TRUE
    }

    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut callback_data as *mut CallbackData as LPARAM,
        );
    }

    Ok(callback_data.detected_meeting)
}

#[allow(dead_code)]
#[cfg(target_os = "windows")]
fn check_browser_meetings() -> Result<bool, String> {
    match check_meeting_windows() {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    }
}

#[tauri::command]
fn check_browser_meeting_debug() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let mut debug_info = Vec::new();

        match is_hardware_in_use_by_meeting_app() {
            Ok(Some(app_name)) => {
                debug_info.push(app_name);
            }
            Ok(None) => {
                debug_info.push("Hardware: Idle (mic & webcam not in use by meeting/browser apps)".to_string());
            }
            Err(e) => {
                debug_info.push(format!("Hardware check error: {}", e));
            }
        }

        match is_presentation_or_fullscreen_active() {
            Ok(Some(info)) => {
                debug_info.push(format!("Display mode: {}", info));
            }
            Ok(None) => {
                debug_info.push("Display mode: Normal (not in presentation/full-screen)".to_string());
            }
            Err(e) => {
                debug_info.push(format!("Display mode error: {}", e));
            }
        }

        match check_meeting_windows() {
            Ok(Some(meeting_info)) => {
                debug_info.push(format!("Meeting window detected: {}", meeting_info));
            }
            Ok(None) => {
                debug_info.push("Meeting windows: None detected".to_string());
            }
            Err(e) => {
                debug_info.push(format!("Window check error: {}", e));
            }
        }

        let in_meeting = match is_meeting_active() {
            Ok(Some(reason)) => format!("Overall meeting status: ACTIVE ({})", reason),
            Ok(None) => "Overall meeting status: NOT ACTIVE".to_string(),
            Err(e) => format!("Overall meeting error: {}", e),
        };
        debug_info.push(in_meeting);

        Ok(debug_info.join(" | "))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok("Meeting detection is only supported on Windows".to_string())
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
    info!("🧪 Creating debug test window...");

    // Close existing window if it exists
    if let Some(existing) = app_handle.get_webview_window("debug_test") {
        info!("📄 Closing existing debug test window");
        let _ = existing.close();
    }

    // Create window in separate thread as recommended by Tauri docs
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        info!("📂 Attempting to load: test.html");
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
                info!("✅ Debug test window created successfully!");
                info!("🎯 Window label: {}", window.label());
                info!("📋 Expected content: Colorful gradient with TEST SUCCESS message");

                // Try to inject some debugging JavaScript after a delay
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window.eval("console.log('🔥 Debug test window JavaScript executed!'); document.title = 'TEST WINDOW LOADED';");
            }
            Err(e) => {
                info!("❌ Failed to create debug test window: {}", e);
            }
        }
    });

    info!("🚀 Debug test window creation initiated in separate thread");
    Ok(())
}

#[tauri::command]
async fn get_primary_monitor_size(app_handle: tauri::AppHandle) -> Result<(u32, u32), String> {
    match app_handle.primary_monitor() {
        Ok(Some(monitor)) => {
            let scale = monitor.scale_factor();
            let logical_size = monitor.size().to_logical::<f64>(scale);
            Ok((logical_size.width.round() as u32, logical_size.height.round() as u32))
        }
        Ok(None) => {
            warn!("No primary monitor found, using default size");
            Ok((800, 600))
        }
        Err(e) => {
            warn!("Error getting monitor info: {}, using default size", e);
            Ok((800, 600))
        }
    }
}

#[tauri::command]
fn meeting_detected_notification(app_handle: tauri::AppHandle, reason: Option<String>) -> Result<(), String> {
    info!("🤝 Creating meeting detected notification window (reason: {:?})...", reason);

    WindowManager::close_existing_window(&app_handle, "meeting_notification");
    let config = WindowConfig::meeting_notification(&app_handle, reason);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
fn break_ended_early(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("🏃 Break ended early - user returned");

    // Emit event across app
    if let Err(e) = app_handle.emit("break-ended-early", ()) {
        warn!("Failed to emit break-ended-early event: {}", e);
    }

    // Fallback invocation for direct JS callback
    if let Some(main_window) = app_handle.get_webview_window("main") {
        info!("📱 Found main window, calling handleEarlyBreakReturn fallback");
        let _ = main_window.eval("if (window.handleEarlyBreakReturn) { window.handleEarlyBreakReturn(); }");
    } else {
        info!("❌ Main window not found!");
    }

    Ok(())
}

#[tauri::command]
fn skip_break(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("⏭️ Skip break requested");

    // Close pre-break window if it exists
    if let Some(window) = app_handle.get_webview_window("pre_break") {
        let _ = window.close();
    }

    // Close any active break windows across all monitors
    WindowManager::close_all_force_break_windows(&app_handle);
    if let Some(window) = app_handle.get_webview_window("notify") {
        let _ = window.close();
    }

    // Emit event across app
    if let Err(e) = app_handle.emit("break-skipped", ()) {
        warn!("Failed to emit break-skipped event: {}", e);
    }

    // Fallback invocation for direct JS callback
    if let Some(main_window) = app_handle.get_webview_window("main") {
        info!("📱 Found main window, calling handleBreakSkipped fallback");
        let _ = main_window.eval("if (window.handleBreakSkipped) { window.handleBreakSkipped(); }");
    } else {
        info!("❌ Main window not found!");
    }

    info!("✅ Break skipped successfully");
    Ok(())
}

#[tauri::command]
fn show_index_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("Showing index window...");
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
                info!("🚪 Quit selected from tray menu");
                app.exit(0);
            }
            "show" => {
                info!("👁️ Show selected from tray menu");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                info!("🫥 Hide selected from tray menu");
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
                        info!("🫥 Main window hidden via tray click");
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        info!("👁️ Main window shown via tray click");
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

            // Self-heal autostart: ensure Windows registry matches settings.json
            if let Ok(Some(settings)) = load_settings(app.handle().clone()) {
                if settings.autostart {
                    let autostart_mgr = app.handle().autolaunch();
                    match autostart_mgr.is_enabled() {
                        Ok(false) => {
                            info!("🔄 Settings specify autostart=true but registry key is missing. Restoring...");
                            if let Err(e) = autostart_mgr.enable() {
                                warn!("⚠️ Failed to restore autostart on startup: {}", e);
                            } else {
                                info!("✅ Autostart successfully restored on startup");
                            }
                        }
                        Ok(true) => {
                            info!("✅ Autostart is active in Windows registry");
                        }
                        Err(e) => {
                            warn!("⚠️ Could not check autostart status on startup: {}", e);
                        }
                    }
                }
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
            download_and_install_update,
            set_media_was_playing,
            get_media_was_playing,
            clear_media_was_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_hardware_meeting_detection_runs() {
        let result = is_hardware_in_use_by_meeting_app();
        assert!(result.is_ok(), "is_hardware_in_use_by_meeting_app should not error");
        println!("Hardware (mic/webcam) meeting app result: {:?}", result.unwrap());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_presentation_or_fullscreen_detection_runs() {
        let result = is_presentation_or_fullscreen_active();
        assert!(result.is_ok(), "is_presentation_or_fullscreen_active should not error");
        println!("Presentation/fullscreen result: {:?}", result.unwrap());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_meeting_windows_detection_runs() {
        let result = check_meeting_windows();
        assert!(result.is_ok(), "check_meeting_windows should not error");
        println!("Meeting windows result: {:?}", result.unwrap());
    }

    #[test]
    fn test_is_meeting_active_runs() {
        let result = is_meeting_active();
        assert!(result.is_ok(), "is_meeting_active should not error");
        println!("is_meeting_active result: {:?}", result.unwrap());
    }

    #[test]
    fn test_check_browser_meeting_debug_runs() {
        let result = check_browser_meeting_debug();
        assert!(result.is_ok(), "check_browser_meeting_debug should not error");
        println!("check_browser_meeting_debug output: {:?}", result.unwrap());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_classify_fullscreen_window_youtube_ignored() {
        let chrome_yt = classify_fullscreen_window(
            "chrome.exe",
            "Rick Astley - Never Gonna Give You Up (Official Music Video) - YouTube - Google Chrome",
            "Chrome_WidgetWin_1",
        );
        assert_eq!(chrome_yt, None, "YouTube in Chrome must not be detected as meeting");

        let edge_yt = classify_fullscreen_window(
            "msedge.exe",
            "YouTube - Microsoft Edge",
            "Chrome_WidgetWin_1",
        );
        assert_eq!(edge_yt, None, "YouTube in Edge must not be detected as meeting");

        let firefox_yt = classify_fullscreen_window(
            "firefox.exe",
            "Lofi Hip Hop - YouTube — Mozilla Firefox",
            "MozillaWindowClass",
        );
        assert_eq!(firefox_yt, None, "YouTube in Firefox must not be detected as meeting");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_classify_fullscreen_window_media_players_ignored() {
        let vlc = classify_fullscreen_window("vlc.exe", "movie.mkv - VLC media player", "Qt5QWindowIcon");
        assert_eq!(vlc, None, "VLC must not be detected as meeting");

        let netflix = classify_fullscreen_window("chrome.exe", "Stranger Things | Netflix - Google Chrome", "Chrome_WidgetWin_1");
        assert_eq!(netflix, None, "Netflix must not be detected as meeting");

        let twitch = classify_fullscreen_window("chrome.exe", "Twitch - Google Chrome", "Chrome_WidgetWin_1");
        assert_eq!(twitch, None, "Twitch must not be detected as meeting");

        let game = classify_fullscreen_window("game.exe", "Cyberpunk 2077", "GameWindowClass");
        assert_eq!(game, None, "Games must not be detected as meeting");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_classify_fullscreen_window_presentation_detected() {
        let ppt = classify_fullscreen_window("powerpnt.exe", "PowerPoint Slide Show - [Deck1]", "screenClass");
        assert_eq!(ppt, Some("PowerPoint Presentation".to_string()));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_classify_fullscreen_window_assessments_and_meetings_detected() {
        let hackerrank = classify_fullscreen_window("chrome.exe", "Solve Challenge | HackerRank - Google Chrome", "Chrome_WidgetWin_1");
        assert!(hackerrank.is_some(), "HackerRank should be detected");

        let meet = classify_fullscreen_window("chrome.exe", "Meet - abc-defg-hij - Google Chrome", "Chrome_WidgetWin_1");
        assert!(meet.is_some(), "Google Meet should be detected");
    }
}

