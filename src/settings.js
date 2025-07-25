import { settingsManager, DEFAULT_SETTINGS } from './shared/settings.js';
import { UIUtils, TabManager } from './shared/ui-utils.js';
import { DebugUtils } from './shared/debug-utils.js';
import { updateManager } from './shared/update-manager.js';

const { invoke } = window.__TAURI__.core;

// Tab navigation
const tabManager = new TabManager();

// Settings management
async function saveSettings() {
  try {
    const newSettings = {
      auto_pause: document.getElementById("auto-pause").checked,
      meeting_detect: document.getElementById("meeting-detect").checked,
      pre_break: document.getElementById("pre-break").checked,
      pre_break_minutes: parseInt(document.getElementById("pre-break-minutes").value) || 0,
      pre_break_seconds: parseInt(document.getElementById("pre-break-seconds").value) || 0,
      break_chime: document.getElementById("break-chime").checked,
      autostart: document.getElementById("autostart").checked
    };

    await settingsManager.save(newSettings);
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

async function loadSettings() {
  return await settingsManager.load();
}

function applySettingsToUI(settings) {
  try {
    const fieldMappings = {
      auto_pause: 'auto-pause',
      meeting_detect: 'meeting-detect',
      pre_break: 'pre-break',
      pre_break_minutes: 'pre-break-minutes',
      pre_break_seconds: 'pre-break-seconds',
      break_chime: 'break-chime',
      autostart: 'autostart'
    };

    UIUtils.applySettingsToForm(settings, fieldMappings);
    
    // Apply update check setting
    const updateCheckEnabled = updateManager.isUpdateCheckEnabled();
    document.getElementById('auto-update-check').checked = updateCheckEnabled;
    
    updatePreBreakTimingVisibility();
    updateLastCheckInfo();
    console.log('Settings applied to UI:', settings);
  } catch (error) {
    console.error('Failed to apply settings to UI:', error);
  }
}

function updatePreBreakTimingVisibility() {
  const preBreakEnabled = document.getElementById("pre-break").checked;
  UIUtils.toggleElementVisibility("pre-break-timing-card", preBreakEnabled);
}

// Update functions
function updateLastCheckInfo() {
  const lastCheckInfo = updateManager.getLastCheckInfo();
  const element = document.getElementById('last-check-info');
  
  if (element) {
    if (lastCheckInfo) {
      element.textContent = `Last checked: ${lastCheckInfo.timeAgo}`;
    } else {
      element.textContent = 'Never checked for updates';
    }
  }
}

async function handleUpdateCheck() {
  const button = document.getElementById('check-updates-btn');
  const status = document.getElementById('update-status');
  
  if (!button || !status) return;
  
  // Update UI to show checking state
  button.disabled = true;
  button.innerHTML = '<span>🔄</span>Checking...';
  status.textContent = 'Checking for updates...';
  status.style.color = 'var(--warning)';
  
  try {
    const result = await updateManager.checkForUpdatesManually();
    
    // Update last check info
    updateLastCheckInfo();
    
    if (result.hasUpdate) {
      status.textContent = `Update ${result.latestVersion} available!`;
      status.style.color = 'var(--success)';
    } else if (result.reason === 'up_to_date') {
      status.textContent = 'You have the latest version!';
      status.style.color = 'var(--success)';
    } else if (result.reason === 'error') {
      status.textContent = 'Failed to check for updates';
      status.style.color = 'var(--danger)';
    }
    
  } catch (error) {
    console.error('Manual update check failed:', error);
    status.textContent = 'Error checking for updates';
    status.style.color = 'var(--danger)';
  } finally {
    // Reset button state
    button.disabled = false;
    button.innerHTML = '<span>🔍</span>Check for Updates';
    
    // Reset status after 5 seconds
    setTimeout(() => {
      status.textContent = 'Click to check for updates';
      status.style.color = 'var(--text-muted)';
    }, 5000);
  }
}

function handleUpdateCheckToggle(enabled) {
  updateManager.setUpdateCheckEnabled(enabled);
  console.log(`Update checking ${enabled ? 'enabled' : 'disabled'}`);
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

// Input validation is now handled by UIUtils

// Debug functions - now using DebugUtils
function setQuickTimer(seconds) {
  DebugUtils.log(`Setting ${seconds} second timer`);
  DebugUtils.log(`Timer functionality available on main page`);
}

async function debugClearSettings() {
  const cleared = await DebugUtils.clearSettings(DEFAULT_SETTINGS);
  if (cleared) {
    applySettingsToUI(DEFAULT_SETTINGS);
    DebugUtils.log('UI reset to default values');
  }
}

// Initialize application
window.addEventListener("DOMContentLoaded", async () => {
  // Initialize tabs
  tabManager.init();

  // Load and apply saved settings
  const savedSettings = await loadSettings();
  applySettingsToUI(savedSettings);

  // Settings form
  const settingsInputs = [
    "auto-pause", "meeting-detect", "pre-break", "break-chime"
  ];

  settingsInputs.forEach(id => {
    const element = document.getElementById(id);
    if (element) {
      element.addEventListener('change', saveSettings);
    }
  });

  // Update check toggle
  document.getElementById("auto-update-check").addEventListener('change', (e) => {
    handleUpdateCheckToggle(e.target.checked);
  });

  // Manual update check button
  document.getElementById("check-updates-btn").addEventListener('click', handleUpdateCheck);

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

  UIUtils.setupTimeInputs(timeInputs, saveSettings);

  // Debug event listeners
  const debugButtonMappings = {
    'debug-force-break': () => DebugUtils.testForceBreak(),
    'debug-pre-break': () => DebugUtils.testPreBreak(),
    'debug-notify': () => DebugUtils.testNotify(),
    'debug-lock': () => DebugUtils.testLockScreen(),
    'debug-timer-10s': () => setQuickTimer(10),
    'debug-timer-30s': () => setQuickTimer(30),
    'debug-timer-60s': () => setQuickTimer(60),
    'debug-timer-5m': () => setQuickTimer(300),
    'debug-play-chime': () => DebugUtils.testPlayChime(),
    'debug-media-pause': () => DebugUtils.testMediaPause(),
    'debug-meeting-check': () => DebugUtils.testMeetingCheck(),
    'debug-browser-meeting-check': () => DebugUtils.testBrowserMeetingCheck(),
    'debug-meeting-notification': () => DebugUtils.testMeetingNotification(),
    'debug-autostart-check': () => DebugUtils.testAutostartCheck(),
    'debug-test-updates': () => DebugUtils.testUpdateCheck(),
    'debug-clear-settings': debugClearSettings,
    'debug-close-force': () => DebugUtils.closeWindow('force_break'),
    'debug-close-notify': () => DebugUtils.closeWindow('notify'),
    'debug-close-prebreak': () => DebugUtils.closeWindow('pre_break'),
    'debug-test-sequence': () => DebugUtils.runTestSequence(),
    'debug-help': () => DebugUtils.showHelp(),
    'debug-clear': () => DebugUtils.clear()
  };

  DebugUtils.setupDebugListeners(debugButtonMappings);

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

  DebugUtils.log('Break Reminder Pro settings page initialized successfully');
  console.log('Break Reminder Pro settings loaded successfully!');
});