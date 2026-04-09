# HyperCoWork Rust -- Windows Installer
#
# Automates full setup: Rust, llama.cpp, HyperCoWork, models
# Run in PowerShell as Administrator:
#   irm https://raw.githubusercontent.com/multidimensionalinteractive/HyperCowork/main/install.ps1 | iex
#
# Or download and run:
#   .\install.ps1

param(
    [string]$InstallDir = "$env:USERPROFILE\hypercowork",
    [switch]$SkipRust,
    [switch]$SkipLlama,
    [switch]$SkipModels,
    [switch]$Cuda,
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$VERSION = "0.1.0"
$REPO = "multidimensionalinteractive/HyperCowork"
$LLAMA_CPP_REPO = "ggml-org/llama.cpp"

# Colors for output
function Write-Color($Text, $Color = "White") {
    Write-Host $Text -ForegroundColor $Color
}

function Write-Header($Text) {
    Write-Host ""
    Write-Host "  +==============================================+" -ForegroundColor DarkCyan
    Write-Host "  |  $Text" -ForegroundColor Cyan
    Write-Host "  +==============================================+" -ForegroundColor DarkCyan
    Write-Host ""
}

function Write-Step($Text) {
    Write-Host "  > " -ForegroundColor DarkGray -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Write-Done($Text) {
    Write-Host "  > " -ForegroundColor Green -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Write-Skip($Text) {
    Write-Host "  o " -ForegroundColor Yellow -NoNewline
    Write-Host $Text -ForegroundColor Gray
}

function Write-Error($Text) {
    Write-Host "  X " -ForegroundColor Red -NoNewline
    Write-Host $Text -ForegroundColor White
}

function Test-Command($Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Test-NvidiaGpu() {
    try {
        $gpu = & nvidia-smi --query-gpu=name --format=csv,noheader 2>$null
        return [bool]$gpu
    } catch {
        return $false
    }
}

# Progress spinner for long operations
$spinnerChars = @("*", "*", "*", "*", "*", "*", "*", "*", "*", "*")
$spinnerIndex = 0
$spinnerActive = $false

function Start-Spinner {
    $script:spinnerActive = $true
    $script:spinnerIndex = 0
}

function Update-Spinner($Text) {
    if ($script:spinnerActive) {
        Write-Host "`r  $($spinnerChars[$script:spinnerIndex]) $Text" -ForegroundColor Cyan -NoNewline
        $script:spinnerIndex = ($script:spinnerIndex + 1) % $spinnerChars.Length
    }
}

function Stop-Spinner {
    $script:spinnerActive = $false
    Write-Host ""
}

# --- ASCII Banner ---
function Show-Banner {
    Write-Host ""
    Write-Host "  +==========================================================+" -ForegroundColor Cyan
    Write-Host "  |   HYPER - AGENT - COWORK - FLEET                      |" -ForegroundColor Cyan
    Write-Host "  |          CONTROL SYSTEM                               |" -ForegroundColor Cyan
    Write-Host "  +==========================================================+" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  Version $VERSION" -ForegroundColor Gray
    Write-Host ""
}

# --- Help ---
if ($Help) {
    Show-Banner
    Write-Host @"
  Usage:
    .\install.ps1                    Install everything
    .\install.ps1 -SkipRust         Skip Rust installation
    .\install.ps1 -SkipLlama         Skip llama.cpp build
    .\install.ps1 -SkipModels        Skip model download
    .\install.ps1 -Cuda              Build llama.cpp with CUDA support
    .\install.ps1 -Help              Show this help

  Install location: $InstallDir

  Requirements:
    - Windows 10/11 (PowerShell 5.1+)
    - Git
    - Visual Studio Build Tools (for Rust)
    - 10GB+ free disk space

"@ -ForegroundColor White
    exit 0
}

# --- Show Banner ---
Show-Banner

Write-Header "SYSTEM CHECK"

# Check admin (for certain operations)
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")

# Check PowerShell version
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-Error "PowerShell 5.1+ required. Current: $($PSVersionTable.PSVersion)"
    exit 1
}
Write-Done "PowerShell $($PSVersionTable.PSVersion)"

# Check Windows
if ($env:OS -ne "Windows_NT") {
    Write-Error "Windows required"
    exit 1
}
Write-Done "Windows"

# Check admin status
if ($isAdmin) {
    Write-Step "Running as Administrator"
} else {
    Write-Step "Not running as Administrator (some features may fail)"
}

# --- Detect Missing Tools ---
$missingTools = @()

Write-Header "DEPENDENCIES"

if (-not (Test-Command git)) {
    Write-Step "Git not found - will install"
    $missingTools += "git"
} else {
    $gitVer = & git --version 2>$null
    Write-Done "Git: $gitVer"
}

if (-not (Test-Command cargo)) {
    Write-Step "Rust/Cargo not found - will install"
    $missingTools += "rust"
} else {
    $cargoVer = & cargo --version 2>$null
    Write-Done "Cargo: $cargoVer"
}

if (-not (Test-Command bun)) {
    Write-Step "Bun not found - will install"
    $missingTools += "bun"
} else {
    $bunVer = & bun --version 2>$null
    Write-Done "Bun: $bunVer"
}

if ((Test-Command vcxsrv)) {
    Write-Done "VS Build Tools: Found"
} else {
    Write-Step "VS Build Tools not found - will install"
    $missingTools += "vsbuild"
}

if ((Test-NvidiaGpu) -and $Cuda) {
    Write-Done "NVIDIA GPU detected - CUDA build enabled"
} elseif ($Cuda) {
    Write-Error "CUDA requested but no NVIDIA GPU found"
    exit 1
}

# --- Auto-Install Missing Dependencies ---
if ($missingTools -contains "git") {
    Write-Header "INSTALLING GIT"
    Write-Step "Downloading Git for Windows..."
    $gitInstaller = "$env:TEMP\git-installer.exe"
    Start-Spinner
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://github.com/git-for-windows/git/releases/download/v2.45.1.windows.1/Git-2.45.1-64-bit.exe" -OutFile $gitInstaller -UseBasicParsing
        Stop-Spinner
        Write-Done "Downloaded Git installer"
        Write-Step "Running installer..."
        Start-Process -Wait -FilePath $gitInstaller -ArgumentList "/SILENT", "/NORESTART", "/NOCANCEL"
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
        Write-Done "Git installed"
    } catch {
        Stop-Spinner
        Write-Error "Failed to install Git: $_"
        exit 1
    }
}

if ($missingTools -contains "rust") {
    Write-Header "INSTALLING RUST"
    Write-Step "Downloading rustup..."
    $rustupInstaller = "$env:TEMP\rustup-init.exe"
    Start-Spinner
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://win.rustup.rs" -OutFile $rustupInstaller -UseBasicParsing
        Stop-Spinner
        Write-Done "Downloaded rustup"
        Write-Step "Running installer (default profile)..."
        Start-Process -Wait -FilePath $rustupInstaller -ArgumentList "-y", "--default-toolchain", "stable"
        & "$env:USERPROFILE\.cargo\env.ps1" 2>$null
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
        Write-Done "Rust installed"
    } catch {
        Stop-Spinner
        Write-Error "Failed to install Rust: $_"
        exit 1
    }
}

if ($missingTools -contains "bun") {
    Write-Header "INSTALLING BUN"
    Write-Step "Downloading Bun..."
    $bunInstaller = "$env:TEMP\bun-installer.ps1"
    Start-Spinner
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://bun.sh/install.ps1" -OutFile $bunInstaller -UseBasicParsing
        Stop-Spinner
        Write-Done "Downloaded installer"
        Write-Step "Running Bun installer..."
        & $bunInstaller -Path "$env:LOCALAPPDATA\bun"
        $env:Path = "$env:LOCALAPPDATA\bun\bin;$env:Path"
        Write-Done "Bun installed"
    } catch {
        Stop-Spinner
        Write-Error "Failed to install Bun: $_"
        exit 1
    }
}

