const { invoke } = window.__TAURI__.core;

let timerInterval = null;
let timerSeconds = 0;
let isTimerRunning = false;

// Default settings
const defaultSettings = {
  break_minutes: 50,
  break_seconds: 0,
  break_duration_minutes: 10,
  break_duration_seconds: 0,
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

// Initialize store
async function initStore() {
  try {
    console.log('Settings store initialized (using Tauri commands)');
  } catch (error) {
    console.error('Failed to initialize store:', error);
  }
}

// Save settings to store
async function saveSettings() {
  try {
    const settings = {
      break_minutes: parseInt(document.getElementById("break-minutes").value) || 0,
      break_seconds: parseInt(document.getElementById("break-seconds").value) || 0,
      break_duration_minutes: parseInt(document.getElementById("break-duration-minutes").value) || 0,
      break_duration_seconds: parseInt(document.getElementById("break-duration-seconds").value) || 0,
      break_mode: document.querySelector('input[name="break-mode"]:checked')?.value || 'force',
      auto_pause: document.getElementById("auto-pause").checked,
      meeting_detect: document.getElementById("meeting-detect").checked,
      pre_break: document.getElementById("pre-break").checked,
      pre_break_minutes: parseInt(document.getElementById("pre-break-minutes").value) || 0,
      pre_break_seconds: parseInt(document.getElementById("pre-break-seconds").value) || 0,
      break_chime: document.getElementById("break-chime").checked,
      recurring: document.getElementById("recurring").checked,
      autostart: document.getElementById("autostart").checked,
      auto_start_timer: document.getElementById("auto-start-timer")?.checked || false
    };
    
    await invoke('save_settings', { settings });
    console.log('Settings saved:', settings);
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

// Load settings from store
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

// Apply loaded settings to UI
function applySettingsToUI(settings) {
  try {
    document.getElementById("break-minutes").value = settings.break_minutes;
    document.getElementById("break-seconds").value = settings.break_seconds;
    document.getElementById("break-duration-minutes").value = settings.break_duration_minutes;
    document.getElementById("break-duration-seconds").value = settings.break_duration_seconds;
    
    // Set break mode
    const breakModeRadio = document.querySelector(`input[name="break-mode"][value="${settings.break_mode}"]`);
    if (breakModeRadio) breakModeRadio.checked = true;
    
    // Set checkboxes
    document.getElementById("auto-pause").checked = settings.auto_pause;
    document.getElementById("meeting-detect").checked = settings.meeting_detect;
    document.getElementById("pre-break").checked = settings.pre_break;
    document.getElementById("pre-break-minutes").value = settings.pre_break_minutes || 0;
    document.getElementById("pre-break-seconds").value = settings.pre_break_seconds || 30;
    document.getElementById("break-chime").checked = settings.break_chime;
    document.getElementById("recurring").checked = settings.recurring;
    document.getElementById("autostart").checked = settings.autostart;
    
    if (document.getElementById("auto-start-timer")) {
      document.getElementById("auto-start-timer").checked = settings.auto_start_timer;
    }
    
    // Update pre-break timing visibility
    updatePreBreakTimingVisibility();
    
    console.log('Settings applied to UI:', settings);
  } catch (error) {
    console.error('Failed to apply settings to UI:', error);
  }
}

// Handle autostart toggle
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
    // Revert checkbox state on error
    document.getElementById("autostart").checked = !enabled;
  }
}

// Auto-start timer if setting is enabled
async function autoStartTimer() {
  try {
    const settings = await loadSettings();
    if (settings.auto_start_timer) {
      const breakTimerSeconds = getBreakTimerValue();
      if (breakTimerSeconds > 0) {
        console.log('Auto-starting timer with', breakTimerSeconds, 'seconds');
        startTimer(breakTimerSeconds);
        document.getElementById('timer-status').textContent = 'Timer auto-started from saved settings';
      }
    }
  } catch (error) {
    console.error('Failed to auto-start timer:', error);
  }
}

// Get pre-break timing value in seconds
function getPreBreakTimingValue() {
  const minutes = parseInt(document.getElementById("pre-break-minutes").value, 10) || 0;
  const seconds = parseInt(document.getElementById("pre-break-seconds").value, 10) || 0;
  return (minutes * 60) + seconds;
}

// Update visibility of pre-break timing inputs
function updatePreBreakTimingVisibility() {
  const preBreakEnabled = document.getElementById("pre-break").checked;
  const timingGroup = document.getElementById("pre-break-timing-group");
  
  if (timingGroup) {
    timingGroup.style.display = preBreakEnabled ? 'block' : 'none';
  }
}

function startTimer(seconds) {
  timerSeconds = seconds;
  isTimerRunning = true;
  updateTimerDisplay();
  updateUIState();
  
  if (timerInterval) clearInterval(timerInterval);
  timerInterval = setInterval(() => {
    timerSeconds--;
    updateTimerDisplay();
    if (timerSeconds <= 0) {
      clearInterval(timerInterval);
      isTimerRunning = false;
      updateUIState();
      handleBreakTime();
    }
  }, 1000);
}

function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  isTimerRunning = false;
  timerSeconds = 0;
  updateTimerDisplay();
  updateUIState();
}

