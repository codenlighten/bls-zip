# Git Synchronization Guide

> **Keep your Boundless BLS Platform and Enterprise E² Multipass in sync with GitHub**

This guide explains how to keep your local development environment synchronized with the GitHub repository at https://github.com/Saifullah62/BLS

---

## Table of Contents

- [Quick Start](#quick-start)
- [Sync Scripts](#sync-scripts)
- [Manual Sync Workflow](#manual-sync-workflow)
- [Best Practices](#best-practices)
- [Common Scenarios](#common-scenarios)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

### Using Sync Scripts (Recommended)

We provide two sync scripts for easy repository management:

#### Windows Batch Script (sync-github.bat)
```cmd
# Double-click sync-github.bat or run from terminal:
sync-github.bat
```

#### PowerShell Script (sync-github.ps1)
```powershell
# Run from PowerShell:
.\sync-github.ps1

# Or with specific action:
.\sync-github.ps1 -Action pull    # Pull latest changes
.\sync-github.ps1 -Action push    # Push your changes
.\sync-github.ps1 -Action sync    # Full sync (pull + push)
.\sync-github.ps1 -Action status  # Show detailed status
```

---

## Sync Scripts

### Features

Both scripts provide:
- ✅ Interactive menu for sync operations
- ✅ Pull latest changes from GitHub
- ✅ Push your local changes to GitHub
- ✅ Full sync (pull + push)
- ✅ Detailed status information
- ✅ Error handling and conflict detection
- ✅ Automatic commit prompts

### Script Comparison

| Feature | sync-github.bat | sync-github.ps1 |
|---------|----------------|-----------------|
| Windows Compatibility | ✅ All versions | ✅ Windows 7+ |
| Interactive Menu | ✅ | ✅ |
| Color Output | ⚠️ Limited | ✅ Full |
| Error Messages | ✅ | ✅ Enhanced |
| Command-line Args | ❌ | ✅ |

---

## Manual Sync Workflow

If you prefer manual git commands:

### 1. Pull Latest Changes (Update Local)

```bash
# Fetch latest changes from GitHub
git fetch origin

# Pull changes from main branch
git pull origin main
```

**Use this when:**
- Starting work on the project
- Before making new changes
- Someone else updated the repository

### 2. Push Your Changes (Update GitHub)

```bash
# Check what files have changed
git status

# Stage all changes
git add .

# Or stage specific files
git add path/to/file

# Commit with descriptive message
git commit -m "Description of your changes"

# Push to GitHub
git push origin main
```

**Use this when:**
- You've completed a feature or fix
- You want to back up your work
- You want to share changes with others

### 3. Full Sync (Both Ways)

```bash
# Pull first to get latest changes
git pull origin main

# Stage and commit your changes
git add .
git commit -m "Your commit message"

# Push your changes
git push origin main
```

**Use this when:**
- You're actively collaborating
- You want to ensure everything is up-to-date

---

## Best Practices

### 📋 Commit Messages

Write clear, descriptive commit messages:

**Good examples:**
```
Add hardware pass integration to Enterprise E²
Fix blockchain sync issue in p2p network layer
Update README with Boundless Trust information
Implement GeoSovereign compliance module
```

**Bad examples:**
```
fixes
update
changes
wip
```

### 🔄 Sync Frequency

**Recommended sync schedule:**

| Scenario | Pull Frequency | Push Frequency |
|----------|---------------|----------------|
| Solo development | Start of day | End of feature |
| Team development | Every 1-2 hours | After each commit |
| Critical fixes | Before starting | Immediately after fix |

### 📁 What to Commit

**DO commit:**
- ✅ Source code (.rs, .ts, .tsx files)
- ✅ Configuration files (.toml, .json, .yaml)
- ✅ Documentation (.md files)
- ✅ Database migrations (.sql files)
- ✅ Scripts (.sh, .bat, .ps1)
- ✅ Example files (.example)

**DON'T commit:**
- ❌ Build artifacts (target/, dist/, build/)
- ❌ Dependencies (node_modules/, cargo packages)
- ❌ Environment files (.env, .env.local)
- ❌ IDE settings (.vscode/, .idea/)
- ❌ Database files (*.db, *.sqlite)
- ❌ Log files (*.log)
- ❌ Temporary files (*.tmp, *.bak)

These are already excluded in `.gitignore`.

---

## Common Scenarios

### Scenario 1: Starting Your Day

```bash
# Pull latest changes
git pull origin main

# Check status
git status

# Start working...
```

### Scenario 2: After Making Changes

```bash
# Check what changed
git status

# Review changes
git diff

# Stage all changes
git add .

# Commit
git commit -m "Implement wallet transaction signing with PQC"

# Push to GitHub
git push origin main
```

### Scenario 3: Before Making a Pull Request

```bash
# Ensure you're on main branch
git branch --show-current

# Pull latest changes
git pull origin main

# Make sure everything works
cargo test  # For Rust code
npm test    # For frontend

# Push your final changes
git push origin main
```

### Scenario 4: Multiple Files Changed

```bash
# See all changes
git status

# Stage specific components
git add enterprise/src/services/wallet.rs
git add enterprise/src/api/wallet.rs
git commit -m "Add wallet balance caching"

git add rpc/src/server.rs
git commit -m "Add new RPC endpoint for proof verification"

# Push all commits
git push origin main
```

---

## Troubleshooting

### Issue: Merge Conflicts

**Symptoms:**
```
CONFLICT (content): Merge conflict in <filename>
Automatic merge failed; fix conflicts and then commit the result.
```

**Solution:**
```bash
# Open conflicted files in your editor
# Look for conflict markers:
# <<<<<<< HEAD
# Your changes
# =======
# Remote changes
# >>>>>>> origin/main

# Edit files to resolve conflicts
# Remove conflict markers
# Keep the correct code

# Stage resolved files
git add <resolved-file>

# Complete the merge
git commit -m "Resolve merge conflicts in <files>"

# Push
git push origin main
```

### Issue: Detached HEAD State

**Symptoms:**
```
You are in 'detached HEAD' state...
```

**Solution:**
```bash
# Return to main branch
git checkout main

# Pull latest
git pull origin main
```

### Issue: Push Rejected

**Symptoms:**
```
! [rejected]        main -> main (fetch first)
error: failed to push some refs
```

**Solution:**
```bash
# Pull first (remote has changes you don't have)
git pull origin main

# Then push
git push origin main
```

### Issue: Accidentally Committed Secrets

**Symptoms:**
You committed `.env` file or API keys

**Solution:**
```bash
# Remove file from git tracking but keep locally
git rm --cached .env

# Ensure .gitignore includes it
echo ".env" >> .gitignore

# Commit the removal
git commit -m "Remove sensitive environment file"

# Push
git push origin main

# IMPORTANT: Change any exposed secrets immediately!
```

### Issue: Want to Undo Last Commit

**Symptoms:**
You committed something wrong

**Solution:**
```bash
# Undo commit but keep changes
git reset --soft HEAD~1

# Make corrections
# Then commit again
git add .
git commit -m "Corrected commit message"
git push origin main
```

---

## Component-Specific Sync

### Blockchain Core (Boundless BLS)

**Key directories:**
- `core/`, `consensus/`, `crypto/`, `wasm-runtime/`
- `p2p/`, `rpc/`, `storage/`, `node/`

**Before syncing:**
```bash
# Ensure code compiles
cargo build --all

# Run tests
cargo test --all
```

### Enterprise E² Multipass

**Key directories:**
- `enterprise/src/`
- `enterprise/frontend/`
- `enterprise/migrations/`

**Before syncing:**
```bash
# Rust backend
cd enterprise
cargo build

# Frontend
cd enterprise/frontend
npm run build

# Database migrations (if any new ones)
sqlx migrate info
```

---

## Automated Sync (Advanced)

### Using Git Hooks

You can automate syncing with git hooks. Create `.git/hooks/pre-push`:

```bash
#!/bin/bash
# Run tests before pushing

echo "Running tests before push..."

# Test Rust code
cargo test --all || exit 1

# Test frontend (if npm is available)
if command -v npm &> /dev/null; then
    cd enterprise/frontend
    npm test || exit 1
fi

echo "All tests passed! Proceeding with push..."
```

Make it executable:
```bash
chmod +x .git/hooks/pre-push
```

---

## Quick Reference Card

```bash
# Daily workflow
git pull origin main        # Start: Get latest
# ... do work ...
git add .                   # Stage changes
git commit -m "Message"     # Commit with message
git push origin main        # Push to GitHub

# Check status
git status                  # See what changed
git log --oneline -5        # Recent commits
git diff                    # See exact changes

# Sync helpers
.\sync-github.bat          # Windows batch script
.\sync-github.ps1          # PowerShell script

# Branch info
git branch                  # List branches
git branch --show-current   # Current branch

# Remote info
git remote -v              # Show remote URLs
git fetch origin           # Fetch without merging
```

---

## Support

If you encounter issues with git sync:

1. Check this guide first
2. Try the sync scripts (they handle most common cases)
3. Review git status: `git status`
4. Check recent commits: `git log --oneline -5`
5. For serious issues, create a backup: `cp -r boundless-bls-platform boundless-bls-platform-backup`

**Need help?**
- GitHub Issues: https://github.com/Saifullah62/BLS/issues
- Email: contact@boundlesstrust.org

---

**Remember:** Commit early, commit often, sync regularly! 🚀
