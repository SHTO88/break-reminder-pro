### **Product Requirements Document: Break Reminder Pro (Tauri Edition)**

**1. Overview**
Break Reminder Pro is a minimalist, resource-efficient Windows desktop application built with Tauri. It is designed to help users maintain their health and focus by providing configurable, assertive, and intelligent break notifications.

**2. Target Audience**
Windows users who spend long hours at their computer, including professionals, developers, students, and gamers.

**3. Core Features**

*   **3.1. Reminder Configuration:**
    *   Set time *until* break (in minutes).
    *   Set break *duration* (in minutes).
    *   Enable/disable recurring break cycle.

*   **3.2. Break Modes:**
    *   **Force Break:** A full-screen, always-on-top, black window with a minimalist countdown timer.
    *   **Notify:** A persistent, custom, non-blocking notification window with a countdown timer.
    *   **Lock Screen:** Triggers the native Windows lock screen.

*   **3.3. Smart Features & Customization (Toggles):**
    *   **Auto-Pause Media:** Pauses video/music when a break starts.
    *   **Meeting Detection:** Skips "Force Break" or "Lock Screen" if a video meeting is active, showing a simple notification instead.
    *   **End-of-Break Chime:** Plays a subtle sound when the break is over.
    *   **Pre-Break Notifications:** Shows small, custom notifications 5 minutes and 1 minute before a break.

*   **3.4. System Integration:**
    *   **Auto-Start with Windows:** Option to launch the app on system startup.
    *   **State Persistence:** Remembers all user settings and the timer state across app restarts.

**4. Technical Stack**

*   **Framework:** **Tauri 2.0**
*   **Backend:** **Rust**
*   **Frontend:** **Vanilla JavaScript, HTML5, CSS3**. This keeps the application lightweight and fast. We will use modern CSS for styling to ensure a beautiful UI.
*   **State Management:**
    *   **Persistence:** `tauri-plugin-store` to save and load user settings to a local file.
    *   **Auto-Start:** `tauri-plugin-autostart` to manage the "start with Windows" feature.
*   **Core Rust Crates:**
    *   `tauri`: The main framework crate.
    *   `sysinfo`: To get the list of running processes for the meeting detection feature.
    *   `enigo`: To simulate media key presses (Play/Pause) for the auto-pause feature. This provides broad compatibility.

**5. Implementation Plan**

I will build the application in the following sequence:

*   **Phase 1: Project Scaffolding & Core UI**
    1.  **Initialize Project:** Use `npm create tauri-app@latest` to generate the project structure.
    2.  **Build Settings UI:** Design the main window UI using HTML and CSS. This will include inputs for timers, radio buttons for modes, and toggles for all smart features.
    3.  **Frontend Logic:** Implement the core timer countdown logic in JavaScript.

*   **Phase 2: Backend Rust Commands**
    1.  **Create Tauri Commands:** Expose Rust functions to the JavaScript frontend.
    2.  **Implement `lock_screen`:** Write a Rust function that executes the Windows command to lock the screen.
    3.  **Implement `control_media(action)`:** Write a Rust function using the `enigo` crate to simulate a `MediaPlayPause` key press.
    4.  **Implement `is_meeting_active`:** Write a Rust function using the `sysinfo` crate to check for running processes like `zoom.exe`, `teams.exe`, etc.

*   **Phase 3: Window Management & Break UI**
    1.  **Create Break Windows:** Use Tauri's window management APIs from Rust to create new webview windows for the "Force Break" and "Notify" modes.
    2.  **Style Break Windows:**
        *   **Force Break:** Configure the window to be fullscreen, always-on-top, and undecorated. The frontend for this window will be a simple black page with a countdown.
        *   **Notify & Pre-Break:** Configure these windows to be small, undecorated, and always-on-top.

*   **Phase 4: Integration & Finalization**
    1.  **Connect Frontend to Backend:** Wire the UI controls to call the Rust commands.
    2.  **Implement Smart Logic:**
        *   Before a forced break, call `is_meeting_active`. If it returns true, show a notification window instead.
        *   When a break starts, call `control_media('pause')`.
    3.  **Add Plugins:** Integrate `tauri-plugin-store` to save all settings and `tauri-plugin-autostart` for the startup option.
    4.  **Add Audio:** Implement the end-of-break chime using a JavaScript `Audio` object.
    5.  **Testing & Packaging:** Thoroughly test all features and then build the final MSI installer for Windows.
