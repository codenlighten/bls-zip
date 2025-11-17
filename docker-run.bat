@echo off
REM Boundless BLS Blockchain - Docker Quick Start Script for Windows
REM This script provides easy commands to manage the blockchain with Docker

setlocal enabledelayedexpansion

REM Colors for output
set "GREEN=[92m"
set "YELLOW=[93m"
set "RED=[91m"
set "BLUE=[94m"
set "NC=[0m"

echo %BLUE%========================================%NC%
echo %BLUE%  Boundless BLS Blockchain - Docker%NC%
echo %BLUE%========================================%NC%
echo.

REM Check if Docker is installed
where docker >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo %RED%Error: Docker is not installed or not in PATH%NC%
    echo Please install Docker Desktop from https://www.docker.com/products/docker-desktop
    exit /b 1
)

REM Check if Docker is running
docker info >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo %RED%Error: Docker is not running%NC%
    echo Please start Docker Desktop
    exit /b 1
)

if "%1"=="" goto :show_help

if /i "%1"=="build" goto :build
if /i "%1"=="run" goto :run
if /i "%1"=="dev" goto :dev
if /i "%1"=="network" goto :network
if /i "%1"=="stop" goto :stop
if /i "%1"=="clean" goto :clean
if /i "%1"=="logs" goto :logs
if /i "%1"=="cli" goto :cli
if /i "%1"=="help" goto :show_help
goto :show_help

:show_help
echo %YELLOW%Usage: docker-run.bat [command]%NC%
echo.
echo %BLUE%Commands:%NC%
echo   build     - Build the Docker image
echo   run       - Run a single node
echo   dev       - Run a single mining node (development mode)
echo   network   - Start 3-node network with Docker Compose
echo   stop      - Stop all running containers
echo   clean     - Remove all containers and volumes
echo   logs      - View logs from running nodes
echo   cli       - Run CLI commands
echo   help      - Show this help message
echo.
echo %BLUE%Examples:%NC%
echo   docker-run.bat build
echo   docker-run.bat dev
echo   docker-run.bat network
echo   docker-run.bat logs node1
echo   docker-run.bat cli keygen
goto :eof

:build
echo %YELLOW%Building Docker image...%NC%
docker build -t boundless-bls:latest .
if %ERRORLEVEL% equ 0 (
    echo %GREEN%Build successful!%NC%
) else (
    echo %RED%Build failed!%NC%
    exit /b 1
)
goto :eof

:run
echo %YELLOW%Starting single node...%NC%
docker run -d ^
  --name boundless-node ^
  -p 30333:30333 ^
  -p 9933:9933 ^
  -v boundless-data:/data ^
  boundless-bls:latest ^
  --base-path /data --rpc-external --rpc-cors all

if %ERRORLEVEL% equ 0 (
    echo %GREEN%Node started successfully!%NC%
    echo.
    echo %BLUE%Access RPC at: http://localhost:9933%NC%
    echo %BLUE%View logs: docker logs -f boundless-node%NC%
) else (
    echo %RED%Failed to start node!%NC%
    exit /b 1
)
goto :eof

:dev
echo %YELLOW%Starting development node with mining...%NC%
docker run -d ^
  --name boundless-dev-node ^
  -p 30333:30333 ^
  -p 9933:9933 ^
  -v boundless-dev-data:/data ^
  boundless-bls:latest ^
  --dev --mining --base-path /data --rpc-external --rpc-cors all

if %ERRORLEVEL% equ 0 (
    echo %GREEN%Development node started successfully!%NC%
    echo.
    echo %BLUE%Mining is active%NC%
    echo %BLUE%Access RPC at: http://localhost:9933%NC%
    echo %BLUE%View logs: docker logs -f boundless-dev-node%NC%
    echo.
    echo %YELLOW%Starting log stream (Ctrl+C to exit)...%NC%
    timeout /t 2 /nobreak >nul
    docker logs -f boundless-dev-node
) else (
    echo %RED%Failed to start development node!%NC%
    exit /b 1
)
goto :eof

:network
echo %YELLOW%Starting 3-node network with Docker Compose...%NC%
docker-compose up -d
if %ERRORLEVEL% equ 0 (
    echo %GREEN%Network started successfully!%NC%
    echo.
    echo %BLUE%Nodes:%NC%
    echo   Node 1 (Mining): http://localhost:9933
    echo   Node 2: http://localhost:9934
    echo   Node 3: http://localhost:9935
    echo.
    echo %BLUE%View logs: docker-compose logs -f node1%NC%
    echo %BLUE%Stop network: docker-compose down%NC%
) else (
    echo %RED%Failed to start network!%NC%
    exit /b 1
)
goto :eof

:stop
echo %YELLOW%Stopping all Boundless containers...%NC%
docker stop boundless-node boundless-dev-node 2>nul
docker-compose down 2>nul
echo %GREEN%All containers stopped!%NC%
goto :eof

:clean
echo %YELLOW%Removing all Boundless containers and volumes...%NC%
echo %RED%WARNING: This will delete all blockchain data!%NC%
set /p confirm="Are you sure? (yes/no): "
if /i not "%confirm%"=="yes" (
    echo %YELLOW%Operation cancelled%NC%
    goto :eof
)

docker rm -f boundless-node boundless-dev-node 2>nul
docker-compose down -v 2>nul
docker volume rm boundless-data boundless-dev-data 2>nul
echo %GREEN%Cleanup complete!%NC%
goto :eof

:logs
if "%2"=="" (
    echo %YELLOW%Showing logs for single node...%NC%
    docker logs -f boundless-node 2>nul
    if %ERRORLEVEL% neq 0 (
        docker logs -f boundless-dev-node 2>nul
        if %ERRORLEVEL% neq 0 (
            echo %RED%No running single node found%NC%
            echo %YELLOW%Try: docker-run.bat logs node1%NC%
        )
    )
) else (
    echo %YELLOW%Showing logs for %2...%NC%
    docker-compose logs -f %2
)
goto :eof

:cli
if "%2"=="" (
    docker run --rm boundless-bls:latest boundless-cli --help
) else (
    docker run --rm boundless-bls:latest boundless-cli %2 %3 %4 %5 %6 %7 %8 %9
)
goto :eof

:eof
endlocal