function updateUIState() {
  const startBtn = document.querySelector('.btn-primary');
  const stopBtn = document.querySelector('.btn-secondary');
  const statusElement = document.getElementById('timer-status');
  
  if (isTimerRunning) {
    startBtn.disabled = true;
    startBtn.style.opacity = '0.6';
    stopBtn.disabled = false;
    stopBtn.style.opacity = '1';
    statusElement.textContent = 'Timer is running. Stay focused!';
  } else {
    startBtn.disabled = false;
    startBtn.style.opacity = '1';
    stopBtn.disabled = true;
    stopBtn.style.opacity = '0.6';
    if (timerSeconds <= 0) {
      statusElement.textContent = 'Configure your break settings below';
    } else {
      statusElement.textContent = 'Timer stopped. Ready to start again.';
    }
  }
}

async function handleBreakTime() {
  try {
    // Get selected break mode
    const breakMode = document.querySelector('input[name="break-mode"]:checked').value;
    
    // Get smart features settings
    const autoPauseMedia = document.getElementById("auto-pause").checked;
    const meetingDetect = document.getElementById("meeting-detect").checked;
    const preBreak = document.getElementById("pre-break").checked;
    
    // Check if we're in a meeting and should skip break
    if (meetingDetect) {
      const inMeeting = await invoke("is_meeting_active");
      if (inMeeting) {
        console.log("Meeting detected, postponing break...");
        document.getElementById('timer-status').textContent = 'Meeting detected - break postponed by 5 minutes';
        // Postpone break by 5 minutes
        startTimer(5 * 60);
        return;
      }
    }
    
    // Show pre-break notification if enabled
    if (preBreak) {
      const preBreakTimingSeconds = getPreBreakTimingValue();
      const preBreakTimingMs = preBreakTimingSeconds * 1000;
      
      await invoke("pre_break_notification_window");
      
      // Update status message with actual timing
      const minutes = Math.floor(preBreakTimingSeconds / 60);
      const seconds = preBreakTimingSeconds % 60;
      let timingText = '';
      if (minutes > 0) {
        timingText = `${minutes} minute${minutes > 1 ? 's' : ''}`;
        if (seconds > 0) {
          timingText += ` ${seconds} second${seconds > 1 ? 's' : ''}`;
        }
      } else {
        timingText = `${seconds} second${seconds > 1 ? 's' : ''}`;
      }
      
      document.getElementById('timer-status').textContent = `Pre-break warning shown - break starting in ${timingText}`;
      
      // Wait for configured time before showing main break
      setTimeout(async () => {
        await triggerMainBreak(breakMode, autoPauseMedia);
      }, preBreakTimingMs);
    } else {
      await triggerMainBreak(breakMode, autoPauseMedia);
    }
    
  } catch (error) {
    console.error("Error handling break time:", error);
    document.getElementById('timer-status').textContent = 'Error triggering break. Please try again.';
  }
}

