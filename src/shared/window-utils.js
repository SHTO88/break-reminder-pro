/**
 * Shared Window Utilities Module
 * Common window management functions used across break windows
 */

const { invoke } = window.__TAURI__.core;

/**
 * Window Utilities Class
 */
export class WindowUtils {
  /**
   * Close a window by label
   */
  static async closeWindow(label) {
    try {
      await invoke('close_window', { label });
      console.log(`Window ${label} closed successfully`);
    } catch (error) {
      console.error(`Error closing window ${label}:`, error);
      // Fallback: try to close directly
      try {
        const { getCurrentWindow } = window.__TAURI__.window;
        await getCurrentWindow().close();
      } catch (fallbackError) {
        console.error('Fallback close failed:', fallbackError);
      }
    }
  }

  /**
   * Get screen dimensions from Tauri
   */
  static async getScreenDimensions() {
    try {
      const [width, height] = await invoke('get_primary_monitor_size');
      console.log(`📺 Screen dimensions from Tauri: ${width}x${height}`);
      return { width, height };
    } catch (error) {
      console.warn('⚠️ Failed to get monitor size from Tauri, using screen API:', error);
      const fallbackWidth = screen.width || 1920;
      const fallbackHeight = screen.height || 1080;
      console.log(`📺 Fallback screen dimensions: ${fallbackWidth}x${fallbackHeight}`);
      return { width: fallbackWidth, height: fallbackHeight };
    }
  }

  /**
   * Position window at specified location
   */
  static async positionWindow(x, y) {
    try {
      const { getCurrentWindow } = window.__TAURI__.window;
      const currentWindow = getCurrentWindow();
      
      // Ensure coordinates are valid numbers
      const posX = Math.max(0, Math.floor(x));
      const posY = Math.max(0, Math.floor(y));
      
      await currentWindow.setPosition({ x: posX, y: posY });
      console.log(`✅ Window positioned at (${posX}, ${posY})`);
      
      // Verify position was set
      setTimeout(async () => {
        try {
          const actualPos = await currentWindow.outerPosition();
          console.log(`📍 Actual window position: (${actualPos.x}, ${actualPos.y})`);
        } catch (e) {
          console.warn('Could not verify window position:', e);
        }
      }, 100);
      
    } catch (error) {
      console.error('❌ Error positioning window:', error);
      throw error;
    }
  }

  /**
   * Show and focus window
   */
  static async showAndFocus() {
    try {
      const { getCurrentWindow } = window.__TAURI__.window;
      const currentWindow = getCurrentWindow();
      await currentWindow.show();
      await currentWindow.setFocus();
    } catch (error) {
      console.error('Error showing/focusing window:', error);
    }
  }

  /**
   * Set window always on top
   */
  static async setAlwaysOnTop(onTop = true) {
    try {
      const { getCurrentWindow } = window.__TAURI__.window;
      const currentWindow = getCurrentWindow();
      await currentWindow.setAlwaysOnTop(onTop);
    } catch (error) {
      console.error('Error setting always on top:', error);
    }
  }

  /**
   * Get window size
   */
  static async getWindowSize() {
    try {
      const { getCurrentWindow } = window.__TAURI__.window;
      const currentWindow = getCurrentWindow();
      return await currentWindow.outerSize();
    } catch (error) {
      console.error('Error getting window size:', error);
      return { width: 400, height: 300 };
    }
  }
}

/**
 * Window Positioning Manager
 */
export class WindowPositionManager {
  constructor(windowLabel) {
    this.windowLabel = windowLabel;
    this.storageKey = `${windowLabel}Position`;
  }

  /**
   * Save current window position to localStorage
   */
  async savePosition() {
    try {
      const { getCurrentWindow } = window.__TAURI__.window;
      const currentWindow = getCurrentWindow();
      const position = await currentWindow.outerPosition();
      localStorage.setItem(this.storageKey, JSON.stringify(position));
      console.log(`Position saved for ${this.windowLabel}:`, position);
    } catch (error) {
      console.error('Error saving position:', error);
    }
  }

  /**
   * Load saved position from localStorage
   */
  loadSavedPosition() {
    try {
      const saved = localStorage.getItem(this.storageKey);
      return saved ? JSON.parse(saved) : null;
    } catch (error) {
      console.error('Error loading saved position:', error);
      return null;
    }
  }

