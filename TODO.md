# Break Reminder Pro - TODO List

This list tracks the development progress based on the project's PRD.

**Phase 1: Project Scaffolding & Core UI**
- [ ] Initialize Tauri Project (`npm create tauri-app@latest`)
- [ ] Design Main Settings UI (HTML/CSS)
- [ ] Implement Frontend Timer Logic (JavaScript)

**Phase 2: Backend Rust Commands**
- [ ] Expose Rust functions as Tauri commands
- [ ] Implement `lock_screen` command
- [ ] Implement `control_media` command (using `enigo` crate)
- [ ] Implement `is_meeting_active` command (using `sysinfo` crate)

**Phase 3: Window Management & Break UI**
- [ ] Create "Force Break" window (fullscreen, always-on-top)
- [ ] Create "Notify" window (small, always-on-top)
- [ ] Create "Pre-Break Notification" windows

**Phase 4: Integration & Finalization**
- [ ] Connect UI controls to backend Rust commands
- [ ] Implement meeting detection logic flow
- [ ] Implement auto-pause media logic flow
- [ ] Integrate `tauri-plugin-store` for settings persistence
- [ ] Integrate `tauri-plugin-autostart` for auto-start functionality
- [ ] Implement end-of-break audio chime
- [ ] Final testing and packaging into a Windows installer (.msi)
