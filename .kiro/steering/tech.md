# Break Reminder Pro - Technical Stack

## Framework & Architecture
- **Framework**: Tauri 2.0 (Rust backend + Web frontend)
- **Backend**: Rust with Tauri commands for system integration
- **Frontend**: Vanilla JavaScript, HTML5, CSS3 (no frameworks for minimal footprint)
- **Build System**: Tauri CLI with npm scripts

## Key Dependencies

### Rust Dependencies (Cargo.toml)
- `tauri`: Main framework with unstable features enabled
- `tauri-plugin-store`: Settings persistence
- `tauri-plugin-autostart`: Windows startup integration
- `enigo`: Media control via keyboard simulation
- `sysinfo`: Process detection for meeting awareness
- `serde`: JSON serialization for settings

### Frontend Dependencies (package.json)
- `@tauri-apps/cli`: Build and development tooling
- `@tauri-apps/plugin-store`: Frontend store integration

## Build Profiles
- **dev**: Optimized for development with debug info
- **dev-fast**: Maximum compilation speed for rapid iteration
- **release**: Production builds with full optimization

## Common Commands

### Development
```bash
# Start development server with hot reload
npm run dev

# Fast development (minimal optimization)
npm run dev-fast

# Install dependencies
npm install
```

### Building
```bash
# Build production MSI installer
npm run build

# Build debug version
npm run build-debug

# Build Rust dependencies (first time setup)
cd src-tauri && cargo build
```

### Testing
- Manual testing via built-in debug panel
- Test break modes, smart features, and system integration
- Verify settings persistence and autostart functionality

## Architecture Patterns
- **Command Pattern**: Rust functions exposed as Tauri commands
- **Event-Driven**: Timer-based UI updates and break triggers
- **State Management**: JSON file persistence for user settings
- **Window Management**: Multiple webview windows for different break modes

## File Structure Conventions
- Frontend assets in `src/` directory
- Rust backend in `src-tauri/src/`
- Settings stored in OS-specific app data directory
- Build outputs in `src-tauri/target/release/bundle/`