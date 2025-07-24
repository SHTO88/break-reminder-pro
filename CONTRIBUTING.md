# Contributing to Break Reminder Pro

Thank you for your interest in contributing to Break Reminder Pro! We welcome contributions from the community and are grateful for your support in making this project better.

## 🌟 Ways to Contribute

### 🐛 Report Bugs
- Use the [GitHub Issues](https://github.com/SHTO88/break-reminder-pro/issues) page
- Search existing issues first to avoid duplicates
- Provide detailed information about the bug
- Include steps to reproduce the issue
- Mention your Windows version and app version

### 💡 Suggest Features
- Open a [Feature Request](https://github.com/SHTO88/break-reminder-pro/issues/new)
- Describe the feature and its benefits
- Explain how it aligns with the project's goals
- Consider privacy and simplicity implications

### 🔧 Submit Code Changes
- Fork the repository
- Create a feature branch
- Make your changes
- Test thoroughly
- Submit a pull request

### 📖 Improve Documentation
- Fix typos or unclear explanations
- Add examples or clarifications
- Update outdated information
- Translate documentation

## 🚀 Getting Started

### Development Setup
1. **Prerequisites**
   - Node.js (v18+)
   - Rust (latest stable)
   - Tauri CLI

2. **Clone and Setup**
   ```bash
   git clone https://github.com/SHTO88/break-reminder-pro.git
   cd break-reminder-pro
   npm install
   cd src-tauri && cargo build && cd ..
   ```

3. **Development Commands**
   ```bash
   npm run dev          # Start development server
   npm run dev-fast     # Fast development build
   npm run build        # Production build
   ```

### Project Structure
```
break-reminder-pro/
├── src/                 # Frontend (HTML, CSS, JS)
├── src-tauri/          # Backend (Rust)
│   ├── src/            # Rust source code
│   ├── icons/          # App icons
│   └── Cargo.toml      # Rust dependencies
├── .kiro/              # Kiro IDE configuration
└── README.md           # Project documentation
```

## 📋 Development Guidelines

### Code Style
- **Frontend**: Follow existing HTML/CSS/JS patterns
- **Backend**: Use `cargo fmt` for Rust formatting
- **Comments**: Add clear comments for complex logic
- **Naming**: Use descriptive variable and function names

### Testing
- Test all break modes (Force, Notify, Lock)
- Test smart features (meeting detection, media pause)
- Test system integration (tray, autostart)
- Test on different Windows versions if possible

### Commit Messages
Use clear, descriptive commit messages:
```
feat: add meeting detection for Teams
fix: resolve system tray icon not showing
docs: update installation instructions
refactor: simplify timer logic
```

### Pull Request Process
1. **Before Submitting**
   - Ensure code follows project style
   - Test your changes thoroughly
   - Update documentation if needed
   - Check that build passes

2. **PR Description**
   - Describe what changes you made
   - Explain why the changes are needed
   - Reference any related issues
   - Include screenshots for UI changes

3. **Review Process**
   - Maintainers will review your PR
   - Address any feedback promptly
   - Be open to suggestions and changes

## 🎯 Project Goals

Keep these principles in mind when contributing:

### Privacy First
- No data collection or tracking
- All data stays on user's device
- No external network requests (except updates)

### Simplicity
- Clean, intuitive user interface
- Minimal configuration required
- Focus on core functionality

### Performance
- Lightweight and fast
- Minimal resource usage
- Quick startup time

### Accessibility
- Clear visual design
- Keyboard shortcuts
- Screen reader compatibility

## 🚫 What We Don't Accept

- Features that compromise privacy
- Unnecessary complexity or bloat
- Dependencies with security concerns
- Changes that break existing functionality
- Code without proper testing

## 📞 Getting Help

### Questions?
- Check existing [Issues](https://github.com/SHTO88/break-reminder-pro/issues)
- Start a [Discussion](https://github.com/SHTO88/break-reminder-pro/discussions)
- Contact maintainers directly

### Stuck on Development?
- Review the [README.md](README.md) setup instructions
- Check Tauri documentation
- Ask in GitHub Discussions

## 🏆 Recognition

Contributors will be:
- Listed in the project's contributors section
- Mentioned in release notes for significant contributions
- Given credit in the app's about section

## 📜 Code of Conduct

### Our Standards
- Be respectful and inclusive
- Focus on constructive feedback
- Help create a welcoming environment
- Respect different viewpoints and experiences

### Unacceptable Behavior
- Harassment or discrimination
- Trolling or inflammatory comments
- Personal attacks
- Publishing private information

### Enforcement
Violations may result in temporary or permanent bans from the project.

---

Thank you for contributing to Break Reminder Pro! Together, we can build a better, healthier way to work. 🚀