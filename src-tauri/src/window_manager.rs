use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use std::thread;
use std::time::Duration;

pub struct WindowConfig {
    pub label: String,
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub fullscreen: bool,
    pub always_on_top: bool,
    pub decorations: bool,
    pub resizable: bool,
    pub focused: bool,
    pub visible: bool,
    pub skip_taskbar: bool,
    pub maximized: bool,
    pub transparent: bool,
    pub shadow: bool,
    pub position: Option<(f64, f64)>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            url: String::new(),
            title: String::new(),
            width: 400.0,
            height: 300.0,
            fullscreen: false,
            always_on_top: false,
            decorations: true,
            resizable: true,
            focused: true,
            visible: false, // Start hidden to prevent flash
            skip_taskbar: false,
            maximized: false,
            transparent: false,
            shadow: true,
            position: None,
        }
    }
}

pub struct WindowManager;

impl WindowManager {
    pub fn create_window(app_handle: AppHandle, config: WindowConfig) -> Result<(), String> {
        let handle = app_handle.clone();
        
        thread::spawn(move || {
            println!("📂 Creating window '{}' with URL: {}", config.label, config.url);
            
            let mut builder = WebviewWindowBuilder::new(
                &handle,
                &config.label,
                WebviewUrl::App(config.url.into())
            )
            .title(&config.title)
            .inner_size(config.width, config.height)
            .fullscreen(config.fullscreen)
            .always_on_top(config.always_on_top)
            .decorations(config.decorations)
            .resizable(config.resizable)
            .focused(config.focused)
            .visible(config.visible)
            .skip_taskbar(config.skip_taskbar)
            .maximized(config.maximized)
            .transparent(config.transparent)
            .shadow(config.shadow);

            // Set position if provided
            if let Some((x, y)) = config.position {
                builder = builder.position(x, y);
                println!("🎯 Setting initial position for '{}': ({:.0}, {:.0})", config.label, x, y);
            }

            match builder.build() {
                Ok(window) => {
                    println!("✅ Window '{}' created successfully!", config.label);
                    
                    // Inject positioning signal IMMEDIATELY to prevent JS positioning conflicts
                    if config.position.is_some() {
                        let js_code = format!(
                            "window.RUST_POSITIONED = true; \
                             console.log('🔥 Window {} positioned by Rust - JS positioning disabled');",
                            config.label
                        );
                        let _ = window.eval(&js_code);
                    }
                    
                    // Wait for content to load
                    thread::sleep(Duration::from_millis(200));
                    
                    // Double-check position if specified
                    if let Some((x, y)) = config.position {
                        if let Err(e) = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { 
                            x: x as i32, 
                            y: y as i32 
                        })) {
                            println!("⚠️ Failed to set position for '{}': {}", config.label, e);
                        } else {
                            println!("✅ Position confirmed for '{}': ({:.0}, {:.0})", config.label, x, y);
                        }
                    }
                    
                    // Show window after positioning
                    if let Err(e) = window.show() {
                        println!("⚠️ Failed to show window '{}': {}", config.label, e);
                    } else {
                        println!("✅ Window '{}' shown successfully", config.label);
                    }
                    
                    // Set always on top for notification windows
                    if config.always_on_top {
                        if let Err(e) = window.set_always_on_top(true) {
                            println!("⚠️ Failed to set always on top for '{}': {}", config.label, e);
                        }
                    }
                    
                    // Focus if required
                    if config.focused {
                        if let Err(e) = window.set_focus() {
                            println!("⚠️ Failed to focus window '{}': {}", config.label, e);
                        }
                    }
                    
                    // Final positioning confirmation
                    thread::sleep(Duration::from_millis(100));
                    if config.position.is_some() {
                        let final_js = format!(
                            "console.log('🎯 Final positioning confirmation for {}'); \
                             document.title = '{} - POSITIONED BY RUST';",
                            config.label, config.title
                        );
                        let _ = window.eval(&final_js);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create window '{}': {}", config.label, e);
                }
            }
        });
        
