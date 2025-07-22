const { invoke } = window.__TAURI__.core;

// Default settings
const defaultSettings = {
  break_minutes: 20,
  break_seconds: 0,
  break_duration_minutes: 0,
  break_duration_seconds: 20,
  break_mode: 'force',
  auto_pause: false,
  meeting_detect: false,
  pre_break: false,
  pre_break_minutes: 0,
  pre_break_seconds: 30,
  break_chime: false,
  recurring: false,
  autostart: false,
  auto_start_timer: false
};

// Tab navigation
function initTabs() {
  const tabs = document.querySelectorAll('.tab');
  const tabContents = document.querySelectorAll('.tab-content');
  
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetTab = tab.dataset.tab;
      
      // Update active tab
      tabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      
      // Show target content
      tabContents.forEach(content => content.classList.remove('active'));
      document.getElementById(`${targetTab}-tab`).classList.add('active');
    });
  });
}

// Settings management
async function saveSettings() {
  try {
    const settings = {
      break_minutes: 50, // These are handled on main page
      break_seconds: 0,
      break_duration_minutes: 10,
      break_duration_seconds: 0,
      break_mode: 'force',
      auto_pause: document.getElementById("auto-pause").checked,
      meeting_detect: document.getElementById("meeting-detect").checked,
      pre_break: document.getElementById("pre-break").checked,
      pre_break_minutes: parseInt(document.getElementById("pre-break-minutes").value) || 0,
      pre_break_seconds: parseInt(document.getElementById("pre-break-seconds").value) || 0,
      break_chime: document.getElementById("break-chime").checked,
      recurring: document.getElementById("recurring").checked,
      autostart: document.getElementById("autostart").checked,
      auto_start_timer: false
    };
    
    // Load existing settings to preserve values from main page
    const existingSettings = await loadSettings();
    const mergedSettings = { ...existingSettings, ...settings };
    
    await invoke('save_settings', { settings: mergedSettings });
    console.log('Settings saved:', mergedSettings);
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

async function loadSettings() {
  try {
    const settings = await invoke('load_settings');
    if (settings) {
      console.log('Settings loaded:', settings);
      return settings;
    }
    return defaultSettings;
  } catch (error) {
    console.error('Failed to load settings:', error);
    return defaultSettings;
  }
}

function applySettingsToUI(settings) {
  try {
    // Set checkboxes
    document.getElementById("auto-pause").checked = settings.auto_pause;
    document.getElementById("meeting-detect").checked = settings.meeting_detect;
    document.getElementById("pre-break").checked = settings.pre_break;
    document.getElementById("pre-break-minutes").value = settings.pre_break_minutes || 0;
    document.getElementById("pre-break-seconds").value = settings.pre_break_seconds || 30;
    document.getElementById("break-chime").checked = settings.break_chime;
    document.getElementById("recurring").checked = settings.recurring;
    document.getElementById("autostart").checked = settings.autostart;
    
    // Update pre-break timing visibility
    updatePreBreakTimingVisibility();
    
    console.log('Settings applied to UI:', settings);
  } catch (error) {
    console.error('Failed to apply settings to UI:', error);
  }
}

function updatePreBreakTimingVisibility() {
  const preBreakEnabled = document.getElementById("pre-break").checked;
  const timingCard = document.getElementById("pre-break-timing-card");
  
  if (timingCard) {
    timingCard.style.display = preBreakEnabled ? 'block' : 'none';
  }
}

// Autostart handling
async function handleAutostartToggle(enabled) {
  try {
    if (enabled) {
      await invoke('enable_autostart');
      console.log('Autostart enabled');
    } else {
      await invoke('disable_autostart');
      console.log('Autostart disabled');
    }
  } catch (error) {
    console.error('Failed to toggle autostart:', error);
    document.getElementById("autostart").checked = !enabled;
  }
}

// Input validation
function formatTimeInput(input, maxValue) {
  let value = parseInt(input.value, 10);
  if (isNaN(value) || value < 0) {
    input.value = 0;
  } else if (value > maxValue) {
    input.value = maxValue;
  } else {
    input.value = value;
  }
}

// Debug functions
function debugLog(message) {
  console.log(`[DEBUG] ${message}`);
  
  const output = document.getElementById('debug-output');
  if (output) {
    const timestamp = new Date().toLocaleTimeString();
    output.textContent += `[${timestamp}] ${message}\n`;
    output.scrollTop = output.scrollHeight;
  }
}

function debugClear() {
  const output = document.getElementById('debug-output');
  if (output) {
    output.textContent = 'Debug output cleared...\n';
  }
  debugLog('Debug console cleared');
}

function setQuickTimer(seconds) {
  debugLog(`Setting ${seconds} second timer`);
  // This would need to communicate with main window
  debugLog(`Timer functionality available on main page`);
}

async function debugForceBreak() {
  try {
    debugLog('Triggering force break window...');
    await invoke("force_break_window", { duration: 300 });
    debugLog(`✅ Force break window created`);
  } catch (error) {
    debugLog(`❌ Error triggering force break: ${error}`);
  }
}

async function debugPreBreak() {
  try {
    debugLog('Triggering pre-break notification...');
    await invoke("pre_break_notification_window");
    debugLog('✅ Pre-break notification window created');
  } catch (error) {
    debugLog(`❌ Error triggering pre-break: ${error}`);
  }
}

async function debugNotify() {
  try {
    debugLog('Triggering notify window...');
    await invoke("notify_window");
    debugLog('✅ Notify window created');
  } catch (error) {
    debugLog(`❌ Error triggering notify: ${error}`);
  }
}

async function debugLockScreen() {
  try {
    debugLog('Triggering screen lock...');
    await invoke("lock_screen");
    debugLog('Screen lock triggered');
  } catch (error) {
    debugLog(`Error locking screen: ${error}`);
  }
}

async function debugPlayChime() {
  try {
    debugLog('Testing chime sound...');
    await invoke("play_chime");
    debugLog('✅ Chime played successfully');
  } catch (error) {
    debugLog(`❌ Error playing chime: ${error}`);
  }
}

async function debugMediaPause() {
  try {
    debugLog('Testing media pause...');
    await invoke("control_media", { action: "pause" });
    debugLog('Media pause command sent');
  } catch (error) {
    debugLog(`Error pausing media: ${error}`);
  }
}

async function debugMeetingCheck() {
  try {
    debugLog('Checking meeting status...');
    const inMeeting = await invoke("is_meeting_active");
    debugLog(`Meeting detection result: ${inMeeting ? 'In meeting' : 'No meeting detected'}`);
  } catch (error) {
    debugLog(`Error checking meeting status: ${error}`);
  }
}

async function debugBrowserMeetingCheck() {
  try {
    debugLog('Checking browser meeting status...');
    const result = await invoke("check_browser_meeting_debug");
    debugLog(`Browser meeting detection: ${result}`);
  } catch (error) {
    debugLog(`Error checking browser meeting status: ${error}`);
  }
}

async function debugAutostartCheck() {
  try {
    debugLog('Checking autostart status...');
    const isEnabled = await invoke("is_autostart_enabled");
    debugLog(`Autostart status: ${isEnabled ? 'Enabled' : 'Disabled'}`);
  } catch (error) {
    debugLog(`Error checking autostart: ${error}`);
  }
}

async function debugCloseWindow(windowLabel) {
  try {
    debugLog(`Closing ${windowLabel} window...`);
    await invoke("close_window", { label: windowLabel });
    debugLog(`${windowLabel} window closed`);
  } catch (error) {
    debugLog(`Error closing ${windowLabel} window: ${error}`);
  }
}

async function debugClearSettings() {
  try {
    debugLog('Clearing all settings...');
    if (confirm('Are you sure you want to clear all settings? This cannot be undone.')) {
      await invoke('save_settings', { settings: defaultSettings });
      debugLog('Settings cleared and reset to defaults');
      
      applySettingsToUI(defaultSettings);
      debugLog('UI reset to default values');
    } else {
      debugLog('Settings clear cancelled by user');
    }
  } catch (error) {
    debugLog(`Error clearing settings: ${error}`);
  }
}

async function debugTestSequence() {
  try {
    debugLog('=== STARTING FULL TEST SEQUENCE ===');
    
    debugLog('1. Testing settings persistence...');
    await saveSettings();
    await loadSettings();
    debugLog('Settings test completed');
    
    debugLog('2. Testing system features...');
    await debugMeetingCheck();
    await debugBrowserMeetingCheck();
    await debugAutostartCheck();
    await debugMediaPause();
    
    debugLog('3. Testing notification windows...');
    await debugPreBreak();
    await new Promise(resolve => setTimeout(resolve, 2000));
    await debugCloseWindow('pre_break');
    
    await debugNotify();
    await new Promise(resolve => setTimeout(resolve, 2000));
    await debugCloseWindow('notify');
    
    debugLog('=== FULL TEST SEQUENCE COMPLETED ===');
    
  } catch (error) {
    debugLog(`Error in test sequence: ${error}`);
  }
}

function debugHelp() {
  debugLog('=== BREAK REMINDER PRO DEBUG HELP ===');
  debugLog('');
  debugLog('BREAK TESTING:');
  debugLog('  🖥️ Force Break - Opens fullscreen break window');
  debugLog('  ⏰ Pre-Break - Shows pre-break warning notification');
  debugLog('  🔔 Notify - Opens break notification window');
  debugLog('  🔒 Lock Screen - Locks the computer screen');
  debugLog('');
  debugLog('QUICK TIMERS:');
  debugLog('  ⏱️ Timer functions - Available on main timer page');
  debugLog('');
  debugLog('SYSTEM TESTS:');
  debugLog('  🔔 Play Chime - Tests break notification sound');
  debugLog('  ⏸️ Media Pause - Tests media control functionality');
  debugLog('  👥 Meeting Check - Tests desktop meeting detection');
  debugLog('  🌐 Browser Meeting - Tests browser meeting detection');
  debugLog('  🚀 Autostart - Tests Windows autostart status');
  debugLog('  🗑️ Clear Settings - Resets all settings to defaults');
  debugLog('');
  debugLog('WINDOW CONTROLS:');
  debugLog('  ❌ Close windows - Closes specific break windows');
  debugLog('  🧪 Full Test - Runs comprehensive test sequence');
  debugLog('=====================================');
}

// Initialize application
window.addEventListener("DOMContentLoaded", async () => {
  // Initialize tabs
  initTabs();
  
  // Load and apply saved settings
  const savedSettings = await loadSettings();
  applySettingsToUI(savedSettings);
  
  // Settings form
  const settingsInputs = [
    "auto-pause", "meeting-detect", "pre-break", "break-chime", "recurring"
  ];
  
  settingsInputs.forEach(id => {
    const element = document.getElementById(id);
    if (element) {
      element.addEventListener('change', saveSettings);
    }
  });
  
  // Autostart toggle
  document.getElementById("autostart").addEventListener('change', async (e) => {
    await handleAutostartToggle(e.target.checked);
    await saveSettings();
  });
  
  // Pre-break toggle
  document.getElementById("pre-break").addEventListener('change', () => {
    updatePreBreakTimingVisibility();
    saveSettings();
  });
  
  // Time input validation
  const timeInputs = [
    { element: document.getElementById("pre-break-minutes"), max: 10 },
    { element: document.getElementById("pre-break-seconds"), max: 59 }
  ];
  
  timeInputs.forEach(({ element, max }) => {
    element.addEventListener('blur', () => {
      formatTimeInput(element, max);
      saveSettings();
    });
    element.addEventListener('input', () => {
      if (element.value.length > 3) {
        element.value = element.value.slice(0, 3);
      }
    });
  });
  
  // Debug event listeners
  document.getElementById('debug-force-break').addEventListener('click', debugForceBreak);
  document.getElementById('debug-pre-break').addEventListener('click', debugPreBreak);
  document.getElementById('debug-notify').addEventListener('click', debugNotify);
  document.getElementById('debug-lock').addEventListener('click', debugLockScreen);
  
  document.getElementById('debug-timer-10s').addEventListener('click', () => setQuickTimer(10));
  document.getElementById('debug-timer-30s').addEventListener('click', () => setQuickTimer(30));
  document.getElementById('debug-timer-60s').addEventListener('click', () => setQuickTimer(60));
  document.getElementById('debug-timer-5m').addEventListener('click', () => setQuickTimer(300));
  
  document.getElementById('debug-play-chime').addEventListener('click', debugPlayChime);
  document.getElementById('debug-media-pause').addEventListener('click', debugMediaPause);
  document.getElementById('debug-meeting-check').addEventListener('click', debugMeetingCheck);
  document.getElementById('debug-browser-meeting-check').addEventListener('click', debugBrowserMeetingCheck);
  document.getElementById('debug-autostart-check').addEventListener('click', debugAutostartCheck);
  document.getElementById('debug-clear-settings').addEventListener('click', debugClearSettings);
  
  document.getElementById('debug-close-force').addEventListener('click', () => debugCloseWindow('force_break'));
  document.getElementById('debug-close-notify').addEventListener('click', () => debugCloseWindow('notify'));
  document.getElementById('debug-close-prebreak').addEventListener('click', () => debugCloseWindow('pre_break'));
  document.getElementById('debug-test-sequence').addEventListener('click', debugTestSequence);
  
  document.getElementById('debug-help').addEventListener('click', debugHelp);
  document.getElementById('debug-clear').addEventListener('click', debugClear);
  
  // Check autostart status on startup
  try {
    const isEnabled = await invoke('is_autostart_enabled');
    if (isEnabled !== document.getElementById("autostart").checked) {
      document.getElementById("autostart").checked = isEnabled;
      await saveSettings();
    }
  } catch (error) {
    console.error('Failed to check autostart status:', error);
  }
  
  debugLog('Break Reminder Pro settings page initialized successfully');
  console.log('Break Reminder Pro settings loaded successfully!');
});