# Break Reminder Pro (Tauri Edition)

Break Reminder Pro is a minimalist, resource-efficient Windows desktop application built with Tauri. It helps users maintain their health and focus by providing configurable, assertive, and intelligent break notifications.

## Prerequisites

- [Node.js](https://nodejs.org/) (v18 or newer recommended)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites/):
  - Install with: `cargo install tauri-cli`

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```
2. (First time only) Build Rust dependencies:
   ```bash
   cd src-tauri
   cargo build
   ```

## Running the App (Development)

Start the Tauri development server:

```bash
npm run tauri dev
```

This will launch the Break Reminder Pro app with hot reload for frontend and backend code.

## Building the Windows Installer

To build a production MSI installer:

```bash
npm run tauri build
```

The installer will be generated in the `src-tauri/target/release/bundle/msi/` directory.

## Testing

- Test all break modes: Force Break, Notify, Lock Screen.
- Test smart features: Auto-Pause Media, Meeting Detection, End-of-Break Chime, Pre-Break Notifications.
- Test settings persistence and auto-start with Windows.
- Test timer and UI responsiveness.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