        Ok(())
    }

    pub fn get_screen_center_position(app_handle: &AppHandle, window_width: f64, window_height: f64) -> (f64, f64) {
        match app_handle.primary_monitor() {
            Ok(Some(monitor)) => {
                let size = monitor.size();
                let screen_width = size.width as f64;
                let screen_height = size.height as f64;
                
                let x = (screen_width - window_width) / 2.0;
                let y = (screen_height - window_height) / 2.0;
                
                println!("📺 Screen: {}x{}, Window: {}x{}, Center: ({:.0}, {:.0})", 
                    screen_width, screen_height, window_width, window_height, x, y);
                
                (x, y)
            }
            _ => {
                println!("⚠️ Could not get monitor info, using default center position");
                (200.0, 200.0)
            }
        }
    }

    pub fn get_bottom_center_position(app_handle: &AppHandle, window_width: f64, window_height: f64, margin_bottom: f64) -> (f64, f64) {
        match app_handle.primary_monitor() {
            Ok(Some(monitor)) => {
                let size = monitor.size();
                let screen_width = size.width as f64;
                let screen_height = size.height as f64;
                
                let x = (screen_width - window_width) / 2.0;
                let y = screen_height - window_height - margin_bottom;
                
                println!("📺 Screen: {}x{}, Window: {}x{}, Bottom Center: ({:.0}, {:.0})", 
                    screen_width, screen_height, window_width, window_height, x, y);
                
                (x, y)
            }
            _ => {
                println!("⚠️ Could not get monitor info, using default bottom center position");
                (200.0, 400.0)
            }
        }
    }

    pub fn close_existing_window(app_handle: &AppHandle, label: &str) {
        if let Some(existing) = app_handle.get_webview_window(label) {
            println!("📄 Closing existing window '{}'", label);
            let _ = existing.close();
        }
    }
}

// Predefined window configurations
impl WindowConfig {
    pub fn force_break(duration: u32) -> Self {
        Self {
            label: "force_break".to_string(),
            url: format!("force_break.html?duration={}", duration),
            title: "Break Time".to_string(),
            width: 1920.0,
            height: 1080.0,
            fullscreen: true,
            always_on_top: true,
            decorations: false,
            resizable: false,
            focused: true,
            visible: false,
            skip_taskbar: true,
            maximized: true,
            transparent: false,
            shadow: false,
            position: None,
        }
    }

    pub fn notify(app_handle: &AppHandle, duration: u32) -> Self {
        let window_width = 480.0;
        let window_height = 350.0; // Increased height for better content fit
        let position = WindowManager::get_screen_center_position(app_handle, window_width, window_height);
        
        Self {
            label: "notify".to_string(),
            url: format!("notify.html?duration={}", duration),
            title: "Break Time - Break Reminder Pro".to_string(),
            width: window_width,
            height: window_height,
            fullscreen: false,
            always_on_top: true,
            decorations: true,
            resizable: false,
            focused: true,
            visible: false,
            skip_taskbar: false,
            maximized: false,
            transparent: false,
            shadow: true,
            position: Some(position),
        }
    }

    pub fn pre_break(app_handle: &AppHandle, remaining_seconds: u32) -> Self {
        let window_width = 220.0;
        let window_height = 90.0;
        let position = WindowManager::get_bottom_center_position(app_handle, window_width, window_height, 120.0);
        
        Self {
            label: "pre_break".to_string(),
            url: format!("pre_break.html?seconds={}", remaining_seconds),
            title: "Pre-Break Warning".to_string(),
            width: window_width,
            height: window_height,
            fullscreen: false,
            always_on_top: true,
            decorations: false,
            resizable: false,
            focused: false,
            visible: false,
            skip_taskbar: true,
            maximized: false,
            transparent: true,
            shadow: false,
            position: Some(position),
        }
    }

    pub fn meeting_notification(app_handle: &AppHandle) -> Self {
        let window_width = 240.0;
        let window_height = 90.0;
        let position = WindowManager::get_bottom_center_position(app_handle, window_width, window_height, 220.0);
        
        Self {
            label: "meeting_notification".to_string(),
            url: "meeting_notification.html".to_string(),
            title: "Meeting Detected".to_string(),
            width: window_width,
            height: window_height,
            fullscreen: false,
            always_on_top: true,
            decorations: false,
            resizable: false,
            focused: false,
            visible: false,
            skip_taskbar: true,
            maximized: false,
            transparent: true,
            shadow: false,
            position: Some(position),
        }
    }

    pub fn update_notification(
        app_handle: &AppHandle,
        version: String,
        notes: String,
        download_url: String,
        published_at: String,
    ) -> Self {
        let window_width = 450.0;
        let window_height = 300.0;
        let position = WindowManager::get_screen_center_position(app_handle, window_width, window_height);
        
        // URL encode the parameters
        let encoded_version = urlencoding::encode(&version);
        let encoded_notes = urlencoding::encode(&notes);
        let encoded_url = urlencoding::encode(&download_url);
        let encoded_date = urlencoding::encode(&published_at);
        
        Self {
            label: "update_notification".to_string(),
            url: format!(
                "update_notification.html?version={}&notes={}&url={}&date={}",
                encoded_version, encoded_notes, encoded_url, encoded_date
            ),
            title: "Update Available".to_string(),
            width: window_width,
            height: window_height,
            fullscreen: false,
            always_on_top: true,
            decorations: true,
            resizable: false,
            focused: true,
            visible: false,
            skip_taskbar: false,
            maximized: false,
            transparent: false,
            shadow: true,
            position: Some(position),
        }
    }
}