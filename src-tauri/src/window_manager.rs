use log::{info, warn};
use tauri::{AppHandle, LogicalPosition, Manager, Position, WebviewUrl, WebviewWindowBuilder};
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
    /// Optional JS snippet injected after the window loads (used to pass rich
    /// data that would be unsafe or too long to embed in a query string).
    pub inject_js: Option<String>,
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
            inject_js: None,
        }
    }
}

pub struct WindowManager;

impl WindowManager {
    pub fn create_window(app_handle: AppHandle, config: WindowConfig) -> Result<(), String> {
        let handle = app_handle.clone();
        
        thread::spawn(move || {
            info!("Creating window '{}' url='{}'", config.label, config.url);
            
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
                builder = builder.initialization_script("window.RUST_POSITIONED = true;");
            }

            // Inject any custom JS payload (runs before DOM loads, eliminating race conditions)
            if let Some(ref js) = config.inject_js {
                builder = builder.initialization_script(js);
            }

            match builder.build() {
                Ok(window) => {
                    info!("Window '{}' created", config.label);

                    // Confirm position in logical coordinates
                    if let Some((x, y)) = config.position {
                        if let Err(e) = window.set_position(Position::Logical(LogicalPosition { x, y })) {
                            warn!("Failed to confirm position for '{}': {}", config.label, e);
                        }
                    }
                    
                    // Show window
                    if let Err(e) = window.show() {
                        warn!("Failed to show window '{}': {}", config.label, e);
                    } else {
                        info!("Window '{}' shown", config.label);
                    }
                    
                    // Focus if required
                    if config.focused {
                        if let Err(e) = window.set_focus() {
                            warn!("Failed to focus window '{}': {}", config.label, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create window '{}': {}", config.label, e);
                }
            }
        });
        
        Ok(())
    }

    /// Spawns a force break overlay on every available monitor to prevent multi-monitor leak.
    pub fn create_force_break_windows(app_handle: &AppHandle, duration: u32) -> Result<(), String> {
        Self::close_all_force_break_windows(app_handle);

        let monitors = app_handle.available_monitors().unwrap_or_default();
        let primary_monitor = app_handle.primary_monitor().ok().flatten();

        if monitors.is_empty() {
            let config = WindowConfig::force_break(duration);
            return Self::create_window(app_handle.clone(), config);
        }

        let primary_pos = primary_monitor.as_ref().map(|m| *m.position());
        let mut primary_created = false;
        let mut secondary_idx = 1;

        for monitor in &monitors {
            let is_primary = primary_pos.map_or(!primary_created, |p| p == *monitor.position());

            let (label, url) = if is_primary && !primary_created {
                primary_created = true;
                (
                    "force_break".to_string(),
                    format!("force_break.html?duration={}", duration),
                )
            } else {
                let lbl = format!("force_break_{}", secondary_idx);
                secondary_idx += 1;
                (
                    lbl,
                    format!("force_break.html?duration={}&secondary=true", duration),
                )
            };

            let scale = monitor.scale_factor();
            let mon_pos = monitor.position().to_logical::<f64>(scale);
            let mon_size = monitor.size().to_logical::<f64>(scale);

            let config = WindowConfig {
                label,
                url,
                title: "Break Time".to_string(),
                width: mon_size.width,
                height: mon_size.height,
                fullscreen: true,
                always_on_top: true,
                decorations: false,
                resizable: false,
                focused: is_primary,
                visible: false,
                skip_taskbar: true,
                maximized: true,
                transparent: false,
                shadow: false,
                position: Some((mon_pos.x, mon_pos.y)),
                inject_js: None,
            };

            let _ = Self::create_window(app_handle.clone(), config);
        }

        Ok(())
    }

    /// Closes all force break windows across all monitors.
    pub fn close_all_force_break_windows(app_handle: &AppHandle) {
        for (label, window) in app_handle.webview_windows() {
            if label.starts_with("force_break") {
                info!("Closing force break window '{}'", label);
                let _ = window.close();
            }
        }
        thread::sleep(Duration::from_millis(150));
    }

    pub fn get_screen_center_position(app_handle: &AppHandle, window_width: f64, window_height: f64) -> (f64, f64) {
        match app_handle.primary_monitor() {
            Ok(Some(monitor)) => {
                let scale = monitor.scale_factor();
                let mon_size = monitor.size().to_logical::<f64>(scale);
                let mon_pos = monitor.position().to_logical::<f64>(scale);
                
                let x = mon_pos.x + (mon_size.width - window_width) / 2.0;
                let y = mon_pos.y + (mon_size.height - window_height) / 2.0;
                
                info!("📺 Primary Monitor (scale {:.2}): logical size {:.0}x{:.0}, pos ({:.0}, {:.0}) -> Center: ({:.0}, {:.0})", 
                    scale, mon_size.width, mon_size.height, mon_pos.x, mon_pos.y, x, y);
                
                (x, y)
            }
            _ => {
                warn!("⚠️ Could not get monitor info, using default center position");
                (200.0, 200.0)
            }
        }
    }

    pub fn get_bottom_center_position(app_handle: &AppHandle, window_width: f64, window_height: f64, margin_bottom: f64) -> (f64, f64) {
        match app_handle.primary_monitor() {
            Ok(Some(monitor)) => {
                let scale = monitor.scale_factor();
                let mon_size = monitor.size().to_logical::<f64>(scale);
                let mon_pos = monitor.position().to_logical::<f64>(scale);
                
                let x = mon_pos.x + (mon_size.width - window_width) / 2.0;
                let y = mon_pos.y + mon_size.height - window_height - margin_bottom;
                
                info!("📺 Primary Monitor (scale {:.2}): logical size {:.0}x{:.0}, pos ({:.0}, {:.0}) -> Bottom Center: ({:.0}, {:.0})", 
                    scale, mon_size.width, mon_size.height, mon_pos.x, mon_pos.y, x, y);
                
                (x, y)
            }
            _ => {
                warn!("⚠️ Could not get monitor info, using default bottom center position");
                (200.0, 400.0)
            }
        }
    }

    pub fn close_existing_window(app_handle: &AppHandle, label: &str) {
        if label == "force_break" || label.starts_with("force_break") {
            Self::close_all_force_break_windows(app_handle);
            return;
        }
        if let Some(existing) = app_handle.get_webview_window(label) {
            info!("Closing existing window '{}'", label);
            let _ = existing.close();
            // Brief pause to let the close event propagate before the caller
            // tries to create a new window with the same label.
            thread::sleep(Duration::from_millis(150));
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
            inject_js: None,
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
            inject_js: None,
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
            inject_js: None,
        }
    }

    pub fn meeting_notification(app_handle: &AppHandle, reason: Option<String>) -> Self {
        let window_width = 260.0;
        let window_height = 96.0;
        let position = WindowManager::get_bottom_center_position(app_handle, window_width, window_height, 220.0);
        
        let url = match reason {
            Some(ref r) if !r.is_empty() => {
                format!("meeting_notification.html?reason={}", urlencoding::encode(r))
            }
            _ => "meeting_notification.html".to_string(),
        };

        Self {
            label: "meeting_notification".to_string(),
            url,
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
            inject_js: None,
        }
    }

    pub fn update_notification(
        app_handle: &AppHandle,
        version: String,
        notes: String,
        download_url: String,
        release_url: String,
        published_at: String,
    ) -> Self {
        let window_width = 500.0;
        let window_height = 520.0;
        let position = WindowManager::get_screen_center_position(app_handle, window_width, window_height);

        // Escape the strings for safe embedding in a JS string literal.
        // Using JSON serialization is the safest way — serde_json escapes all
        // special characters including newlines, quotes, and backslashes.
        let version_json     = serde_json::to_string(&version).unwrap_or_else(|_| "\"\"".into());
        let notes_json       = serde_json::to_string(&notes).unwrap_or_else(|_| "\"\"".into());
        let url_json         = serde_json::to_string(&download_url).unwrap_or_else(|_| "\"\"".into());
        let rel_url_json     = serde_json::to_string(&release_url).unwrap_or_else(|_| "\"\"".into());
        let published_json   = serde_json::to_string(&published_at).unwrap_or_else(|_| "\"\"".into());

        let inject = format!(
            "window.__UPDATE_DATA__ = {{ version: {}, notes: {}, downloadUrl: {}, releaseUrl: {}, publishedAt: {} }}; \
             console.log('✅ Update data injected for version:', window.__UPDATE_DATA__.version);",
            version_json, notes_json, url_json, rel_url_json, published_json
        );

        Self {
            label: "update_notification".to_string(),
            url: "update_notification.html".to_string(),
            title: "Update Available".to_string(),
            width: window_width,
            height: window_height,
            fullscreen: false,
            always_on_top: true,
            decorations: true,
            resizable: true,
            focused: true,
            visible: false,
            skip_taskbar: false,
            maximized: false,
            transparent: false,
            shadow: true,
            position: Some(position),
            inject_js: Some(inject),
        }
    }
}