async function triggerMainBreak(breakMode, autoPauseMedia) {
  try {
    // Auto-pause media if enabled
    if (autoPauseMedia) {
      await invoke("control_media", { action: "pause" });
    }
    
    // Get break duration for force break mode
    const breakDurationSeconds = getBreakDurationValue();
    
    // Trigger the appropriate break mode
    switch (breakMode) {
      case "force":
        await invoke("force_break_window", { duration: breakDurationSeconds });
        document.getElementById('timer-status').textContent = 'Force break window opened - take your break!';
        break;
      case "notify":
        await invoke("notify_window");
        document.getElementById('timer-status').textContent = 'Break notification shown';
        break;
      case "lock":
        await invoke("lock_screen");
        document.getElementById('timer-status').textContent = 'Screen locked for break time';
        break;
      default:
        console.log("Unknown break mode:", breakMode);
    }
    
    // Check if recurring is enabled
    const recurring = document.getElementById("recurring").checked;
    if (recurring) {
      // Wait for break duration then restart timer
      const breakDurationSeconds = getBreakDurationValue();
      setTimeout(() => {
        const breakTimerSeconds = getBreakTimerValue();
        startTimer(breakTimerSeconds);
      }, breakDurationSeconds * 1000);
    }
    
  } catch (error) {
    console.error("Error triggering break:", error);
    document.getElementById('timer-status').textContent = 'Error during break. Please check your settings.';
  }
}

function getBreakTimerValue() {
  const minutes = parseInt(document.getElementById("break-minutes").value, 10) || 0;
  const seconds = parseInt(document.getElementById("break-seconds").value, 10) || 0;
  return (minutes * 60) + seconds;
}

function getBreakDurationValue() {
  const minutes = parseInt(document.getElementById("break-duration-minutes").value, 10) || 0;
  const seconds = parseInt(document.getElementById("break-duration-seconds").value, 10) || 0;
  return (minutes * 60) + seconds;
}

function updateTimerDisplay() {
  const label = document.getElementById("timer-label");
  const countdown = document.getElementById("timer-countdown");
  
  if (timerSeconds > 0) {
    const min = Math.floor(timerSeconds / 60)
      .toString()
      .padStart(2, "0");
    const sec = (timerSeconds % 60).toString().padStart(2, "0");
    countdown.textContent = `${min}:${sec}`;
    label.textContent = "Next break in:";
  } else if (isTimerRunning) {
    countdown.textContent = "00:00";
    label.textContent = "Break time!";
  } else {
    countdown.textContent = "--:--";
    label.textContent = "Ready to start";
  }
}

function validateTimeInputs() {
  const breakMinutes = parseInt(document.getElementById("break-minutes").value, 10) || 0;
  const breakSeconds = parseInt(document.getElementById("break-seconds").value, 10) || 0;
  const durationMinutes = parseInt(document.getElementById("break-duration-minutes").value, 10) || 0;
  const durationSeconds = parseInt(document.getElementById("break-duration-seconds").value, 10) || 0;
  
  const totalBreakTime = (breakMinutes * 60) + breakSeconds;
  const totalDuration = (durationMinutes * 60) + durationSeconds;
  
  if (totalBreakTime <= 0) {
    return "Please enter a valid break timer (must be greater than 00:00)";
  }
  
  if (totalDuration <= 0) {
    return "Please enter a valid break duration (must be greater than 00:00)";
  }
  
  // Validate pre-break timing if pre-break is enabled
  const preBreakEnabled = document.getElementById("pre-break").checked;
  if (preBreakEnabled) {
    const preBreakMinutes = parseInt(document.getElementById("pre-break-minutes").value, 10) || 0;
    const preBreakSeconds = parseInt(document.getElementById("pre-break-seconds").value, 10) || 0;
    const totalPreBreakTime = (preBreakMinutes * 60) + preBreakSeconds;
    
    if (totalPreBreakTime <= 0) {
      return "Please enter a valid pre-break warning time (must be greater than 00:00 when enabled)";
    }
    
    if (totalPreBreakTime >= totalBreakTime) {
      return "Pre-break warning time must be less than the break timer duration";
    }
  }
  
  return null; // No errors
}

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

