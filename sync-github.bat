@echo off
REM Boundless BLS Platform - GitHub Sync Script
REM This script helps keep your local repository in sync with GitHub

echo ========================================
echo Boundless BLS Platform - GitHub Sync
echo ========================================
echo.

REM Check if we're in a git repository
git rev-parse --git-dir >nul 2>&1
if errorlevel 1 (
    echo ERROR: Not a git repository!
    echo Please run this script from the boundless-bls-platform directory.
    pause
    exit /b 1
)

REM Show current branch
echo Current branch:
git branch --show-current
echo.

REM Fetch latest changes from GitHub
echo Fetching latest changes from GitHub...
git fetch origin
if errorlevel 1 (
    echo ERROR: Failed to fetch from remote!
    pause
    exit /b 1
)
echo.

REM Show status
echo Current status:
git status --short
echo.

REM Ask user what to do
echo What would you like to do?
echo [1] Pull latest changes from GitHub (update local)
echo [2] Push local changes to GitHub (update remote)
echo [3] Full sync (pull then push)
echo [4] Show detailed status
echo [5] Exit
echo.

choice /c 12345 /n /m "Enter your choice (1-5): "

if errorlevel 5 goto end
if errorlevel 4 goto status
if errorlevel 3 goto fullsync
if errorlevel 2 goto push
if errorlevel 1 goto pull

:pull
echo.
echo Pulling latest changes from GitHub...
git pull origin main
if errorlevel 1 (
    echo.
    echo WARNING: Merge conflicts detected or pull failed!
    echo Please resolve conflicts manually.
    pause
    exit /b 1
)
echo.
echo ✓ Successfully pulled changes from GitHub!
goto end

:push
echo.
echo Checking for uncommitted changes...
git diff-index --quiet HEAD --
if errorlevel 1 (
    echo.
    echo You have uncommitted changes. Showing status:
    git status
    echo.
    choice /c YN /n /m "Would you like to commit these changes? (Y/N): "
    if errorlevel 2 goto end

    echo.
    set /p commit_msg="Enter commit message: "

    git add .
    git commit -m "%commit_msg%"
    if errorlevel 1 (
        echo ERROR: Commit failed!
        pause
        exit /b 1
    )
)

echo.
echo Pushing changes to GitHub...
git push origin main
if errorlevel 1 (
    echo.
    echo ERROR: Push failed!
    echo This might be because remote has changes you don't have locally.
    echo Try running a full sync (option 3) instead.
    pause
    exit /b 1
)
echo.
echo ✓ Successfully pushed changes to GitHub!
goto end

:fullsync
echo.
echo Performing full sync (pull + push)...
echo.

REM First pull
echo [1/2] Pulling latest changes...
git pull origin main
if errorlevel 1 (
    echo.
    echo WARNING: Pull failed or conflicts detected!
    echo Please resolve conflicts before pushing.
    pause
    exit /b 1
)

REM Check for local changes
git diff-index --quiet HEAD --
if errorlevel 1 (
    echo.
    echo You have uncommitted changes. Showing status:
    git status
    echo.
    choice /c YN /n /m "Would you like to commit these changes? (Y/N): "
    if errorlevel 2 goto end

    echo.
    set /p commit_msg="Enter commit message: "

    git add .
    git commit -m "%commit_msg%"
    if errorlevel 1 (
        echo ERROR: Commit failed!
        pause
        exit /b 1
    )
)

REM Then push
echo.
echo [2/2] Pushing local changes...
git push origin main
if errorlevel 1 (
    echo ERROR: Push failed!
    pause
    exit /b 1
)

echo.
echo ✓ Full sync completed successfully!
goto end

:status
echo.
echo ========================================
echo Detailed Git Status
echo ========================================
echo.
echo Current branch:
git branch --show-current
echo.
echo Branch status:
git status
echo.
echo Recent commits:
git log --oneline -5
echo.
echo Remote status:
git remote -v
echo.
pause
goto end

:end
echo.
echo ========================================
echo Sync complete!
echo ========================================
pause
