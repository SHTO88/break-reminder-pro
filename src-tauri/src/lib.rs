use serde::{Deserialize, Serialize};
use std::fs;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, AppHandle, WindowEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

// For autostart plugin
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

// Import our window manager module
mod window_manager;
use window_manager::{WindowManager, WindowConfig};

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
    println!("💥 Creating force break window with duration: {} seconds", break_duration);

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
    println!("🔔 Creating notify window with duration: {} seconds", break_duration);

    WindowManager::close_existing_window(&app_handle, "notify");
    let config = WindowConfig::notify(&app_handle, break_duration);
    WindowManager::create_window(app_handle, config)
}

#[tauri::command]
fn pre_break_notification_window(app_handle: tauri::AppHandle, remaining_seconds: Option<u32>) -> Result<(), String> {
    let seconds = remaining_seconds.unwrap_or(30);
    println!("⏰ Creating pre-break window with {} seconds remaining...", seconds);

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
        window.hide().map_err(|e| format!("Failed to hide window: {}", e))?;
        println!("🫥 Main window hidden to system tray");
    }
    Ok(())
}

#[tauri::command]
async fn show_from_tray(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| format!("Failed to show window: {}", e))?;
        window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
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
    let config = WindowConfig::update_notification(&app_handle, version, notes, download_url, published_at);
    WindowManager::create_window(app_handle, config)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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

#[tauri::command]
fn control_media(action: &str) -> Result<(), String> {
    use enigo::{Enigo, Key, KeyboardControllable};
    let mut enigo = Enigo::new();
    
    println!("🎵 Media control requested: {}", action);
    
    match action {
        "pause" => {
            // Use MediaStop instead of MediaPlayPause to avoid toggle behavior
            // MediaStop will stop playing media and do nothing if already stopped
            println!("🛑 Sending MediaStop key (safer than toggle)");
            enigo.key_click(Key::MediaStop);
            Ok(())
        }
        "playpause" => {
            // Keep the toggle behavior for explicit play/pause requests
            println!("⏯️ Sending MediaPlayPause key (toggle)");
            enigo.key_click(Key::MediaPlayPause);
            Ok(())
        }
        _ => Err("Unsupported media action".to_string()),
    }
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
    use winapi::um::winuser::{EnumWindows, GetWindowTextW, IsWindowVisible};
    use winapi::shared::windef::HWND;
    use winapi::shared::minwindef::{BOOL, LPARAM, TRUE, FALSE};
    
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
        meeting_indicators: meeting_indicators.iter().map(|s| s.to_lowercase()).collect(),
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
            Err(e) => Ok(format!("Error checking browser meetings: {}", e))
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
    let quit_item = MenuItem::with_id(app, "quit", "Quit Break Reminder Pro", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
    
    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;
    
    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Break Reminder Pro - Click to toggle window")
        .icon(app.default_window_icon().unwrap().clone())
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
    
    println!("✅ System tray initialized successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // Setup system tray
            if let Err(e) = setup_system_tray(app.handle()) {
                println!("❌ Failed to setup system tray: {}", e);
            }
            
            // Handle window close events to hide to tray instead of closing
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing
                        api.prevent_close();
                        
                        // Hide the window instead
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                            println!("🫥 Main window hidden to tray instead of closing");
                        }
                    }
                });
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            lock_screen,
            control_media,
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
            show_update_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
