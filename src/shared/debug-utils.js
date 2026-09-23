/**
 * Shared Debug Utilities Module
 * Common debugging functions used across the application
 */

const { invoke } = window.__TAURI__.core;

/**
 * Debug Utilities Class
 */
export class DebugUtils {
  /**
   * Log debug message with timestamp
   */
  static log(message, outputElementId = "debug-output") {
    console.log(`[DEBUG] ${message}`);

    const output = document.getElementById(outputElementId);
    if (output) {
      const timestamp = new Date().toLocaleTimeString();
      output.textContent += `[${timestamp}] ${message}\n`;
      output.scrollTop = output.scrollHeight;
    }
  }

  /**
   * Clear debug output
   */
  static clear(outputElementId = "debug-output") {
    const output = document.getElementById(outputElementId);
    if (output) {
      output.textContent = "Debug output cleared...\n";
    }
    this.log("Debug console cleared");
  }

  /**
   * Test break window functions
   */
  static async testForceBreak(duration = 300) {
    try {
      this.log("Triggering force break window...");
      await invoke("force_break_window", { duration });
      this.log(`✅ Force break window created with ${duration}s duration`);
    } catch (error) {
      this.log(`❌ Error triggering force break: ${error}`);
    }
  }

  static async testPreBreak() {
    try {
      this.log("Triggering pre-break notification...");
      await invoke("pre_break_notification_window", { remainingSeconds: 30 });
      this.log("✅ Pre-break notification window created");
    } catch (error) {
      this.log(`❌ Error triggering pre-break: ${error}`);
    }
  }

  static async testNotify(duration = 600) {
    try {
      this.log("Triggering notify window...");
      await invoke("notify_window", { duration });
      this.log(`✅ Notify window created with ${duration}s duration`);
    } catch (error) {
      this.log(`❌ Error triggering notify: ${error}`);
    }
  }

  static async testLockScreen() {
    try {
      this.log("Triggering screen lock...");
      await invoke("lock_screen");
      this.log("✅ Screen lock triggered");
    } catch (error) {
      this.log(`❌ Error locking screen: ${error}`);
    }
  }

  /**
   * Test system features
   */
  static async testPlayChime() {
    try {
      this.log("Testing chime sound...");
      await invoke("play_chime");
      this.log("✅ Chime played successfully");
    } catch (error) {
      this.log(`❌ Error playing chime: ${error}`);
    }
  }

  static async testMediaPause() {
    try {
      this.log("Testing media pause...");
      await invoke("control_media", { action: "pause" });
      this.log("✅ Media pause command sent");
    } catch (error) {
      this.log(`❌ Error pausing media: ${error}`);
    }
  }

  static async testMeetingCheck() {
    try {
      this.log("Checking meeting status...");
      const meetingReason = await invoke("is_meeting_active");
      this.log(
        `Meeting detection result: ${meetingReason ? `In meeting (${meetingReason})` : "No meeting detected"}`,
      );
    } catch (error) {
      this.log(`❌ Error checking meeting status: ${error}`);
    }
  }

  static async testBrowserMeetingCheck() {
    try {
      this.log("Checking browser meeting status...");
      const result = await invoke("check_browser_meeting_debug");
      this.log(`Browser meeting detection: ${result}`);
    } catch (error) {
      this.log(`❌ Error checking browser meeting status: ${error}`);
    }
  }

  static async testMeetingNotification() {
    try {
      this.log("Testing meeting detected notification...");
      await invoke("meeting_detected_notification", {
        reason: "Debug Test (Microphone active by: msedge.exe)",
      });
      this.log("✅ Meeting notification window created successfully");
    } catch (error) {
      this.log(`❌ Error showing meeting notification: ${error}`);
    }
  }

  static async testAutostartCheck() {
    try {
      this.log("Checking autostart status...");
      const isEnabled = await invoke("is_autostart_enabled");
      this.log(`Autostart status: ${isEnabled ? "Enabled" : "Disabled"}`);
    } catch (error) {
      this.log(`❌ Error checking autostart: ${error}`);
    }
  }