if ($missingTools -contains "vsbuild") {
    Write-Header "INSTALLING VS BUILD TOOLS"
    Write-Step "Downloading VS Build Tools installer..."
    $vsInstaller = "$env:TEMP\vs_buildtools.exe"
    Start-Spinner
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $vsInstaller -UseBasicParsing
        Stop-Spinner
        Write-Done "Downloaded VS Build Tools"
        Write-Step "Installing (this takes 10-20 minutes)..."
        Write-Host "  > NOTE: You may see a GUI window for VS Build Tools" -ForegroundColor Yellow
        Start-Process -Wait -FilePath $vsInstaller -ArgumentList "--quiet", "--wait", "--norestart", "--nocache", "--add", "Microsoft.VisualStudio.Workload.VCTools", "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64", "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22000"
        Write-Done "VS Build Tools installed"
    } catch {
        Stop-Spinner
        Write-Error "Failed to install VS Build Tools: $_"
        exit 1
    }
}

# --- Create Install Directory ---
Write-Header "SETUP"
Write-Step "Creating $InstallDir"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path "$InstallDir\bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$InstallDir\config" | Out-Null
New-Item -ItemType Directory -Force -Path "$InstallDir\models" | Out-Null
Write-Done "Directories created"

# --- Clone Repository ---
Write-Header "CLONING REPO"
$repoDir = "$InstallDir\source"
if (Test-Path $repoDir) {
    Write-Step "Updating existing source..."
    try {
        Push-Location $repoDir
        & git pull --rebase 2>$null
        Pop-Location
        Write-Done "Updated"
    } catch {
        Write-Skip "Update failed, using existing"
    }
} else {
    Write-Step "Cloning $REPO..."
    Start-Spinner
    try {
        & git clone --depth 1 "https://github.com/$REPO.git" $repoDir 2>&1 | Out-Null
        Stop-Spinner
        Write-Done "Cloned"
    } catch {
        Stop-Spinner
        Write-Error "Failed to clone: $_"
        exit 1
    }
}