// Add auto-start timer toggle to the UI
function addAutoStartTimerToggle() {
  // Find the Additional Options card by looking for the card that contains "Additional Options" text
  const additionalOptionsCards = Array.from(document.querySelectorAll('.settings-card'));
  const additionalOptionsCard = additionalOptionsCards.find(card => 
    card.querySelector('.card-title')?.textContent?.includes('Additional Options')
  );
  
  if (!additionalOptionsCard) {
    console.error('Additional Options card not found');
    return;
  }
  
  const toggleGroup = additionalOptionsCard.querySelector('.toggle-group');
  if (!toggleGroup) {
    console.error('Toggle group not found in Additional Options card');
    return;
  }
  
  // Check if auto-start timer toggle already exists
  if (document.getElementById('auto-start-timer')) {
    console.log('Auto-start timer toggle already exists, skipping');
    return;
  }
  
  const toggleHtml = `
    <label class="toggle-option">
      <input type="checkbox" id="auto-start-timer" />
      <span class="toggle-switch"></span>
      <div class="toggle-content">
        <span class="toggle-title">Auto-Start Timer on Launch</span>
        <span class="toggle-desc">Automatically start timer when app opens</span>
      </div>
    </label>
  `;
  
  toggleGroup.insertAdjacentHTML('beforeend', toggleHtml);
  
  // Add event listener for the new toggle
  document.getElementById("auto-start-timer").addEventListener('change', saveSettings);
}

// Debug functionality
let debugVisible = true;

function debugLog(message) {
  console.log(`[DEBUG] ${message}`); // Always log to console first
  
  const output = document.getElementById('debug-output');
  if (output) {
    const timestamp = new Date().toLocaleTimeString();
    output.textContent += `[${timestamp}] ${message}\n`;
    output.scrollTop = output.scrollHeight;
  } else {
    console.warn('Debug output element not found!');
  }
}

function debugClear() {
  const output = document.getElementById('debug-output');
  if (output) {
    output.textContent = 'Debug output cleared...\n';
  }
  debugLog('Debug console cleared');
}

function debugHelp() {
  debugLog('=== BREAK REMINDER PRO DEBUG HELP ===');
  debugLog('');
  debugLog('BREAK TESTING:');
  debugLog('  🖥️ Force Break Now - Opens force break window immediately');
  debugLog('  ⏰ Pre-Break Warning - Shows pre-break notification');
  debugLog('  🔔 Notify Window - Opens notification window');
  debugLog('  🔒 Lock Screen - Locks the screen (Windows only)');
  debugLog('');
  debugLog('QUICK TIMERS:');
  debugLog('  ⏱️ 10/30/60 Second Timer - Sets short timers for testing');
  debugLog('  ⏱️ 5 Minute Timer - Sets 5-minute timer');
  debugLog('');
  debugLog('FEATURE TESTING:');
  debugLog('  ⏸️ Test Media Pause - Tests media control functionality');
  debugLog('  👥 Check Meeting Status - Tests meeting detection');
  debugLog('  🚀 Check Autostart Status - Tests Windows autostart');
  debugLog('  🗑️ Clear All Settings - Resets app to default state');
  debugLog('');
  debugLog('WINDOW CONTROLS:');
  debugLog('  ❌ Close Force/Notify/Pre-Break - Closes specific windows');
  debugLog('  🧪 Test Full Sequence - Runs comprehensive test');
  debugLog('');
  debugLog('KEYBOARD SHORTCUTS (Ctrl+Shift+):');
  debugLog('  F - Force Break    P - Pre-Break    N - Notify');
  debugLog('  T - 10s Timer      M - Meeting      D - Toggle Debug');
  debugLog('  R - Test Sequence  H - This Help');
  debugLog('');
  debugLog('Type "help" or press Ctrl+Shift+H for this help again');
  debugLog('=====================================');
}

function toggleDebugSection() {
  const debugContent = document.getElementById('debug-content');
  const toggleBtn = document.getElementById('toggle-debug');
  
  if (debugVisible) {
    debugContent.style.display = 'none';
    toggleBtn.textContent = 'Show Debug';
    debugVisible = false;
  } else {
    debugContent.style.display = 'block';
    toggleBtn.textContent = 'Hide Debug';
    debugVisible = true;
  }
}

// Debug timer functions
function setQuickTimer(seconds) {
  debugLog(`Setting ${seconds} second timer`);
  if (isTimerRunning) {
    stopTimer();
    debugLog('Stopped existing timer');
  }
  startTimer(seconds);
  debugLog(`Timer started for ${seconds} seconds`);
}

