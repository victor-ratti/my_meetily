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

function Add-PathIfExists {
    param([string]$PathToAdd)

    if (-not $PathToAdd -or -not (Test-Path $PathToAdd)) {
        return
    }

    $PathParts = $env:Path -split ";"
    if ($PathParts -contains $PathToAdd) {
        return
    }

    $env:Path = "$PathToAdd;$env:Path"
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

    Add-PathIfExists $ExtraPath

    if (Test-CommandAvailable $CommandName) {
        return
    }

    if (-not $InstallPrereqs) {
        throw "$CommandName is not available. $ManualInstallHint`nRerun with .\run-local.cmd -InstallPrereqs to install it automatically with winget."
    }

    Install-WithWinget -PackageId $PackageId -PackageName $PackageName

    Add-PathIfExists $ExtraPath

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

function Get-VsDevCmdPath {
    $ProgramFilesX86 = ${env:ProgramFiles(x86)}
    if (-not $ProgramFilesX86) {
        return $null
    }

    $VsWhere = Join-Path $ProgramFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $VsWhere)) {
        return $null
    }

    $InstallPath = & $VsWhere `
        -latest `
        -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath

    if (-not $InstallPath) {
        $InstallPath = & $VsWhere `
            -latest `
            -products * `
            -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 `
            -property installationPath
    }

    if (-not $InstallPath) {
        return $null
    }

    $VsDevCmd = Join-Path $InstallPath "Common7\Tools\VsDevCmd.bat"
    if (Test-Path $VsDevCmd) {
        return $VsDevCmd
    }

    return $null
}

function Import-VsDevEnvironment {
    if ($env:VSCMD_VER) {
        return
    }

    $VsDevCmd = Get-VsDevCmdPath
    if (-not $VsDevCmd) {
        throw "Visual Studio developer environment was not found. Rerun .\run-local.cmd -InstallPrereqs, then restart PowerShell."
    }

    $Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "amd64" }
    Write-Step "Loading Visual Studio C++ build environment ($Arch)"

    $EnvironmentOutput = cmd.exe /s /c "`"$VsDevCmd`" -arch=$Arch -host_arch=$Arch >nul && set"
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to load Visual Studio developer environment."
    }

    foreach ($Line in $EnvironmentOutput) {
        $SeparatorIndex = $Line.IndexOf("=")
        if ($SeparatorIndex -le 0) {
            continue
        }

        $Name = $Line.Substring(0, $SeparatorIndex)
        $Value = $Line.Substring($SeparatorIndex + 1)
        [Environment]::SetEnvironmentVariable($Name, $Value, "Process")
    }
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

    Ensure-Command `
        -CommandName "clang" `
        -PackageId "LLVM.LLVM" `
        -PackageName "LLVM / Clang" `
        -ManualInstallHint "Install LLVM from https://llvm.org/ or install the Visual Studio Clang tools." `
        -ExtraPath (Join-Path $env:ProgramFiles "LLVM\bin")

    Import-VsDevEnvironment
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