# --- Build ---
Write-Header "BUILDING"
$buildStart = Get-Date

Write-Host "  Compiling crates..." -ForegroundColor Cyan
Write-Host "  > This will take 5-15 minutes on first build" -ForegroundColor DarkGray

$env:CARGO_TERM_PROGRESS_WIDTH = 60
& cargo build --release -p hypercowork-server 2>&1 | ForEach-Object {
    if ($_ -match "Compiling (\S+)") {
        Write-Host "  ~ Compiling $($matches[1])..." -ForegroundColor DarkGray -NoNewline
        Write-Host "`r" -NoNewline
    }
    if ($_ -match "error\[E\d+\]") {
        Write-Host "" -ForegroundColor Red
        Write-Host $_ -ForegroundColor Red
    }
}
Write-Done "Server built"

Write-Step "Building router..."
& cargo build --release -p hypercowork-router 2>&1 | Out-Null
Write-Done "Router built"

$buildElapsed = (Get-Date) - $buildStart
Write-Host "  Build time: $($buildElapsed.Minutes)m $($buildElapsed.Seconds)s" -ForegroundColor DarkGray

# --- Install Binaries ---
Write-Header "INSTALLING"

Write-Step "Copying binaries..."
$buildDir = "$repoDir\target\release"
Copy-Item "$buildDir\hypercowork-server.exe" "$InstallDir\bin\" -Force
Copy-Item "$buildDir\hypercowork-router.exe" "$InstallDir\bin\" -Force
Write-Done "Binaries installed"

# --- Create Config Files ---
Write-Header "CONFIGURATION"

# Server config - using single quotes so backslash and quotes are literal
$serverConfig = @'
# HyperCoWork Server Config
# Generated by Windows installer v$VERSION

host = "127.0.0.1"
port = 9876

# Approval mode: auto, manual, timeout
approval_mode = "timeout"
approval_timeout_secs = 30

# Authorized workspace roots (add your project dirs)
# workspaces = ["C:\Users\$env:USERNAME\projects"]

# CORS (empty = allow all for local use)
cors_origins = []
'@

$configPath = "$InstallDir\config\server.toml"
if (-not (Test-Path $configPath)) {
    $serverConfig | Out-File -FilePath $configPath -Encoding UTF8
    Write-Done "Server config > $configPath"
} else {
    Write-Skip "Config exists: $configPath"
}

# Router config - using single quotes so backslash and quotes are literal
$routerConfig = @'
# HyperCoWork Router Config

[[telegram]]
id = "main"
# token = "YOUR_BOT_TOKEN_HERE"

[router]
opencode_url = "http://localhost:9876"
dedup_window_secs = 30
'@

$routerConfigPath = "$InstallDir\config\router.toml"
if (-not (Test-Path $routerConfigPath)) {
    $routerConfig | Out-File -FilePath $routerConfigPath -Encoding UTF8
    Write-Done "Router config > $routerConfigPath"
} else {
    Write-Skip "Config exists: $routerConfigPath"
}

# --- Create Launchers ---
Write-Header "SHORTCUTS"

# Start server batch file - using single quotes for literal content
$startServer = @'
@echo off
echo.
echo   [R] Starting HyperCoWork Server...
echo.
cd /d "$InstallDir"
"$InstallDir\bin\hypercowork-server.exe" --workspace "%cd%" --config "$InstallDir\config\server.toml"
pause
'@
$startServerPath = "$InstallDir\Start Server.bat"
$startServer | Out-File -FilePath $startServerPath -Encoding ASCII
Write-Done "Start Server > $startServerPath"

