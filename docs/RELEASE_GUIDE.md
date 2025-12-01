# Release Guide

This guide explains how to create a new release for Break Reminder Pro.

## Prerequisites

- Git installed and configured
- Write access to the repository
- All changes committed and pushed to main branch

## Release Process

### Step 1: Update Version Numbers

Update the version in **both** files to match your new version:

**File: `src-tauri/Cargo.toml`**
```toml
[package]
version = "1.0.2"  # Update this line
```

**File: `src-tauri/tauri.conf.json`**
```json
{
  "version": "1.0.2",  // Update this line
}
```

### Step 2: Update CHANGELOG.md

Add your changes at the top of the file, below the header:

```markdown
## [1.0.2] - 2024-01-15

### Added
- New feature description

### Fixed
- Bug fix description

### Changed
- Change description
```

**Categories to use:**
- `Added` - New features
- `Fixed` - Bug fixes
- `Changed` - Changes to existing features
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Security` - Security fixes

### Step 3: Commit Changes

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore: release v1.0.2"
git push origin main
```

### Step 4: Create and Push Tag

```bash
git tag v1.0.2
git push origin v1.0.2
```

**Important:** The tag must start with `v` (e.g., `v1.0.2`, not `1.0.2`)

### Step 5: Wait for Build

The GitHub Actions workflow will automatically:
1. Build Windows installers (.exe and .msi)
2. Create a GitHub release
3. Upload the installers to the release
4. Generate release notes with download links

**Build time:** Approximately 10-15 minutes

### Step 6: Verify Release

1. Go to: `https://github.com/SHTO88/break-reminder-pro/releases`
2. Check that the new release appears
3. Verify version numbers in filenames match
4. Test download links work
5. Optionally: Edit release notes to add more details

## Quick Command Reference

```bash
# Complete release in one go (after updating files)
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore: release v1.0.2"
git push origin main
git tag v1.0.2
git push origin v1.0.2
```

## Alternative: Manual Workflow Trigger

If you need to rebuild a release without creating a new tag:

1. Go to: `https://github.com/SHTO88/break-reminder-pro/actions`
2. Click on "Release" workflow
3. Click "Run workflow" button
4. Enter the tag (e.g., `v1.0.2`)
5. Click "Run workflow"

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **Major (X.0.0)**: Breaking changes, major rewrites
  - Example: `v2.0.0` - Complete UI redesign
  
- **Minor (0.X.0)**: New features, backwards compatible
  - Example: `v1.1.0` - Added macOS support
  
- **Patch (0.0.X)**: Bug fixes, small improvements
  - Example: `v1.0.1` - Fixed timer bug

## Troubleshooting

### Build Failed

1. Check GitHub Actions logs for errors
2. Common issues:
   - Syntax error in Cargo.toml or tauri.conf.json
   - Version mismatch between files
   - Missing dependencies

### Wrong Version in Release

1. Delete the tag: `git tag -d v1.0.2 && git push origin :refs/tags/v1.0.2`
2. Delete the release on GitHub
3. Fix version numbers
4. Create tag again

### Release Not Created

- Ensure tag starts with `v`
- Check GitHub Actions permissions
- Verify workflow file syntax

## Post-Release Checklist

- [ ] Verify release appears on GitHub
- [ ] Test download links
- [ ] Download and test installers
- [ ] Update README if needed
- [ ] Announce release (if applicable)
- [ ] Close related issues/PRs

## Example Release Notes

When editing the release on GitHub, you can enhance the auto-generated notes:

```markdown
## 🎉 What's New in v1.0.2

### ✨ New Features
- Media playback now automatically resumes after breaks end
- Improved meeting detection accuracy

### 🐛 Bug Fixes
- Fixed version numbers in release artifacts
- Resolved timer pause issue

### 📦 Installation
Download the installer below and run it. Your settings will be preserved if upgrading.

### 🔄 Upgrading
Simply install over your existing version - no uninstall needed!
```

## Need Help?

- Check [GitHub Actions logs](https://github.com/SHTO88/break-reminder-pro/actions)
- Open an issue if you encounter problems
- Review previous releases for examples
