/**
 * Shared Timer Utilities Module
 * Common timer functions used across the application
 */

/**
 * Timer Utilities Class
 */
export class TimerUtils {
  /**
   * Format seconds into MM:SS format
   */
  static formatTime(seconds) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  /**
   * Parse time inputs and return total seconds
   */
  static parseTimeInputs(minutesElement, secondsElement) {
    const minutes = parseInt(minutesElement.value, 10) || 0;
    const seconds = parseInt(secondsElement.value, 10) || 0;
    return (minutes * 60) + seconds;
  }

  /**
   * Validate time input values
   */
  static validateTimeInput(input, maxValue) {
    let value = parseInt(input.value, 10);
    if (isNaN(value) || value < 0) {
      input.value = 0;
    } else if (value > maxValue) {
      input.value = maxValue;
    } else {
      input.value = value;
    }
  }

  /**
   * Get break duration from URL parameters or localStorage
   */
  static getBreakDuration(defaultDuration = 300) {
    try {
      // First try URL parameters
      const urlParams = new URLSearchParams(window.location.search);
      const duration = urlParams.get('duration');
      
      if (duration && !isNaN(parseInt(duration))) {
        const durationInt = parseInt(duration);
        console.log('Using duration from URL:', durationInt);
        return durationInt;
      }

      // Fallback to localStorage
      const stored = localStorage.getItem('breakDuration');
      if (stored && !isNaN(parseInt(stored))) {
        console.log('Using duration from localStorage:', stored);
        return parseInt(stored);
      }
    } catch (error) {
      console.error('Error getting break duration:', error);
    }

    console.log('Using default duration:', defaultDuration);
    return defaultDuration;
  }
}

/**
 * Countdown Timer Class
 */
export class CountdownTimer {
  constructor(initialSeconds, onTick, onComplete) {
    this.totalSeconds = initialSeconds;
    this.currentSeconds = initialSeconds;
    this.onTick = onTick || (() => {});
    this.onComplete = onComplete || (() => {});
    this.interval = null;
    this.isRunning = false;
    this.isPaused = false;
  }

  /**
   * Start the countdown timer
   */
  start() {
    if (this.isRunning) return;
    
    this.isRunning = true;
    this.isPaused = false;
    
    // Initial tick
    this.onTick(this.currentSeconds, this.totalSeconds);
    
    this.interval = setInterval(() => {
      this.currentSeconds--;
      this.onTick(this.currentSeconds, this.totalSeconds);
      
      if (this.currentSeconds <= 0) {
        this.stop();
        this.onComplete();
      }
    }, 1000);
  }

  /**
   * Pause the timer
   */
  pause() {
    if (!this.isRunning || this.isPaused) return;
    
    this.isPaused = true;
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  /**
   * Resume the timer
   */
  resume() {
    if (!this.isRunning || !this.isPaused) return;
    
    this.isPaused = false;
    this.interval = setInterval(() => {
      this.currentSeconds--;
      this.onTick(this.currentSeconds, this.totalSeconds);
      
      if (this.currentSeconds <= 0) {
        this.stop();
        this.onComplete();
      }
    }, 1000);
  }

  /**
   * Stop the timer
   */
  stop() {
    this.isRunning = false;
    this.isPaused = false;
    
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  /**
   * Reset timer to initial value
   */
  reset(newSeconds = null) {
    this.stop();
    this.totalSeconds = newSeconds || this.totalSeconds;
    this.currentSeconds = this.totalSeconds;
  }

  /**
   * Get current progress as percentage
   */
  getProgress() {
    return ((this.totalSeconds - this.currentSeconds) / this.totalSeconds) * 100;
  }

  /**
   * Get remaining time formatted
   */
  getFormattedTime() {
    return TimerUtils.formatTime(this.currentSeconds);
  }
}