# Start llama server batch file
$startLlama = @'
@echo off
echo.
echo   [L] Starting llama.cpp server...
echo.

set MODEL=%1
if "%MODEL%"=="" set MODEL="$InstallDir\models\Qwen2.5-7B-Instruct-Q4_K_M.gguf"

echo   Model: %MODEL%
echo   Port:  8080
echo.

"$InstallDir\llama.cpp\build\bin\Release\llama-server.exe" -m "%MODEL%" --host 127.0.0.1 --port 8080 -ngl 99 -c 32768 --chat-template chatml
pause
'@
$startLlamaPath = "$InstallDir\Start llama-server.bat"
$startLlama | Out-File -FilePath $startLlamaPath -Encoding ASCII
Write-Done "Start llama-server > $startLlamaPath"

# List models batch file
$listModels = @'
@echo off
echo.
echo   Available models in $InstallDir\models\
echo.
dir /b "$InstallDir\models\*.gguf" 2>nul
if errorlevel 1 echo   No models downloaded yet.
echo.
echo   Usage: "Start llama-server.bat" path\to\model.gguf
echo.
pause
'@
$listModelsPath = "$InstallDir\List Models.bat"
$listModels | Out-File -FilePath $listModelsPath -Encoding ASCII
Write-Done "List Models > $listModelsPath"

# Create hypercowork.bat launcher in bin (for PATH access)
$hypercoworkBat = @'
@echo off
echo.
echo   Starting HyperCoWork Server...
echo.
cd /d "$InstallDir\bin"
start cmd /k "hypercowork-server.exe"
'@
$hypercoworkBatPath = "$InstallDir\bin\hypercowork.bat"
$hypercoworkBat | Out-File -FilePath $hypercoworkBatPath -Encoding ASCII
Write-Done "Created hypercowork command in bin"

# Open frontend batch file
$startFrontend = @'
@echo off
echo.
echo   > Starting HyperCoWork Frontend...
echo.
cd /d "$InstallDir\source\apps\frontend"
bun install
bun dev
'@
$startFrontendPath = "$InstallDir\Start Frontend.bat"
$startFrontend | Out-File -FilePath $startFrontendPath -Encoding ASCII
Write-Done "Start Frontend > $startFrontendPath"

# Create desktop shortcut
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:USERPROFILE\Desktop\HyperCoWork Server.lnk")
$Shortcut.TargetPath = $startServerPath
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.IconLocation = "shell32.dll,44"
$Shortcut.Description = "Start HyperCoWork Rust Server"
$Shortcut.Save()
Write-Done "Desktop shortcut created"

# --- Summary ---
Write-Header "DONE"
Write-Host @"

  +===========================================================+
  |  Installation complete!                                    |
  +===========================================================+

  Install location: $InstallDir

  What's installed:
    > hypercowork-server.exe   (main server)
    > hypercowork-router.exe    (router)
    > Config files              (in config/)
    > Shortcut files           (in $InstallDir)

  To run:
    1. Download a model to models/
    2. Run Start Server.bat to start HyperCoWork
    3. Open http://localhost:3000 for web UI

  Or use the desktop shortcut.

"@ -ForegroundColor White

# --- Optional: Download Model ---
if (-not $SkipModels) {
    Write-Header "DOWNLOAD MODEL"
    Write-Host "  NOTE: Model download is optional and can be done later" -ForegroundColor DarkGray
    Write-Host "  Supported: Qwen2.5-7B-Instruct-Q4_K_M.gguf (~4GB)" -ForegroundColor DarkGray
    Write-Host "  Or any GGUF model from HuggingFace" -ForegroundColor DarkGray

    $downloadModel = Read-Host "  Download Qwen2.5-7B-Instruct-Q4_K_M now? [y/N]"
    if ($downloadModel -eq "y") {
        Write-Step "Downloading model (~4GB, will take a while)..."
        $modelUrl = "https://huggingface.co/bartowski/Qwen2.5-7B-Instruct-GGUF/resolve/main/Qwen2.5-7B-Instruct-Q4_K_M.gguf"
        $modelPath = "$InstallDir\models\Qwen2.5-7B-Instruct-Q4_K_M.gguf"
        Start-Spinner
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing
            Stop-Spinner
            Write-Done "Model downloaded to $modelPath"
        } catch {
            Stop-Spinner
            Write-Warning "Download failed: $_"
            Write-Host "  > Manually download from: $modelUrl" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "Press any key to exit..." -ForegroundColor DarkGray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")