  static async testUpdateCheck() {
    try {
      this.log("Opening update notification with mock data...");
      await invoke("show_update_notification", {
        version: "v0.0.0-TEST",
        notes: [
          "## What's New",
          "",
          "### ✨ New Features",
          "- Added smart break scheduling based on activity detection",
          "- New **Focus Mode** that blocks distracting apps during breaks",
          "- Pre-break warning now supports custom messages",
          "",
          "### 🐛 Bug Fixes",
          "- Fixed crash on break end when media player was running",
          "- Resolved 00:00 freeze caused by blocking `play_chime` call",
          "- Fixed nested scrollbar in update notification window",
          "",
          "### ⚡ Performance",
          "- Reduced startup time by ~40%",
          "- Media control now non-blocking for instant break close",
          "",
          "---",
          "_Thank you for using Break Reminder Pro!_",
        ].join("\n"),
        downloadUrl: "https://github.com",
        publishedAt: new Date().toISOString(),
      });
      this.log("✅ Update notification window opened");
    } catch (error) {
      this.log(`❌ Error showing update notification: ${error}`);
    }
  }

  /**
   * Window control functions
   */
  static async closeWindow(windowLabel) {
    try {
      this.log(`Closing ${windowLabel} window...`);
      await invoke("close_window", { label: windowLabel });
      this.log(`✅ ${windowLabel} window closed`);
    } catch (error) {
      this.log(`❌ Error closing ${windowLabel} window: ${error}`);
    }
  }

  /**
   * Settings management
   */
  static async clearSettings(defaultSettings) {
    try {
      this.log("Clearing all settings...");
      if (
        confirm(
          "Are you sure you want to clear all settings? This cannot be undone.",
        )
      ) {
        await invoke("save_settings", { settings: defaultSettings });
        this.log("✅ Settings cleared and reset to defaults");
        return true;
      } else {
        this.log("Settings clear cancelled by user");
        return false;
      }
    } catch (error) {
      this.log(`❌ Error clearing settings: ${error}`);
      return false;
    }
  }

  /**
   * Run comprehensive test sequence
   */
  static async runTestSequence() {
    try {
      this.log("=== STARTING FULL TEST SEQUENCE ===");

      this.log("1. Testing system features...");
      await this.testMeetingCheck();
      await this.testBrowserMeetingCheck();
      await this.testAutostartCheck();
      await this.testMediaPause();

      this.log("2. Testing notification windows...");
      await this.testPreBreak();
      await new Promise((resolve) => setTimeout(resolve, 2000));
      await this.closeWindow("pre_break");

      await this.testNotify();
      await new Promise((resolve) => setTimeout(resolve, 2000));
      await this.closeWindow("notify");

      this.log("=== FULL TEST SEQUENCE COMPLETED ===");
    } catch (error) {
      this.log(`❌ Error in test sequence: ${error}`);
    }
  }

  /**
   * Show debug help
   */
  static showHelp() {
    this.log("=== BREAK REMINDER PRO DEBUG HELP ===");
    this.log("");
    this.log("BREAK TESTING:");
    this.log("  🖥️ Force Break - Opens fullscreen break window");
    this.log("  ⏰ Pre-Break - Shows pre-break warning notification");
    this.log("  🔔 Notify - Opens break notification window");
    this.log("  🔒 Lock Screen - Locks the computer screen");
    this.log("");
    this.log("SYSTEM TESTS:");
    this.log("  🔔 Play Chime - Tests break notification sound");
    this.log("  ⏸️ Media Pause - Tests media control functionality");
    this.log("  👥 Meeting Check - Tests desktop meeting detection");
    this.log("  🌐 Browser Meeting - Tests browser meeting detection");
    this.log("  🚀 Autostart - Tests Windows autostart status");
    this.log("  🆕 Update Notification - Opens update window with mock data");
    this.log("  🗑️ Clear Settings - Resets all settings to defaults");
    this.log("");
    this.log("WINDOW CONTROLS:");
    this.log("  ❌ Close windows - Closes specific break windows");
    this.log("  🧪 Full Test - Runs comprehensive test sequence");
    this.log("=====================================");
  }

  /**
   * Setup debug event listeners
   */
  static setupDebugListeners(buttonMappings) {
    Object.entries(buttonMappings).forEach(([buttonId, action]) => {
      const button = document.getElementById(buttonId);
      if (button) {
        button.addEventListener("click", action);
      }
    });
  }
}
