/**
 * Shared Settings Management Module
 * Handles all settings operations across the application
 */

const { invoke } = window.__TAURI__.core;

// Default settings configuration
export const DEFAULT_SETTINGS = {
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

/**
 * Settings Manager Class
 */
export class SettingsManager {
  constructor() {
    this.cache = null;
  }

  /**
   * Load settings from backend with caching
   */
  async load() {
    try {
      const settings = await invoke('load_settings');
      this.cache = settings ? { ...DEFAULT_SETTINGS, ...settings } : DEFAULT_SETTINGS;
      console.log('Settings loaded:', this.cache);
      return this.cache;
    } catch (error) {
      console.error('Failed to load settings:', error);
      this.cache = DEFAULT_SETTINGS;
      return this.cache;
    }
  }

  /**
   * Save settings to backend and update cache
   */
  async save(newSettings) {
    try {
      // Merge with existing settings
      const existingSettings = this.cache || await this.load();
      const updatedSettings = { ...existingSettings, ...newSettings };
      
      await invoke('save_settings', { settings: updatedSettings });
      this.cache = updatedSettings;
      console.log('Settings saved:', updatedSettings);
      return updatedSettings;
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  }

  /**
   * Get cached settings or load if not cached
   */
  async get() {
    return this.cache || await this.load();
  }

  /**
   * Clear cache (useful for testing)
   */
  clearCache() {
    this.cache = null;
  }

  /**
   * Get specific setting value
   */
  async getValue(key) {
    const settings = await this.get();
    return settings[key];
  }

  /**
   * Update specific setting
   */
  async updateValue(key, value) {
    return await this.save({ [key]: value });
  }
}

// Create singleton instance
export const settingsManager = new SettingsManager();