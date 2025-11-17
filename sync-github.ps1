# Boundless BLS Platform - GitHub Sync Script (PowerShell)
# This script helps keep your local repository in sync with GitHub

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet('pull', 'push', 'sync', 'status')]
    [string]$Action = 'menu'
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Boundless BLS Platform - GitHub Sync" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if we're in a git repository
$gitCheck = git rev-parse --git-dir 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Not a git repository!" -ForegroundColor Red
    Write-Host "Please run this script from the boundless-bls-platform directory." -ForegroundColor Yellow
    exit 1
}

# Get current branch
$currentBranch = git branch --show-current
Write-Host "Current branch: " -NoNewline
Write-Host $currentBranch -ForegroundColor Green
Write-Host ""

function Show-Status {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Detailed Git Status" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "Branch status:" -ForegroundColor Yellow
    git status
    Write-Host ""

    Write-Host "Recent commits:" -ForegroundColor Yellow
    git log --oneline -5
    Write-Host ""

    Write-Host "Remote repositories:" -ForegroundColor Yellow
    git remote -v
    Write-Host ""
}

function Pull-Changes {
    Write-Host "Fetching latest changes from GitHub..." -ForegroundColor Yellow
    git fetch origin
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to fetch from remote!" -ForegroundColor Red
        return $false
    }

    Write-Host "Pulling changes from origin/$currentBranch..." -ForegroundColor Yellow
    git pull origin $currentBranch
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "WARNING: Merge conflicts detected or pull failed!" -ForegroundColor Red
        Write-Host "Please resolve conflicts manually." -ForegroundColor Yellow
        return $false
    }

    Write-Host ""
    Write-Host "✓ Successfully pulled changes from GitHub!" -ForegroundColor Green
    return $true
}

function Push-Changes {
    # Check for uncommitted changes
    git diff-index --quiet HEAD --
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "You have uncommitted changes:" -ForegroundColor Yellow
        git status --short
        Write-Host ""

        $commit = Read-Host "Would you like to commit these changes? (Y/N)"
        if ($commit -eq 'Y' -or $commit -eq 'y') {
            $commitMsg = Read-Host "Enter commit message"

            git add .
            git commit -m $commitMsg
            if ($LASTEXITCODE -ne 0) {
                Write-Host "ERROR: Commit failed!" -ForegroundColor Red
                return $false
            }
            Write-Host "✓ Changes committed" -ForegroundColor Green
        } else {
            Write-Host "Skipping commit. Uncommitted changes will not be pushed." -ForegroundColor Yellow
            return $false
        }
    }

    Write-Host ""
    Write-Host "Pushing changes to GitHub..." -ForegroundColor Yellow
    git push origin $currentBranch
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "ERROR: Push failed!" -ForegroundColor Red
        Write-Host "Remote may have changes you don't have locally. Try syncing first." -ForegroundColor Yellow
        return $false
    }

    Write-Host ""
    Write-Host "✓ Successfully pushed changes to GitHub!" -ForegroundColor Green
    return $true
}

function Sync-Repository {
    Write-Host "Performing full sync (pull + push)..." -ForegroundColor Yellow
    Write-Host ""

    # Pull first
    Write-Host "[1/2] Pulling latest changes..." -ForegroundColor Cyan
    if (-not (Pull-Changes)) {
        Write-Host "Sync aborted due to pull failure." -ForegroundColor Red
        return $false
    }

    # Then push
    Write-Host ""
    Write-Host "[2/2] Pushing local changes..." -ForegroundColor Cyan
    if (-not (Push-Changes)) {
        Write-Host "Push step skipped or failed." -ForegroundColor Yellow
        return $false
    }

    Write-Host ""
    Write-Host "✓ Full sync completed successfully!" -ForegroundColor Green
    return $true
}

# Execute based on action
switch ($Action) {
    'pull' {
        Pull-Changes
    }
    'push' {
        Push-Changes
    }
    'sync' {
        Sync-Repository
    }
    'status' {
        Show-Status
    }
    'menu' {
        # Show current status
        Write-Host "Current status:" -ForegroundColor Yellow
        git status --short
        Write-Host ""

        # Interactive menu
        Write-Host "What would you like to do?" -ForegroundColor Cyan
        Write-Host "[1] Pull latest changes from GitHub (update local)" -ForegroundColor White
        Write-Host "[2] Push local changes to GitHub (update remote)" -ForegroundColor White
        Write-Host "[3] Full sync (pull then push)" -ForegroundColor White
        Write-Host "[4] Show detailed status" -ForegroundColor White
        Write-Host "[5] Exit" -ForegroundColor White
        Write-Host ""

        $choice = Read-Host "Enter your choice (1-5)"
        Write-Host ""

        switch ($choice) {
            '1' { Pull-Changes }
            '2' { Push-Changes }
            '3' { Sync-Repository }
            '4' { Show-Status }
            '5' {
                Write-Host "Goodbye!" -ForegroundColor Cyan
                exit 0
            }
            default {
                Write-Host "Invalid choice!" -ForegroundColor Red
                exit 1
            }
        }
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Sync complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