// Debug break functions
async function debugForceBreak() {
  try {
    debugLog('Triggering force break window...');
    const breakDurationSeconds = getBreakDurationValue();
    debugLog(`Duration: ${breakDurationSeconds} seconds`);
    await invoke("force_break_window", { duration: breakDurationSeconds });
    debugLog(`✅ Force break window created (fullscreen, ${breakDurationSeconds}s duration)`);
    debugLog('Window should appear black with countdown. Use Escape or math problem to exit.');
  } catch (error) {
    debugLog(`❌ Error triggering force break: ${error}`);
  }
}

async function debugPreBreak() {
  try {
    debugLog('Triggering pre-break notification...');
    await invoke("pre_break_notification_window");
    debugLog('✅ Pre-break notification window created (350x180px, yellow)');
    debugLog('Window should show "Break in:" countdown.');
  } catch (error) {
    debugLog(`❌ Error triggering pre-break: ${error}`);
  }
}

async function debugNotify() {
  try {
    debugLog('Triggering notify window...');
    await invoke("notify_window");
    debugLog('✅ Notify window created (400x200px, blue)');
    debugLog('Window should show "Break starting soon!" message.');
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

// Debug feature functions
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

async function debugAutostartCheck() {
  try {
    debugLog('Checking autostart status...');
    const isEnabled = await invoke("is_autostart_enabled");
    debugLog(`Autostart status: ${isEnabled ? 'Enabled' : 'Disabled'}`);
  } catch (error) {
    debugLog(`Error checking autostart: ${error}`);
  }
}

// Debug window controls
async function debugCloseWindow(windowLabel) {
  try {
    debugLog(`Closing ${windowLabel} window...`);
    await invoke("close_window", { label: windowLabel });
    debugLog(`${windowLabel} window closed`);
  } catch (error) {
    debugLog(`Error closing ${windowLabel} window: ${error}`);
  }
}

async function debugShowMainWindow() {
    try {
        debugLog('Requesting to show main window...');
        await invoke('show_index_window');
        debugLog('✅ Main window shown successfully');
    } catch (error) {
        debugLog(`❌ Error showing main window: ${error}`);
    }
}

// Debug settings
async function debugClearSettings() {
  try {
    debugLog('Clearing all settings...');
    if (confirm('Are you sure you want to clear all settings? This cannot be undone.')) {
      // Save default settings (effectively clearing current settings)
      await invoke('save_settings', { settings: defaultSettings });
      debugLog('Settings cleared and reset to defaults');
      
      // Reset UI to defaults
      applySettingsToUI(defaultSettings);
      debugLog('UI reset to default values');
      
      // Stop any running timer
      if (isTimerRunning) {
        stopTimer();
        debugLog('Timer stopped');
      }
    } else {
      debugLog('Settings clear cancelled by user');
    }
  } catch (error) {
    debugLog(`Error clearing settings: ${error}`);
  }
}

// Test full sequence
// Debug test window
async function debugTestWindow() {
  try {
    debugLog('Creating debug test window...');
    debugLog('This will use notify.html but styled as a test window');
    debugLog('Look for: Purple background, "🧪 Test Window Working!" text');
    await invoke("debug_test_window");
    debugLog('✅ Debug test window created successfully');
    debugLog('Window should show blue->purple background with close button');
    debugLog('Press Escape or click ✕ Close to close the window');
  } catch (error) {
    debugLog(`❌ Error creating debug test window: ${error}`);
  }
}

async function debugTestSequence() {
  try {
    debugLog('=== STARTING FULL TEST SEQUENCE ===');
    
    // Test 0: Simple test window
    debugLog('0. Testing basic window creation...');
    await debugTestWindow();
    
    // Test 1: Settings check
    debugLog('1. Testing settings persistence...');
    await saveSettings();
    const loadedSettings = await loadSettings();
    debugLog(`Settings loaded successfully`);
    
    // Test 2: Meeting detection
    debugLog('2. Testing meeting detection...');
    await debugMeetingCheck();
    
    // Test 3: Autostart status
    debugLog('3. Testing autostart status...');
    await debugAutostartCheck();
    
    // Test 4: Media control
    debugLog('4. Testing media control...');
    await debugMediaPause();
    
    // Test 5: Window creation (with delay)
    debugLog('5. Testing notification windows...');
    debugLog('Creating pre-break window...');
    await debugPreBreak();
    
    await new Promise(resolve => setTimeout(resolve, 3000));
    debugLog('Closing pre-break window...');
    await debugCloseWindow('pre_break');
    
    debugLog('Creating notify window...');
    await debugNotify();
    
    await new Promise(resolve => setTimeout(resolve, 3000));
    debugLog('Closing notify window...');
    await debugCloseWindow('notify');
    
    // Test 6: Quick timer
    debugLog('6. Testing quick timer (10 seconds)...');
    setQuickTimer(10);
    
    debugLog('=== FULL TEST SEQUENCE COMPLETED ===');
    debugLog('Check above for any errors. All features tested successfully!');
    
  } catch (error) {
    debugLog(`Error in test sequence: ${error}`);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  const form = document.getElementById("settings-form");
  const stopButton = document.getElementById("stop-timer");
  
  // Initialize store and load settings
  await initStore();
  
  // Add auto-start timer toggle to UI
  addAutoStartTimerToggle();
  
  // Load and apply saved settings
  const savedSettings = await loadSettings();
  applySettingsToUI(savedSettings);
  
  // Initialize UI state
  updateUIState();
  updateTimerDisplay();
  
  // Auto-start timer if enabled (with a small delay to ensure UI is ready)
  setTimeout(autoStartTimer, 1000);
  
  // Handle form submission
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    
    if (isTimerRunning) {
      return; // Prevent starting multiple timers
    }
    
    // Validate inputs
    const validationError = validateTimeInputs();
    if (validationError) {
      alert(validationError);
      return;
    }
    
    // Save settings before starting timer
    await saveSettings();
    
    // Get break timer value in seconds
    const breakTimerSeconds = getBreakTimerValue();
    
    // Start timer
    startTimer(breakTimerSeconds);
  });
  
  // Handle stop button
  stopButton.addEventListener("click", (e) => {
    e.preventDefault();
    stopTimer();
  });
  
  // Handle autostart toggle
  document.getElementById("autostart").addEventListener('change', async (e) => {
    await handleAutostartToggle(e.target.checked);
    await saveSettings();
  });
  
  // Add event listeners to save settings when inputs change
  const settingsInputs = [
    "break-minutes", "break-seconds", "break-duration-minutes", "break-duration-seconds",
    "pre-break-minutes", "pre-break-seconds",
    "auto-pause", "meeting-detect", "pre-break", "break-chime", "recurring"
  ];
  
  settingsInputs.forEach(id => {
    const element = document.getElementById(id);
    if (element) {
      element.addEventListener('change', saveSettings);
    }
  });
  
  // Add special event listener for pre-break toggle to show/hide timing inputs
  document.getElementById("pre-break").addEventListener('change', () => {
    updatePreBreakTimingVisibility();
    saveSettings();
  });
  
  // Add event listeners for radio buttons
  document.querySelectorAll('input[name="break-mode"]').forEach(radio => {
    radio.addEventListener('change', saveSettings);
  });
  
  // Add input validation for time fields
  const timeInputs = [
    { element: document.getElementById("break-minutes"), max: 180 },
    { element: document.getElementById("break-seconds"), max: 59 },
    { element: document.getElementById("break-duration-minutes"), max: 60 },
    { element: document.getElementById("break-duration-seconds"), max: 59 },
    { element: document.getElementById("pre-break-minutes"), max: 10 },
    { element: document.getElementById("pre-break-seconds"), max: 59 }
  ];
  
  timeInputs.forEach(({ element, max }) => {
    element.addEventListener('blur', () => {
      formatTimeInput(element, max);
      saveSettings(); // Save settings after validation
    });
    element.addEventListener('input', () => {
      if (element.value.length > 3) {
        element.value = element.value.slice(0, 3);
      }
    });
  });
  
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

  // Debug section event listeners
  debugLog('Debug panel initialized');
  
  // Toggle debug visibility
  document.getElementById('toggle-debug').addEventListener('click', toggleDebugSection);
  
  // Break testing buttons
  document.getElementById('debug-force-break').addEventListener('click', debugForceBreak);
  document.getElementById('debug-pre-break').addEventListener('click', debugPreBreak);
  document.getElementById('debug-notify').addEventListener('click', debugNotify);
  document.getElementById('debug-lock').addEventListener('click', debugLockScreen);
  
  // Quick timer buttons
  document.getElementById('debug-timer-10s').addEventListener('click', () => setQuickTimer(10));
  document.getElementById('debug-timer-30s').addEventListener('click', () => setQuickTimer(30));
  document.getElementById('debug-timer-60s').addEventListener('click', () => setQuickTimer(60));
  document.getElementById('debug-timer-5m').addEventListener('click', () => setQuickTimer(300));
  
  // Feature testing buttons
  document.getElementById('debug-media-pause').addEventListener('click', debugMediaPause);
  document.getElementById('debug-meeting-check').addEventListener('click', debugMeetingCheck);
  document.getElementById('debug-autostart-check').addEventListener('click', debugAutostartCheck);
  document.getElementById('debug-clear-settings').addEventListener('click', debugClearSettings);
  
  // Window control buttons
  document.getElementById('debug-close-force').addEventListener('click', () => debugCloseWindow('force_break'));
  document.getElementById('debug-close-notify').addEventListener('click', () => debugCloseWindow('notify'));
  document.getElementById('debug-close-prebreak').addEventListener('click', () => debugCloseWindow('pre_break'));
  document.getElementById('debug-test-window').addEventListener('click', debugTestWindow);
  document.getElementById('debug-show-main').addEventListener('click', debugShowMainWindow);
  document.getElementById('debug-test-sequence').addEventListener('click', debugTestSequence);
  
  // Debug console controls
  document.getElementById('debug-help').addEventListener('click', debugHelp);
  document.getElementById('debug-clear').addEventListener('click', debugClear);
  
  debugLog('All debug event listeners registered');
  
  // Keyboard shortcuts for debug functions
  document.addEventListener('keydown', (e) => {
    // Only activate shortcuts when Ctrl+Shift is held (to avoid conflicts)
    if (e.ctrlKey && e.shiftKey) {
      switch (e.key) {
        case 'F':
          e.preventDefault();
          debugForceBreak();
          debugLog('Keyboard shortcut: Force Break (Ctrl+Shift+F)');
          break;
        case 'P':
          e.preventDefault();
          debugPreBreak();
          debugLog('Keyboard shortcut: Pre-Break (Ctrl+Shift+P)');
          break;
        case 'N':
          e.preventDefault();
          debugNotify();
          debugLog('Keyboard shortcut: Notify (Ctrl+Shift+N)');
          break;
        case 'T':
          e.preventDefault();
          setQuickTimer(10);
          debugLog('Keyboard shortcut: 10s Timer (Ctrl+Shift+T)');
          break;
        case 'M':
          e.preventDefault();
          debugMeetingCheck();
          debugLog('Keyboard shortcut: Meeting Check (Ctrl+Shift+M)');
          break;
        case 'D':
          e.preventDefault();
          toggleDebugSection();
          debugLog('Keyboard shortcut: Toggle Debug (Ctrl+Shift+D)');
          break;
                 case 'R':
           e.preventDefault();
           debugTestSequence();
           debugLog('Keyboard shortcut: Test Sequence (Ctrl+Shift+R)');
           break;
         case 'H':
           e.preventDefault();
           debugHelp();
           debugLog('Keyboard shortcut: Help (Ctrl+Shift+H)');
           break;
      }
    }
  });
  
  // Show keyboard shortcuts in debug log
  debugLog('Keyboard shortcuts enabled:');
  debugLog('  Ctrl+Shift+F - Force Break');
  debugLog('  Ctrl+Shift+P - Pre-Break');
  debugLog('  Ctrl+Shift+N - Notify');
  debugLog('  Ctrl+Shift+T - 10s Timer');
  debugLog('  Ctrl+Shift+M - Meeting Check');
  debugLog('  Ctrl+Shift+D - Toggle Debug');
  debugLog('  Ctrl+Shift+R - Test Sequence');
  debugLog('  Ctrl+Shift+H - Show Help');
});
