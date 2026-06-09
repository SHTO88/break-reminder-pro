# Changelog

All notable changes to Break Reminder Pro will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.1] - 2026-06-10

### Fixed

- **VLC media player** is now correctly paused and resumed during breaks. VLC does not register with the Windows SMTC (System Media Transport Controls) API, so previous versions had no effect on it. The app now targets VLC directly via `WM_APPCOMMAND` sent to its visible window, found by process name rather than window class (which varies across VLC versions).
- VLC that was already paused before a break no longer starts playing when the break ends. State is now detected via the Windows Core Audio session API (`IAudioSessionManager2`) — an active audio session means VLC is playing; inactive means paused — so the app only sends commands when needed.
- Media state is now stored in the Rust process (via an `AtomicBool`) instead of `localStorage`. Break windows (`force_break`, `notify`) are separate Tauri webviews with isolated storage, so the previous `localStorage`-based approach meant the "was media playing" flag was never visible to them, causing resume to always be skipped.
- SMTC-based players (Spotify, YouTube in browser, etc.) that were already paused before the break are no longer accidentally resumed when the break ends. The app now records which SMTC sources it paused and only resumes those.
- Media resume no longer silently fails on force break. The resume `invoke` call was made after `closeWindow()`, which destroyed the webview context. Resume now happens before the window closes.

### Changed

- Log file rotation on startup: `app.log` is moved to `app.log.bak` each time the app launches, so each session starts with a clean file. Total log storage is capped at two files (current + previous session).
- Release build log level corrected to `Info` for the file logger (was incorrectly set to `Debug` in all builds, producing verbose output in production).

## [1.1.0] - 2026-06-09

### Fixed

- Release builds from GitHub Actions were crashing on launch due to stale Rust cache and `sed`-based JSON patching corrupting config files. CI now uses Python for safe JSON patching and strips BOM before building.
- Tauri startup errors are now logged to `app.log` instead of silently killing the process.

## [1.0.9] - 2026-06-09

### Fixed

- Force break screen no longer freezes on 0:00 for 1-2 seconds before dismissing. Settings and screen-lock checks now run in parallel, and the window closes before media resume so the desktop appears immediately.
- End-of-break chime was silent after the v1.0.8 refactor. Chime now plays before the window closes so the webview context is still active.

## [1.0.8] - 2026-06-08

### Fixed

- App crashed silently on launch on some Windows machines (v1.0.5–v1.0.7). Root cause was `OpenInputDesktop` API call used for screen lock detection — on machines with stricter security policies it caused an access violation instead of returning null. Replaced with process-based detection (LogonUI.exe).
- Corrupted capabilities config (UTF-8 BOM) introduced in v1.0.6 that prevented the app from starting.
- Tray icon setup no longer panics if the default window icon is missing in the bundle.

## [1.0.7] - 2026-06-08

### Fixed

- End-of-break chime now actually plays when the setting is enabled. It was wired up in settings but never called at break end.

### Added

- Press `S` during a force break to toggle "Keep media paused after break". Also auto-reveals the controls panel so you can see the current state.

## [1.0.6] - 2026-06-05

### Added

- Force break screen now shows a "Keep media paused after break" toggle (visible when ESC is pressed and media was playing). Applies to that break only.

### Fixed

- Screen lock detection now correctly identifies a locked session using process detection instead of desktop API calls, which were unreliable inside a WebView.

## [1.0.5] - 2026-06-05

### Fixed

- Media playback is no longer resumed after a break if the screen is locked when the break ends.

## [1.0.4] - 2026-06-01

### Fixed

- Media that was already paused before a break no longer starts playing after the break ends. The app now checks playback state before pausing and only resumes if it was the one that paused it.

## [1.0.3] - 2025-12-02

### Added

- Auto resume playback after break is over

### Fixed

- Auto pause media was causing some issues with particular apps.

## [1.0.0] - 2024-01-XX

### 🎉 Initial Release

#### Added

- **Core Break Management**

  - Customizable break intervals (minutes and seconds)
  - Multiple break modes: Force Break, Notification, Lock Screen
  - Recurring break cycles for continuous productivity
  - Break duration customization

- **Smart Features**

  - Meeting detection for video calls (Zoom, Teams, Skype, WebEx, etc.)
  - Auto-pause media during breaks
  - Pre-break warnings with countdown
  - Break chimes (optional audio notifications)

- **System Integration**

  - System tray with context menu
  - Auto-start with Windows (optional)
  - Hide to tray instead of closing
  - Native Windows look and feel

- **User Interface**

  - Modern dark theme design
  - Responsive layout
  - Drag and drop notification windows
  - Keyboard shortcuts support
  - Persistent settings storage

- **Privacy & Security**
  - No data collection or tracking
  - Completely offline operation
  - Local settings storage only
  - Open source codebase

#### Technical Details

- Built with Tauri 2.0 for optimal performance
- Rust backend for system integration
- Modern web frontend (HTML5, CSS3, JavaScript)
- Minimal resource usage (~50MB RAM)
- Windows 10/11 compatibility

#### Break Modes

- **Force Break**: Fullscreen overlay with countdown timer
- **Notification**: Popup window with break reminder
- **Lock Screen**: Automatically locks the computer

#### Smart Detection

- Detects active video conferencing applications
- Monitors browser tabs for web-based meetings
- Postpones breaks during detected meetings
- Configurable meeting detection sensitivity

---

## Future Releases

### Planned Features

- [ ] Multiple monitor support improvements
- [ ] Custom break messages and motivational quotes
- [ ] Additional break modes (gentle reminder, etc.)
- [ ] Customizable break chime sounds
- [ ] Themes and appearance customization
- [ ] Localization for multiple languages
- [ ] macOS and Linux support

### Under Consideration

- [ ] Stretch exercise suggestions during breaks

---

## Version History

### Version Numbering

- **Major** (X.0.0): Breaking changes or major new features
- **Minor** (0.X.0): New features, backwards compatible
- **Patch** (0.0.X): Bug fixes and small improvements

### Release Schedule

- **Major releases**: Every 6-12 months
- **Minor releases**: Every 2-3 months
- **Patch releases**: As needed for critical fixes

---

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Reporting bugs
- Suggesting features
- Submitting code changes
- Improving documentation

## Support

- **Issues**: [GitHub Issues](https://github.com/SHTO88/break-reminder-pro/issues)
- **Discussions**: [GitHub Discussions](https://github.com/SHTO88/break-reminder-pro/discussions)
- **Documentation**: [README.md](README.md)
