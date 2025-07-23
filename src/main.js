const { invoke } = window.__TAURI__.core;

// Timer state
let timerInterval = null;
let timerSeconds = 0;
let isTimerRunning = false;
let isTimerPaused = false;
let startTime = null;
let pausedTime = 0;
let preBreakTriggered = false;
let currentTimerSettings = null; // Cache settings for current timer session
let recurringTimeout = null; // Track recurring timer timeout

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

// Settings management
async function saveSettings() {
  try {
    const breakDurationMinutes = parseInt(document.getElementById("break-duration-minutes").value) || 0;
    const breakDurationSeconds = parseInt(document.getElementById("break-duration-seconds").value) || 0;
    
    // Load existing settings first
    const existingSettings = await loadSettings();
    
    // Only update the settings that are managed on this page
    const updatedSettings = {
      ...existingSettings,
      break_minutes: parseInt(document.getElementById("break-minutes").value) || 0,
      break_seconds: parseInt(document.getElementById("break-seconds").value) || 0,
      break_duration_minutes: breakDurationMinutes,
      break_duration_seconds: breakDurationSeconds,
      break_mode: document.querySelector('input[name="break-mode"]:checked')?.value || 'force',
      recurring: document.getElementById("recurring").checked
    };

    await invoke('save_settings', { settings: updatedSettings });
    console.log('Settings saved:', updatedSettings);
    console.log(`Break duration saved as: ${breakDurationMinutes}m ${breakDurationSeconds}s`);
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

async function loadSettings() {
  try {
    const settings = await invoke('load_settings');
    if (settings) {
      console.log('Settings loaded:', settings);
      // Merge with defaults to ensure all properties exist
      return { ...defaultSettings, ...settings };
    }
    return defaultSettings;
  } catch (error) {
    console.error('Failed to load settings:', error);
    return defaultSettings;
  }
}

function applySettingsToUI(settings) {
  try {
    document.getElementById("break-minutes").value = settings.break_minutes;
    document.getElementById("break-seconds").value = settings.break_seconds;
    document.getElementById("break-duration-minutes").value = settings.break_duration_minutes;
    document.getElementById("break-duration-seconds").value = settings.break_duration_seconds;

    // Set break mode
    const breakModeRadio = document.querySelector(`input[name="break-mode"][value="${settings.break_mode}"]`);
    if (breakModeRadio) breakModeRadio.checked = true;

    // Set recurring toggle
    document.getElementById("recurring").checked = settings.recurring;

    console.log('Settings applied to UI:', settings);
  } catch (error) {
    console.error('Failed to apply settings to UI:', error);
  }
}

// Timer functions
async function startTimer(seconds) {
  console.log(`🚀 Starting timer with ${seconds} seconds (${Math.floor(seconds/60)}:${seconds%60})`);
  
  // Load settings once at the start of the timer
  currentTimerSettings = await loadSettings();
  console.log('⚙️ Settings loaded for timer session:', currentTimerSettings);
  
  timerSeconds = seconds;
  isTimerRunning = true;
  startTime = Date.now();
  preBreakTriggered = false; // Reset pre-break flag
  updateTimerDisplay();
  updateTimerControls();
  updatePanelVisibility();

  if (timerInterval) clearInterval(timerInterval);
  timerInterval = setInterval(async () => {
    timerSeconds--;
    updateTimerDisplay();
    updateRunningTime();
    
    // Debug logging for last 30 seconds
    if (timerSeconds <= 30) {
      console.log(`⏰ Timer countdown: ${timerSeconds} seconds remaining`);
    }
    
    // Check for pre-break warning trigger
    await checkPreBreakTrigger();
    
    if (timerSeconds <= 0) {
      console.log('⏰ Timer reached zero! Triggering break...');
      clearInterval(timerInterval);
      isTimerRunning = false;
      updateTimerControls();
      updatePanelVisibility();
      handleBreakTime();
    }
  }, 1000);
}

function pauseTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  isTimerPaused = true;
  pausedTime += Date.now() - startTime;
  updateTimerControls();
}

function resumeTimer() {
  if (!isTimerPaused) return;
  
  isTimerPaused = false;
  startTime = Date.now();
  
  timerInterval = setInterval(async () => {
    timerSeconds--;
    updateTimerDisplay();
    updateRunningTime();
    
    // Debug logging for last 30 seconds
    if (timerSeconds <= 30) {
      console.log(`⏰ Timer countdown (resumed): ${timerSeconds} seconds remaining`);
    }
    
    // Check for pre-break warning trigger
    await checkPreBreakTrigger();
    
    if (timerSeconds <= 0) {
      console.log('⏰ Timer reached zero (resumed)! Triggering break...');
      clearInterval(timerInterval);
      isTimerRunning = false;
      isTimerPaused = false;
      updateTimerControls();
      updatePanelVisibility();
      handleBreakTime();
    }
  }, 1000);
  
  updateTimerControls();
}

function stopTimer() {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  if (recurringTimeout) {
    clearTimeout(recurringTimeout);
    recurringTimeout = null;
  }
  isTimerRunning = false;
  isTimerPaused = false;
  timerSeconds = 0;
  startTime = null;
  pausedTime = 0;
  currentTimerSettings = null; // Clear cached settings
  updateTimerDisplay();
  updateTimerControls();
  updatePanelVisibility();
}

function updateTimerDisplay() {
  const countdown = document.getElementById("timer-countdown");
  const label = document.getElementById("timer-label");

  if (timerSeconds > 0) {
    const min = Math.floor(timerSeconds / 60).toString().padStart(2, "0");
    const sec = (timerSeconds % 60).toString().padStart(2, "0");
    countdown.textContent = `${min}:${sec}`;
    label.textContent = "Next break in";

    // Add pulse effect when less than 1 minute
    if (timerSeconds <= 60) {
      countdown.classList.add('pulse');
    } else {
      countdown.classList.remove('pulse');
    }
  } else if (isTimerRunning) {
    countdown.textContent = "00:00";
    label.textContent = "Break time!";
    countdown.classList.add('pulse');
  } else {
    countdown.textContent = "--:--";
    label.textContent = "Ready to start";
    countdown.classList.remove('pulse');
  }
}

function updateTimerControls() {
  const startBtn = document.getElementById('start-timer');
  const stopBtn = document.getElementById('stop-timer');
  const pauseBtn = document.getElementById('pause-timer');
  const statusElement = document.getElementById('timer-status');

  if (isTimerRunning) {
    if (startBtn) startBtn.disabled = true;
    if (stopBtn) stopBtn.disabled = false;
    
    if (pauseBtn) {
      if (isTimerPaused) {
        pauseBtn.innerHTML = '<span>▶️</span>Resume';
        pauseBtn.disabled = false;
      } else {
        pauseBtn.innerHTML = '<span>⏸️</span>Pause';
        pauseBtn.disabled = false;
      }
    }
    
    if (statusElement) {
      if (isTimerPaused) {
        statusElement.textContent = 'Timer paused. Click Resume to continue.';
      } else {
        statusElement.textContent = 'Timer is running. Stay focused and productive!';
      }
    }
  } else {
    if (startBtn) startBtn.disabled = false;
    if (stopBtn) stopBtn.disabled = true;
    if (pauseBtn) pauseBtn.disabled = true;
    
    if (statusElement) {
      if (timerSeconds <= 0) {
        statusElement.textContent = 'Configure your break settings below';
      } else {
        statusElement.textContent = 'Timer stopped. Ready to start again.';
      }
    }
  }
}

function updatePanelVisibility() {
  const settingsPanel = document.getElementById('settings-panel');
  const runningPanel = document.getElementById('running-panel');
  const timerHero = document.getElementById('timer-hero');

  if (isTimerRunning) {
    settingsPanel.classList.add('hidden');
    runningPanel.classList.remove('hidden');
    timerHero.classList.remove('hidden');
    updateInfoPanel();
  } else {
    settingsPanel.classList.remove('hidden');
    runningPanel.classList.add('hidden');
    timerHero.classList.add('hidden');
  }
}

function updateRunningTime() {
  if (!startTime && !pausedTime) return;

  let totalElapsed;
  if (isTimerPaused) {
    totalElapsed = Math.floor(pausedTime / 1000);
  } else {
    totalElapsed = Math.floor((pausedTime + (Date.now() - startTime)) / 1000);
  }
  
  const min = Math.floor(totalElapsed / 60).toString().padStart(2, "0");
  const sec = (totalElapsed % 60).toString().padStart(2, "0");

  const runningTimeElement = document.getElementById('running-time');
  if (runningTimeElement) {
    runningTimeElement.textContent = `${min}:${sec}`;
  }
}

function updateInfoPanel() {
  // Update break duration display
  const durationMinutes = parseInt(document.getElementById("break-duration-minutes").value) || 0;
  const durationSeconds = parseInt(document.getElementById("break-duration-seconds").value) || 0;
  const durationMin = durationMinutes.toString().padStart(2, "0");
  const durationSec = durationSeconds.toString().padStart(2, "0");

  const breakDurationDisplay = document.getElementById('break-duration-display');
  if (breakDurationDisplay) {
    breakDurationDisplay.textContent = `${durationMin}:${durationSec}`;
  }

  // Update break mode display
  const breakMode = document.querySelector('input[name="break-mode"]:checked')?.value || 'force';
  const modeTextCompact = document.getElementById('mode-text-compact');
  const modeIconCompact = document.getElementById('mode-icon-compact');

  if (modeTextCompact && modeIconCompact) {
    const modeInfo = {
      'force': { name: 'Force Break', icon: '🔒' },
      'notify': { name: 'Notify Only', icon: '🔔' },
      'lock': { name: 'Lock Screen', icon: '🛡️' }
    };
    const info = modeInfo[breakMode] || modeInfo['force'];
    modeTextCompact.textContent = info.name;
    modeIconCompact.textContent = info.icon;
  }

  // Update recurring status
  const recurring = document.getElementById("recurring").checked;
  const toggleStatusDisplay = document.getElementById('toggle-status-display');
  if (toggleStatusDisplay) {
    toggleStatusDisplay.textContent = recurring ? 'On' : 'Off';
    toggleStatusDisplay.className = recurring ? 'toggle-status status-on' : 'toggle-status status-off';
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
  const totalSeconds = (minutes * 60) + seconds;
  console.log(`Break duration: ${minutes}m ${seconds}s = ${totalSeconds} total seconds`);
  return totalSeconds;
}

// Pre-break trigger check
async function checkPreBreakTrigger() {
  if (preBreakTriggered) return; // Already triggered
  if (!currentTimerSettings) return; // No settings loaded
  
  try {
    if (!currentTimerSettings.pre_break) return; // Pre-break not enabled
    
    const preBreakTimingSeconds = (currentTimerSettings.pre_break_minutes * 60) + currentTimerSettings.pre_break_seconds;
    
    // Trigger pre-break when remaining time equals pre-break timing
    if (timerSeconds === preBreakTimingSeconds) {
      console.log(`🚨 PRE-BREAK TRIGGER! ${preBreakTimingSeconds} seconds remaining (should show at bottom center)`);
      preBreakTriggered = true;
      
      await invoke("pre_break_notification_window");
      
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
    }
  } catch (error) {
    console.error('Error checking pre-break trigger:', error);
  }
}

// Break handling
async function handleBreakTime() {
  console.log('🚨 BREAK TIME TRIGGERED! Starting break handling...');
  try {
    // Save current settings first to ensure we use the latest values
    await saveSettings();
    
    // Small delay to ensure settings are saved
    await new Promise(resolve => setTimeout(resolve, 100));
    
    // Use cached settings from timer session
    const settings = currentTimerSettings || await loadSettings();
    const breakMode = document.querySelector('input[name="break-mode"]:checked').value;

    console.log('🔧 Break time! Using cached settings:', settings);
    console.log('🔧 Pre-break enabled:', settings.pre_break);
    console.log('🔧 Pre-break timing:', settings.pre_break_minutes, 'minutes', settings.pre_break_seconds, 'seconds');
    console.log('🔧 Break mode:', breakMode);

    // Double-check that settings match UI
    const uiDurationMinutes = parseInt(document.getElementById("break-duration-minutes").value) || 0;
    const uiDurationSeconds = parseInt(document.getElementById("break-duration-seconds").value) || 0;
    console.log('UI values:', { minutes: uiDurationMinutes, seconds: uiDurationSeconds });
    
    if (settings.break_duration_minutes !== uiDurationMinutes || settings.break_duration_seconds !== uiDurationSeconds) {
      console.warn('Settings mismatch! Saving again...');
      await saveSettings();
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    // Check if we're in a meeting and should skip break
    if (settings.meeting_detect) {
      const inMeeting = await invoke("is_meeting_active");
      if (inMeeting) {
        console.log("Meeting detected, postponing break...");
        document.getElementById('timer-status').textContent = 'Meeting detected - break postponed by 10 minutes';
        
        // Show meeting detected notification
        try {
          await invoke("meeting_detected_notification");
        } catch (error) {
          console.error("Failed to show meeting notification:", error);
        }
        
        await startTimer(10 * 60);
        return;
      }
    }

    // Trigger the main break immediately (pre-break was already shown during countdown)
    await triggerMainBreak(breakMode, settings);

  } catch (error) {
    console.error("Error handling break time:", error);
    document.getElementById('timer-status').textContent = 'Error triggering break. Please try again.';
  }
}

async function triggerMainBreak(breakMode, settings) {
  try {
    if (settings.auto_pause) {
      await invoke("control_media", { action: "pause" });
    }

    const breakDurationSeconds = getBreakDurationValue();
    console.log(`Triggering break with duration: ${breakDurationSeconds} seconds`);
    console.log(`Break mode: ${breakMode}`);
    console.log(`Settings break duration: ${settings.break_duration_minutes}m ${settings.break_duration_seconds}s`);

    switch (breakMode) {
      case "force":
        console.log(`Calling force_break_window with duration: ${breakDurationSeconds}`);
        await invoke("force_break_window", { duration: breakDurationSeconds });
        document.getElementById('timer-status').textContent = `Force break window opened - ${breakDurationSeconds}s break!`;
        break;
      case "notify":
        console.log(`Calling notify_window with duration: ${breakDurationSeconds}`);
        await invoke("notify_window", { duration: breakDurationSeconds });
        document.getElementById('timer-status').textContent = `Break notification shown - ${breakDurationSeconds}s break!`;
        break;
      case "lock":
        await invoke("lock_screen");
        document.getElementById('timer-status').textContent = 'Screen locked for break time';
        break;
    }

    if (settings.recurring) {
      console.log(`🔄 Setting up recurring timer for ${breakDurationSeconds} seconds`);
      recurringTimeout = setTimeout(async () => {
        console.log('🔄 Recurring timer triggered - starting next timer');
        const breakTimerSeconds = getBreakTimerValue();
        await startTimer(breakTimerSeconds);
      }, breakDurationSeconds * 1000);
    }

  } catch (error) {
    console.error("Error triggering break:", error);
    document.getElementById('timer-status').textContent = 'Error during break. Please check your settings.';
  }
}

// Handle break skip (when user clicks skip break button)
function handleBreakSkipped() {
  console.log('⏭️ Break was skipped by user');
  
  // Clear any existing timers
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  if (recurringTimeout) {
    clearTimeout(recurringTimeout);
    recurringTimeout = null;
  }
  
  // Store recurring setting before clearing timer state
  const wasRecurring = currentTimerSettings && currentTimerSettings.recurring;
  console.log('DEBUG: currentTimerSettings:', currentTimerSettings);
  console.log('DEBUG: wasRecurring:', wasRecurring);
  
  // Update timer state
  isTimerRunning = false;
  isTimerPaused = false;
  timerSeconds = 0;
  
  // If recurring is enabled, start the next timer immediately
  if (wasRecurring) {
    console.log('🔄 Starting next timer immediately due to break skip');
    
    const statusElement = document.getElementById('timer-status');
    if (statusElement) {
      statusElement.textContent = '⏭️ Break skipped - Starting next timer session...';
    }
    
    // Start next timer immediately
    setTimeout(async () => {
      const breakTimerSeconds = getBreakTimerValue();
      await startTimer(breakTimerSeconds);
    }, 500);
  } else {
    // If recurring is not enabled, just stop the timer
    const statusElement = document.getElementById('timer-status');
    if (statusElement) {
      statusElement.textContent = '⏭️ Break skipped - Timer session finished';
    }
    
    // Clear settings and reset UI state
    currentTimerSettings = null;
    updateTimerDisplay();
    updateTimerControls();
    updatePanelVisibility();
  }
}

// Handle early return from break (when user closes break window early)
function handleEarlyBreakReturn() {
  console.log('🏃 User returned early from break');
  console.log('🔧 Current timer settings:', currentTimerSettings);
  console.log('🔧 Recurring enabled:', currentTimerSettings?.recurring);
  console.log('🔧 Timer currently running:', isTimerRunning);
  
  // Stop any currently running timer first
  if (isTimerRunning) {
    console.log('⏹️ Stopping current timer');
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    isTimerRunning = false;
    isTimerPaused = false;
  }
  
  // Clear the existing recurring timeout since user returned early
  if (recurringTimeout) {
    console.log('⏰ Clearing existing recurring timeout');
    clearTimeout(recurringTimeout);
    recurringTimeout = null;
  }
  
  // If recurring is enabled, start the next timer immediately
  if (currentTimerSettings && currentTimerSettings.recurring) {
    console.log('🔄 Starting next timer immediately due to early return');
    
    // Show immediate feedback to user
    const statusElement = document.getElementById('timer-status');
    if (statusElement) {
      statusElement.textContent = '🔄 Break ended early - Starting next timer session...';
    }
    
    // Start immediately with minimal delay
    setTimeout(async () => {
      const breakTimerSeconds = getBreakTimerValue();
      console.log('🚀 Starting new timer with', breakTimerSeconds, 'seconds');
      await startTimer(breakTimerSeconds);
    }, 500); // Minimal delay just to show the message
  } else {
    // If recurring is not enabled, show that break session is complete
    console.log('✅ Recurring not enabled, finishing session');
    const statusElement = document.getElementById('timer-status');
    if (statusElement) {
      statusElement.textContent = '✅ Break completed early - Timer session finished';
    }
    
    // Clear settings and reset UI state
    currentTimerSettings = null;
    updateTimerDisplay();
    updateTimerControls();
    updatePanelVisibility();
  }
}

// Input validation
function validateTimeInputs() {
  const totalBreakTime = getBreakTimerValue();
  const totalDuration = getBreakDurationValue();

  if (totalBreakTime <= 0) {
    return "Please enter a valid break timer (must be greater than 00:00)";
  }

  if (totalDuration <= 0) {
    return "Please enter a valid break duration (must be greater than 00:00)";
  }

  return null;
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

// Initialize application
window.addEventListener("DOMContentLoaded", async () => {
  // Load and apply saved settings
  const savedSettings = await loadSettings();
  applySettingsToUI(savedSettings);

  // Initialize UI state
  updateTimerDisplay();
  updateTimerControls();
  updatePanelVisibility();
  
  // Debug: Log current values
  console.log('Current break duration values:', {
    minutes: document.getElementById("break-duration-minutes").value,
    seconds: document.getElementById("break-duration-seconds").value,
    total: getBreakDurationValue()
  });

  // Timer controls
  document.getElementById('start-timer').addEventListener('click', async () => {
    console.log('🎯 Start timer button clicked');
    if (isTimerRunning) {
      console.log('⚠️ Timer already running, ignoring click');
      return;
    }

    const validationError = validateTimeInputs();
    if (validationError) {
      console.log('❌ Validation error:', validationError);
      alert(validationError);
      return;
    }

    console.log('💾 Saving settings before starting timer...');
    await saveSettings();
    const breakTimerSeconds = getBreakTimerValue();
    console.log('⏰ Break timer value:', breakTimerSeconds, 'seconds');
    await startTimer(breakTimerSeconds);
  });

  document.getElementById('stop-timer').addEventListener('click', () => {
    stopTimer();
  });

  // Settings form
  const timerForm = document.getElementById('timer-form');
  timerForm.addEventListener('change', saveSettings);

  // Recurring toggle
  document.getElementById("recurring").addEventListener('change', saveSettings);

  // Time input validation
  const timeInputs = [
    { element: document.getElementById("break-minutes"), max: 180 },
    { element: document.getElementById("break-seconds"), max: 59 },
    { element: document.getElementById("break-duration-minutes"), max: 60 },
    { element: document.getElementById("break-duration-seconds"), max: 59 }
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

  // Add pause timer functionality
  const pauseBtn = document.getElementById('pause-timer');
  if (pauseBtn) {
    pauseBtn.addEventListener('click', () => {
      if (isTimerPaused) {
        resumeTimer();
      } else {
        pauseTimer();
      }
    });
  }
  // Make functions available globally for Rust to call
  window.handleBreakSkipped = handleBreakSkipped;
  window.handleEarlyBreakReturn = handleEarlyBreakReturn;
  
  // Debug: Log when functions are made available
  console.log('✅ Global functions registered:', {
    handleBreakSkipped: typeof window.handleBreakSkipped,
    handleEarlyBreakReturn: typeof window.handleEarlyBreakReturn
  });
  
  console.log('Break Reminder Pro main page loaded successfully!');
});