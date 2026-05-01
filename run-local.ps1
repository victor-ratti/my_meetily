param(
    [ValidateSet("desktop", "web")]
    [string]$Mode = "desktop",

    [switch]$SkipInstall,
    [switch]$Cpu,
    [switch]$InstallPrereqs
)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$FrontendDir = Join-Path $RootDir "frontend"
$NodeModulesDir = Join-Path $FrontendDir "node_modules"
$NextBin = Join-Path $NodeModulesDir ".bin\next.cmd"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Test-CommandAvailable {
    param([string]$CommandName)

    return [bool](Get-Command $CommandName -ErrorAction SilentlyContinue)
}

function Install-WithWinget {
    param(
        [string]$PackageId,
        [string]$PackageName,
        [string]$Override
    )

    if (-not (Test-CommandAvailable "winget")) {
        throw "$PackageName is missing and winget is not available. Install it manually, then rerun this script."
    }

    Write-Step "Installing $PackageName with winget"
    if ($Override) {
        winget install --id $PackageId -e --source winget --override $Override
    } else {
        winget install --id $PackageId -e --source winget
    }
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install $PackageName with winget."
    }
}

function Ensure-Command {
    param(
        [string]$CommandName,
        [string]$PackageId,
        [string]$PackageName,
        [string]$ManualInstallHint,
        [string]$ExtraPath
    )

    if (Test-CommandAvailable $CommandName) {
        return
    }

    if (-not $InstallPrereqs) {
        throw "$CommandName is not available. $ManualInstallHint`nRerun with .\run-local.cmd -InstallPrereqs to install it automatically with winget."
    }

    Install-WithWinget -PackageId $PackageId -PackageName $PackageName

    if ($ExtraPath -and (Test-Path $ExtraPath)) {
        $env:Path = "$ExtraPath;$env:Path"
    }

    if (-not (Test-CommandAvailable $CommandName)) {
        throw "$PackageName was installed, but $CommandName is still not available in this terminal. Restart your terminal, then rerun .\run-local.cmd."
    }
}

function Test-MsvcBuildTools {
    $ProgramFilesX86 = ${env:ProgramFiles(x86)}
    if (-not $ProgramFilesX86) {
        return $false
    }

    $VsWhere = Join-Path $ProgramFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $VsWhere)) {
        return $false
    }

    $InstallPath = & $VsWhere `
        -latest `
        -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath

    if ($InstallPath) {
        return $true
    }

    $Arm64InstallPath = & $VsWhere `
        -latest `
        -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 `
        -property installationPath

    return [bool]$Arm64InstallPath
}

function Ensure-MsvcBuildTools {
    if (Test-MsvcBuildTools) {
        return
    }

    $ManualInstallHint = "Install Visual Studio 2022 Build Tools with the Desktop development with C++ workload."
    if (-not $InstallPrereqs) {
        throw "MSVC C++ build tools are not available. $ManualInstallHint`nRerun with .\run-local.cmd -InstallPrereqs to install them automatically with winget."
    }

    Install-WithWinget `
        -PackageId "Microsoft.VisualStudio.2022.BuildTools" `
        -PackageName "Visual Studio 2022 Build Tools / MSVC C++" `
        -Override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --includeRecommended"

    if (-not (Test-MsvcBuildTools)) {
        throw "Visual Studio Build Tools were installed, but MSVC was not detected in this terminal. Restart your terminal, then rerun .\run-local.cmd."
    }
}

if (-not (Test-Path $FrontendDir)) {
    throw "frontend directory not found at $FrontendDir"
}

Write-Step "Checking prerequisites"
Ensure-Command `
    -CommandName "node" `
    -PackageId "OpenJS.NodeJS.LTS" `
    -PackageName "Node.js LTS" `
    -ManualInstallHint "Install Node.js LTS from https://nodejs.org/" `
    -ExtraPath (Join-Path $env:ProgramFiles "nodejs")

Ensure-Command `
    -CommandName "npm" `
    -PackageId "OpenJS.NodeJS.LTS" `
    -PackageName "Node.js LTS" `
    -ManualInstallHint "Install Node.js LTS from https://nodejs.org/" `
    -ExtraPath (Join-Path $env:ProgramFiles "nodejs")

if ($Mode -eq "desktop") {
    Ensure-Command `
        -CommandName "cargo" `
        -PackageId "Rustlang.Rustup" `
        -PackageName "Rustup / Cargo" `
        -ManualInstallHint "Install Rust with rustup from https://rustup.rs/, then restart your terminal." `
        -ExtraPath (Join-Path $env:USERPROFILE ".cargo\bin")

    Ensure-MsvcBuildTools
}

Push-Location $FrontendDir
try {
    if (-not $SkipInstall) {
        if (-not (Test-Path $NodeModulesDir) -or -not (Test-Path $NextBin)) {
            Write-Step "Installing frontend dependencies"
            npm install
        } else {
            Write-Step "Frontend dependencies already installed"
        }
    }

    if ($Mode -eq "web") {
        Write-Step "Starting Next.js dev server on http://localhost:3118"
        npm run dev
        exit $LASTEXITCODE
    }

    if ($Cpu) {
        Write-Step "Starting Meetily desktop app with Tauri in CPU mode"
        npm run tauri:dev:cpu
        exit $LASTEXITCODE
    }

    Write-Step "Starting Meetily desktop app with Tauri auto-detection"
    npm run tauri:dev
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
