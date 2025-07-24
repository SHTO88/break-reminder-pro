/**
 * Update Manager Module
 * Handles checking for app updates and notifying users
 */

const { invoke } = window.__TAURI__.core;

/**
 * Update Manager Class
 */
export class UpdateManager {
  constructor() {
    this.checkInterval = 24 * 60 * 60 * 1000; // 24 hours
    this.lastCheckKey = 'lastUpdateCheck';
    this.updateDisabledKey = 'updateCheckDisabled';
    this.repoUrl = 'https://api.github.com/repos/SHTO88/break-reminder-pro/releases/latest';
  }

  /**
   * Check if update checking is enabled
   */
  isUpdateCheckEnabled() {
    const disabled = localStorage.getItem(this.updateDisabledKey);
    return disabled !== 'true';
  }

  /**
   * Enable or disable update checking
   */
  setUpdateCheckEnabled(enabled) {
    localStorage.setItem(this.updateDisabledKey, enabled ? 'false' : 'true');
    console.log(`Update checking ${enabled ? 'enabled' : 'disabled'}`);
  }

  /**
   * Check if enough time has passed since last check
   */
  shouldCheckForUpdates() {
    const lastCheck = localStorage.getItem(this.lastCheckKey);
    if (!lastCheck) return true;

    const now = Date.now();
    const timeSinceLastCheck = now - parseInt(lastCheck);
    return timeSinceLastCheck >= this.checkInterval;
  }

  /**
   * Compare version strings (semantic versioning)
   */
  isNewerVersion(latest, current) {
    try {
      // Remove 'v' prefix if present
      const latestClean = latest.replace(/^v/, '');
      const currentClean = current.replace(/^v/, '');
      
      const latestParts = latestClean.split('.').map(Number);
      const currentParts = currentClean.split('.').map(Number);
      
      // Pad arrays to same length
      const maxLength = Math.max(latestParts.length, currentParts.length);
      while (latestParts.length < maxLength) latestParts.push(0);
      while (currentParts.length < maxLength) currentParts.push(0);
      
      for (let i = 0; i < maxLength; i++) {
        if (latestParts[i] > currentParts[i]) return true;
        if (latestParts[i] < currentParts[i]) return false;
      }
      
      return false; // Versions are equal
    } catch (error) {
      console.error('Error comparing versions:', error);
      return false;
    }
  }

  /**
   * Fetch latest release information from GitHub
   */
  async fetchLatestRelease() {
    try {
      console.log('🔄 Checking for updates...');
      const response = await fetch(this.repoUrl);
      
      if (!response.ok) {
        throw new Error(`GitHub API responded with ${response.status}: ${response.statusText}`);
      }
      
      const release = await response.json();
      console.log('📦 Latest release info:', {
        version: release.tag_name,
        published: release.published_at,
        prerelease: release.prerelease
      });
      
      return release;
    } catch (error) {
      console.error('❌ Failed to fetch latest release:', error);
      throw error;
    }
  }

  /**
   * Get current app version
   */
  async getCurrentVersion() {
    try {
      const version = await invoke('get_app_version');
      console.log('📱 Current app version:', version);
      return version;
    } catch (error) {
      console.error('❌ Failed to get current version:', error);
      return '0.0.0'; // Fallback version
    }
  }

  /**
   * Show update notification to user
   */
  async showUpdateNotification(release) {
    try {
      console.log('🔔 Showing update notification for version:', release.tag_name);
      
      await invoke('show_update_notification', {
        version: release.tag_name,
        notes: release.body || 'No release notes available.',
        downloadUrl: release.html_url,
        publishedAt: release.published_at
      });
    } catch (error) {
      console.error('❌ Failed to show update notification:', error);
      
      // Fallback: show browser alert
      const shouldUpdate = confirm(
        `🎉 New version ${release.tag_name} is available!\n\n` +
        `Would you like to download it now?`
      );
      
      if (shouldUpdate) {
        await invoke('open_url', { url: release.html_url });
      }
    }
  }

  /**
   * Check for updates (main function)
   */
  async checkForUpdates(force = false) {
    try {
      // Check if update checking is enabled
      if (!force && !this.isUpdateCheckEnabled()) {
        console.log('⏭️ Update checking is disabled');
        return { hasUpdate: false, reason: 'disabled' };
      }

      // Check if enough time has passed
      if (!force && !this.shouldCheckForUpdates()) {
        console.log('⏭️ Too soon since last update check');
        return { hasUpdate: false, reason: 'too_soon' };
      }

      // Update last check timestamp
      localStorage.setItem(this.lastCheckKey, Date.now().toString());

      // Fetch latest release and current version
      const [latestRelease, currentVersion] = await Promise.all([
        this.fetchLatestRelease(),
        this.getCurrentVersion()
      ]);

      // Skip pre-releases unless explicitly checking
      if (latestRelease.prerelease && !force) {
        console.log('⏭️ Skipping pre-release version:', latestRelease.tag_name);
        return { hasUpdate: false, reason: 'prerelease' };
      }

      // Compare versions
      const hasUpdate = this.isNewerVersion(latestRelease.tag_name, currentVersion);
      
      if (hasUpdate) {
        console.log('🎉 Update available!', {
          current: currentVersion,
          latest: latestRelease.tag_name
        });
        
        await this.showUpdateNotification(latestRelease);
        
        return {
          hasUpdate: true,
          currentVersion,
          latestVersion: latestRelease.tag_name,
          release: latestRelease
        };
      } else {
        console.log('✅ App is up to date');
        return {
          hasUpdate: false,
          reason: 'up_to_date',
          currentVersion,
          latestVersion: latestRelease.tag_name
        };
      }

    } catch (error) {
      console.error('❌ Update check failed:', error);
      return {
        hasUpdate: false,
        reason: 'error',
        error: error.message
      };
    }
  }

  /**
   * Manual update check (for settings button)
   */
  async checkForUpdatesManually() {
    console.log('🔍 Manual update check requested');
    const result = await this.checkForUpdates(true); // Force check
    
    if (!result.hasUpdate) {
      // Show "no updates" message for manual checks
      let message = '✅ You have the latest version!';
      
      if (result.currentVersion && result.latestVersion) {
        message += `\n\nCurrent: ${result.currentVersion}\nLatest: ${result.latestVersion}`;
      }
      
      if (result.reason === 'error') {
        message = `❌ Failed to check for updates:\n${result.error}`;
      }
      
      alert(message);
    }
    
    return result;
  }

  /**
   * Get last check information
   */
  getLastCheckInfo() {
    const lastCheck = localStorage.getItem(this.lastCheckKey);
    if (!lastCheck) return null;
    
    const timestamp = parseInt(lastCheck);
    const date = new Date(timestamp);
    
    return {
      timestamp,
      date,
      timeAgo: this.getTimeAgo(timestamp)
    };
  }

  /**
   * Get human-readable time ago string
   */
  getTimeAgo(timestamp) {
    const now = Date.now();
    const diff = now - timestamp;
    
    const minutes = Math.floor(diff / (1000 * 60));
    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    return 'Just now';
  }
}

// Create singleton instance
export const updateManager = new UpdateManager();