  /**
   * Position window using saved position or default
   */
  async positionWindow(defaultPositionFn) {
    try {
      // Wait a moment for Rust positioning to complete
      await new Promise(resolve => setTimeout(resolve, 50));
      
      // Check if window was positioned by Rust
      if (window.RUST_POSITIONED) {
        console.log(`🎯 ${this.windowLabel} already positioned by Rust - skipping JS positioning`);
        
        // Just ensure window is shown and configured properly
        await WindowUtils.showAndFocus();
        await WindowUtils.setAlwaysOnTop(true);
        return;
      }

      console.log(`🎯 Rust positioning not detected for ${this.windowLabel}, using JavaScript fallback`);
      
      // Wait a bit more to ensure window is ready
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Try saved position first
      const savedPosition = this.loadSavedPosition();
      if (savedPosition && savedPosition.x !== undefined && savedPosition.y !== undefined) {
        console.log(`📍 Using saved position for ${this.windowLabel}:`, savedPosition);
        await WindowUtils.positionWindow(savedPosition.x, savedPosition.y);
        await WindowUtils.showAndFocus();
        await WindowUtils.setAlwaysOnTop(true);
        return;
      }

      // Use default positioning function
      if (defaultPositionFn) {
        console.log(`📍 Using default positioning for ${this.windowLabel}`);
        await defaultPositionFn();
      }
    } catch (error) {
      console.error(`❌ Error in positionWindow for ${this.windowLabel}:`, error);
      // Fallback to default positioning
      if (defaultPositionFn) {
        try {
          await defaultPositionFn();
        } catch (fallbackError) {
          console.error(`❌ Fallback positioning failed for ${this.windowLabel}:`, fallbackError);
        }
      }
    }
  }

  /**
   * Default positioning strategies
   */
  static async positionAtCenter() {
    try {
      const screen = await WindowUtils.getScreenDimensions();
      const size = await WindowUtils.getWindowSize();
      
      const x = Math.floor((screen.width - size.width) / 2);
      const y = Math.floor((screen.height - size.height) / 2);
      
      console.log(`🎯 Positioning at CENTER: (${x}, ${y}) on ${screen.width}x${screen.height} screen`);
      console.log(`📏 Window size: ${size.width}x${size.height}`);
      
      await WindowUtils.positionWindow(x, y);
      await WindowUtils.showAndFocus();
    } catch (error) {
      console.error('❌ Error in positionAtCenter:', error);
      // Fallback to basic show
      await WindowUtils.showAndFocus();
    }
  }

  static async positionAtBottomCenter() {
    try {
      const screen = await WindowUtils.getScreenDimensions();
      const size = await WindowUtils.getWindowSize();
      
      const x = Math.floor((screen.width - size.width) / 2);
      const y = screen.height - size.height - 150; // 150px from bottom
      
      console.log(`🎯 Positioning at BOTTOM CENTER: (${x}, ${y}) on ${screen.width}x${screen.height} screen`);
      console.log(`📏 Window size: ${size.width}x${size.height}`);
      
      await WindowUtils.positionWindow(x, y);
      await WindowUtils.showAndFocus();
      await WindowUtils.setAlwaysOnTop(true);
    } catch (error) {
      console.error('❌ Error in positionAtBottomCenter:', error);
      // Fallback to basic show with always on top
      await WindowUtils.showAndFocus();
      await WindowUtils.setAlwaysOnTop(true);
    }
  }
}

/**
 * Auto-hide functionality for notifications
 */
export class AutoHideManager {
  constructor(duration = 3000, onComplete = null) {
    this.duration = duration;
    this.onComplete = onComplete || (() => {});
    this.timer = null;
    this.progressTimer = null;
    this.startTime = null;
    this.isPaused = false;
    this.progressElement = null;
  }

  /**
   * Set progress bar element
   */
  setProgressElement(element) {
    this.progressElement = element;
  }

  /**
   * Start auto-hide timer
   */
  start() {
    this.cancel(); // Clear any existing timers
    
    if (this.progressElement) {
      this.progressElement.classList.add('visible');
      this.progressElement.classList.remove('paused');
    }

    this.startTime = Date.now();
    this.isPaused = false;

    // Update progress bar
    this.progressTimer = setInterval(() => {
      if (this.isPaused) return;

      const elapsed = Date.now() - this.startTime;
      const progress = Math.min((elapsed / this.duration) * 100, 100);
      
      if (this.progressElement) {
        this.progressElement.style.width = progress + '%';
      }

      if (progress >= 100) {
        clearInterval(this.progressTimer);
      }
    }, 50);

    // Set main timer
    this.timer = setTimeout(() => {
      this.onComplete();
    }, this.duration);
  }

  /**
   * Pause auto-hide
   */
  pause() {
    if (!this.isPaused && this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
      this.isPaused = true;
      
      if (this.progressElement) {
        this.progressElement.classList.add('paused');
      }
    }
  }

  /**
   * Resume auto-hide
   */
  resume() {
    if (this.isPaused) {
      this.isPaused = false;
      
      if (this.progressElement) {
        this.progressElement.classList.remove('paused');
      }
      
      // Calculate remaining time
      const currentProgress = this.progressElement ? 
        parseFloat(this.progressElement.style.width) || 0 : 0;
      const elapsedTime = (currentProgress / 100) * this.duration;
      const remainingTime = this.duration - elapsedTime;
      
      // Update start time for progress calculation
      this.startTime = Date.now() - elapsedTime;
      
      // Set new timer for remaining time
      this.timer = setTimeout(() => {
        this.onComplete();
      }, remainingTime);
    }
  }

  /**
   * Cancel auto-hide
   */
  cancel() {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
    if (this.progressTimer) {
      clearInterval(this.progressTimer);
      this.progressTimer = null;
    }
    
    if (this.progressElement) {
      this.progressElement.classList.remove('visible');
      this.progressElement.style.width = '0%';
    }
    
    this.isPaused = false;
  }
}