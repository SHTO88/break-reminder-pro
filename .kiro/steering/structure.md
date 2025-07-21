# Break Reminder Pro - Project Structure

## Root Directory
```
break-reminder-pro/
├── src/                    # Frontend web assets
├── src-tauri/             # Rust backend application
├── node_modules/          # Node.js dependencies
├── .kiro/                 # Kiro IDE configuration
├── .git/                  # Git repository
├── package.json           # Node.js project configuration
├── README.md              # Project documentation
└── TODO.md                # Development tasks
```

## Frontend Structure (`src/`)
```
src/
├── assets/                # Static assets (images, icons)
├── index.html            # Main application UI
├── main.js               # Core JavaScript logic (930+ lines)
├── styles.css            # Modern CSS styling
├── force_break.html      # Fullscreen break window
├── notify.html           # Break notification popup
├── pre_break.html        # Pre-break warning window
└── test.html             # Debug/testing window
```

## Backend Structure (`src-tauri/`)
```
src-tauri/
├── src/
│   ├── lib.rs            # Main Rust application logic
│   └── main.rs           # Entry point
├── capabilities/         # Tauri security capabilities
├── icons/               # Application icons
├── target/              # Rust build artifacts
├── Cargo.toml           # Rust dependencies and configuration
├── tauri.conf.json      # Tauri application configuration
└── build.rs             # Build script
```

## Key Files & Responsibilities

### Frontend Core Files
- **`index.html`**: Main UI with timer display, settings forms, and debug panel
- **`main.js`**: Timer logic, settings management, Tauri command invocation, debug utilities
- **`styles.css`**: Modern dark theme with CSS Grid/Flexbox layouts

### Break Window Files
- **`force_break.html`**: Fullscreen black window with countdown timer
- **`notify.html`**: Small notification popup (400x200px)
- **`pre_break.html`**: Pre-break warning window (350x180px)

### Backend Core Files
- **`lib.rs`**: Tauri commands for window management, system integration, settings persistence
- **`main.rs`**: Application entry point

### Configuration Files
- **`package.json`**: npm scripts for dev/build, Tauri CLI dependency
- **`Cargo.toml`**: Rust dependencies, build profiles (dev/dev-fast/release)
- **`tauri.conf.json`**: Window configuration, bundle settings, security policies

## Coding Conventions

### JavaScript
- ES6+ syntax with async/await
- Modular functions with clear naming
- Event-driven architecture for UI updates
- Comprehensive error handling and logging
- Debug utilities integrated into main application

### Rust
- Tauri command pattern for frontend-backend communication
- Async functions for system operations
- Comprehensive error handling with Result types
- Thread spawning for window creation to prevent blocking

### CSS
- CSS custom properties (variables) for theming
- Modern layout with Grid and Flexbox
- Mobile-responsive design patterns
- Component-based class naming

## Data Flow
1. **Settings**: JSON persistence via Tauri store plugin
2. **Timer State**: JavaScript-managed with localStorage backup
3. **System Integration**: Rust commands for Windows-specific features
4. **Window Management**: Tauri WebviewWindowBuilder for break windows