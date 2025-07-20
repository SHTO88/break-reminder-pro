use tauri::{WebviewWindowBuilder, WebviewUrl, Manager};
use serde::{Deserialize, Serialize};
use std::fs;

// For autostart plugin
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

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
    let break_duration = duration.unwrap_or(300); // Default 5 minutes
    let url = format!("force_break.html?duration={}", break_duration);
    
    println!("💥 Creating force break window with duration: {} seconds", break_duration);
    
    // Create window in separate thread as recommended by Tauri docs
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        println!("📂 Attempting to load: {}", url);
        match WebviewWindowBuilder::new(&handle, "force_break", WebviewUrl::App(url.into()))
            .fullscreen(true)
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .focused(true)
            .visible(true)
            .skip_taskbar(true)
            .build()
        {
            Ok(window) => {
                println!("✅ Force break window created successfully!");
                println!("🎯 Window label: {}", window.label());
                println!("📋 Expected content: Fullscreen break window with countdown");
            }
            Err(e) => {
                println!("❌ Failed to create force break window: {}", e);
            }
        }
    });
    
    println!("🚀 Force break window creation initiated in separate thread");
    Ok(())
}

#[tauri::command]
fn close_window(app_handle: tauri::AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&label) {
        window.close().map_err(|e| format!("Failed to close window {}: {}", label, e))?;
    }
    Ok(())
}

#[tauri::command]
fn notify_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🔔 Creating notify window...");
    
    // Close existing window if it exists
    if let Some(existing) = app_handle.get_webview_window("notify") {
        println!("📄 Closing existing notify window");
        let _ = existing.close();
    }
    
    // Create window in separate thread as recommended by Tauri docs
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        println!("📂 Attempting to load: notify.html");
        match WebviewWindowBuilder::new(&handle, "notify", WebviewUrl::App("notify.html".into()))
            .title("Break Notification")
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .inner_size(400.0, 200.0)
            .position(100.0, 100.0)
            .focused(true)
            .visible(true)
            .build()
        {
            Ok(window) => {
                println!("✅ Notify window created successfully!");
                println!("🎯 Window label: {}", window.label());
                println!("📋 Expected content: Blue notification with countdown");
                
                // Try to inject some debugging JavaScript after a delay
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window.eval("console.log('🔥 Notify window JavaScript executed!'); document.title = 'NOTIFY WINDOW LOADED';");
            }
            Err(e) => {
                println!("❌ Failed to create notify window: {}", e);
            }
        }
    });
    
    println!("🚀 Notify window creation initiated in separate thread");
    Ok(())
}

#[tauri::command]
fn pre_break_notification_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("⏰ Creating pre-break window...");
    
    // Close existing window if it exists
    if let Some(existing) = app_handle.get_webview_window("pre_break") {
        println!("📄 Closing existing pre-break window");
        let _ = existing.close();
    }
    
    // Create window in separate thread as recommended by Tauri docs
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        println!("📂 Attempting to load: pre_break.html");
        match WebviewWindowBuilder::new(&handle, "pre_break", WebviewUrl::App("pre_break.html".into()))
            .title("Pre-Break Warning")
            .always_on_top(true)
            .decorations(false)
            .resizable(false)
            .inner_size(350.0, 180.0)
            .position(150.0, 150.0)
            .focused(true)
            .visible(true)
            .build()
        {
            Ok(window) => {
                println!("✅ Pre-break window created successfully!");
                println!("🎯 Window label: {}", window.label());
                println!("📋 Expected content: Yellow pre-break warning with countdown");
                
                // Try to inject some debugging JavaScript after a delay
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window.eval("console.log('🔥 Pre-break window JavaScript executed!'); document.title = 'PRE-BREAK WINDOW LOADED';");
            }
            Err(e) => {
                println!("❌ Failed to create pre-break window: {}", e);
            }
        }
    });
    
    println!("🚀 Pre-break window creation initiated in separate thread");
    Ok(())
}

#[tauri::command]
async fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager.enable()
        .map_err(|e| format!("Failed to enable autostart: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager.disable()
        .map_err(|e| format!("Failed to disable autostart: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app_handle.autolaunch();
    autostart_manager.is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
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
    match action {
        "pause" | "playpause" => {
            enigo.key_click(Key::MediaPlayPause);
            Ok(())
        }
        _ => Err("Unsupported media action".to_string()),
    }
}

#[tauri::command]
fn is_meeting_active() -> Result<bool, String> {
    use sysinfo::System;
    let meeting_processes = ["zoom.exe", "teams.exe", "skype.exe", "webex.exe", "meet.exe"];
    let sys = System::new_all();
    let found = sys.processes().values().any(|proc| {
        let name = proc.name().to_lowercase();
        meeting_processes.iter().any(|mp| name.contains(mp))
    });
    Ok(found)
}

#[tauri::command]
fn save_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let app_data_dir = app_handle.path().app_data_dir()
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
    let app_data_dir = app_handle.path().app_data_dir()
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())

        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None
        ))
        .invoke_handler(tauri::generate_handler![
            greet,
            lock_screen,
            control_media,
            is_meeting_active,
            force_break_window,
            close_window,
            notify_window,
            pre_break_notification_window,
            enable_autostart,
            disable_autostart,
            is_autostart_enabled,
            save_settings,
            load_settings,
            debug_test_window,
            show_index_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
