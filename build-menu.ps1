<#
Qobuz Player - Build Helper (PowerShell)

Usage (PowerShell):
  cd C:\GIT_REPOS\Qobuz-Player
  .\build-menu.ps1

Features:
- Show current version (from src-tauri/Cargo.toml or fallback to src-tauri/tauri.conf.json)
- Check for missing dependencies (rustc, cargo, cargo tauri, WiX/candle)
- Run dev mode (cargo tauri dev)
- Build release (cargo tauri build) and open installer folder
- Set version (validates semantic version and updates both Cargo.toml and tauri.conf.json)

Note: this script uses .NET System.Version for comparisons and PowerShell's text replacement to update files.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Push-Location $PSScriptRoot

function Get-CurrentVersion {
    $cargo = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\Cargo.toml'
    if (Test-Path $cargo) {
        $ct = Get-Content $cargo -Raw
        if ($ct -match 'version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"') { return $Matches[1] }
    }
    $tauri = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\tauri.conf.json'
    if (Test-Path $tauri) {
        $tj = Get-Content $tauri -Raw
        if ($tj -match '"version"\s*:\s*"([0-9]+\.[0-9]+\.[0-9]+)"') { return $Matches[1] }
    }
    return '0.0.0'
}

function Test-Dependency($exe, $name, $checkCommand) {
    $cmd = Get-Command $exe -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host "[OK] $name found: $exe" -ForegroundColor Green
        if ($checkCommand) {
            try { & $checkCommand } catch { }
        }
        return $true
    }
    else {
        Write-Host "[MISSING] $name NOT found: $exe" -ForegroundColor Yellow
        return $false
    }
}

function Test-TauriCLI {
    try {
        & cmd /c "cargo tauri --version" > $null
        Write-Host "[OK] Tauri CLI found: cargo tauri" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "[MISSING] Tauri CLI NOT found: cargo tauri" -ForegroundColor Yellow
        Write-Host "Install via: cargo install tauri-cli or see https://tauri.app/"
        return $false
    }
}

function Test-WiX {
    $candle = Get-Command candle -ErrorAction SilentlyContinue
    if ($candle) {
        Write-Host "[OK] WiX Toolset found: candle.exe" -ForegroundColor Green
        return $true
    }
    else {
        Write-Host "[MISSING] WiX Toolset NOT found: candle.exe" -ForegroundColor Yellow
        Write-Host "Download: https://github.com/wixtoolset/wix/releases/"
        return $false
    }
}

function Set-Version {
    $curr = Get-CurrentVersion
    Write-Host "Current project version: $curr"

    try {
        $cv = [Version]$curr
    }
    catch {
        Write-Host "Current version string is invalid: $curr" -ForegroundColor Red
        Pause; return
    }

    # compute suggested next patch
    $suggest = "$($cv.Major).$($cv.Minor).$($cv.Build + 1)"
    $newver = Read-Host "Enter new version [$suggest]"
    if ([string]::IsNullOrWhiteSpace($newver)) { $newver = $suggest }

    try {
        $nv = [Version]$newver
    }
    catch {
        Write-Host 'Invalid version format. Use MAJOR.MINOR.PATCH (e.g. 1.2.3)' -ForegroundColor Red
        Pause; return
    }

    if ($nv -le $cv) {
        Write-Host "New version must be higher than current version ($curr)!" -ForegroundColor Red
        Pause; return
    }

    # Update Cargo.toml: only change the package version (the version that follows the name = "qobuz-player" line)
    $cargo = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\Cargo.toml'
    if (Test-Path $cargo) {
        try {
            $lines = Get-Content $cargo
            $idx = $null
            for ($i = 0; $i -lt $lines.Count; $i++) {
                if ($lines[$i] -match '^[ \t]*name\s*=\s*"qobuz-player"\s*$') { $idx = $i; break }
            }

            if ($null -ne $idx) {
                # find the first version = "..." after the name line
                $replaced = $false
                for ($j = $idx + 1; $j -lt $lines.Count; $j++) {
                    if ($lines[$j] -match '^[ \t]*version\s*=\s*"[^"]+"\s*$') {
                        $lines[$j] = $lines[$j] -replace '^[ \t]*version\s*=\s*"[^"]+"\s*$', ('version = "' + $newver + '"')
                        $replaced = $true
                        break
                    }
                }
                if (-not $replaced) {
                    # fallback: replace first version in file
                    for ($k = 0; $k -lt $lines.Count; $k++) {
                        if ($lines[$k] -match '^[ \t]*version\s*=\s*"[^"]+"\s*$') { $lines[$k] = ('version = "' + $newver + '"'); $replaced = $true; break }
                    }
                }

                if ($replaced) { Set-Content -Path $cargo -Value $lines } else { Write-Host "No version line found to update in $cargo" -ForegroundColor Yellow }
            }
            else {
                Write-Host "Could not find name = \"qobuz-player\" in $cargo; skipping targeted update" -ForegroundColor Yellow
                # As a conservative fallback, do not edit file automatically
            }
        }
        catch {
            Write-Host ("Failed to update {0}: {1}" -f $cargo, $_) -ForegroundColor Red
            Pause; return
        }
    }
    else {
        Write-Host "Warning: $cargo not found; skipping Cargo.toml update" -ForegroundColor Yellow
    }

    # Update tauri.conf.json: only change the version that follows the productName = "qobuz-player" anchor
    $tauri = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\tauri.conf.json'
    if (Test-Path $tauri) {
        try {
            $tlines = Get-Content $tauri
            $pidx = $null
            for ($i = 0; $i -lt $tlines.Count; $i++) {
                if ($tlines[$i] -match '"productName"\s*:\s*"qobuz-player"') { $pidx = $i; break }
            }

            if ($null -ne $pidx) {
                $replaced = $false
                for ($j = $pidx + 1; $j -lt $tlines.Count; $j++) {
                    if ($tlines[$j] -match '"version"\s*:\s*"[^"]+"') {
                        $tlines[$j] = $tlines[$j] -replace '"version"\s*:\s*"[^"]+"', ('"version": "' + $newver + '"')
                        $replaced = $true
                        break
                    }
                }
                if (-not $replaced) {
                    Write-Host "Could not find a version field after productName in $tauri; no change made" -ForegroundColor Yellow
                }
                else { Set-Content -Path $tauri -Value $tlines }
            }
            else {
                Write-Host "Could not find \"productName\": \"qobuz-player\" in $tauri; skipping targeted update" -ForegroundColor Yellow
            }
        }
        catch {
            Write-Host ("Failed to update {0}: {1}" -f $tauri, $_) -ForegroundColor Red
            Pause; return
        }
    }
    else {
        Write-Host "Warning: $tauri not found; skipping tauri.conf.json update" -ForegroundColor Yellow
    }

    Write-Host "Version updated to $newver in Cargo.toml and tauri.conf.json" -ForegroundColor Green
    Pause
}

function Test-VersionChanged {
    $versionFile = Join-Path -Path $PSScriptRoot -ChildPath '.last-build-version'
    $currentVersion = Get-CurrentVersion
    
    if (Test-Path $versionFile) {
        $lastVersion = Get-Content $versionFile -Raw
        $lastVersion = $lastVersion.Trim()
        
        if ($lastVersion -eq $currentVersion) {
            Write-Host "Current version ($currentVersion) has not been incremented since last build." -ForegroundColor Yellow
            $response = Read-Host "Do you want to set a new version now? (Y/N)"
            if ($response -match '^[Yy]') {
                Set-Version
                return $true
            }
            else {
                $continue = Read-Host "Continue with current version? (Y/N)"
                if ($continue -notmatch '^[Yy]') {
                    Write-Host "Operation cancelled." -ForegroundColor Yellow
                    Pause
                    return $false
                }
            }
        }
    }
    
    return $true
}

function Save-BuildVersion {
    $versionFile = Join-Path -Path $PSScriptRoot -ChildPath '.last-build-version'
    $currentVersion = Get-CurrentVersion
    Set-Content -Path $versionFile -Value $currentVersion
}

function Start-DevMode {
    if (-not (Test-VersionChanged)) { return }
    
    Write-Host 'Running: cargo tauri dev' -ForegroundColor Cyan
    & cmd /c "cargo tauri dev"
    Save-BuildVersion
    Pause
}

function Build-Release {
    if (-not (Test-VersionChanged)) { return }
    
    Write-Host 'Building: cargo tauri build' -ForegroundColor Cyan
    & cmd /c "cargo tauri build"
    Save-BuildVersion
    
    $msiFolder = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\target\release\bundle\msi'
    if (Test-Path $msiFolder) {
        Write-Host "Opening installer folder: $msiFolder"
        Start-Process explorer -ArgumentList $msiFolder
    }
    else {
        Write-Host 'Installer folder not found.' -ForegroundColor Yellow
    }
    Pause
}

function Show-Menu {
    Clear-Host
    $curr = Get-CurrentVersion
    Write-Host '==============================='
    Write-Host 'Qobuz Player Build Menu'
    Write-Host '==============================='
    Write-Host "Current version: $curr"
    Write-Host ''
    Write-Host '1 - Check for missing dependencies'
    Write-Host '2 - Run the app in development mode (cargo tauri dev)'
    Write-Host '3 - Build the app in release mode (cargo tauri build)'
    Write-Host '4 - Open installer folder (src-tauri/target/release/bundle/msi)'
    Write-Host '5 - Set version (update Cargo.toml and tauri.conf.json)'
    Write-Host '0 - Exit'
}

while ($true) {
    Show-Menu
    $choice = Read-Host 'Select an option'
    switch ($choice) {
        '1' {
            # Pipe to Out-Null to avoid printing the boolean return value (True/False)
            Test-Dependency -exe 'rustc' -name 'Rust compiler' -checkCommand 'rustc --version' | Out-Null
            Test-Dependency -exe 'cargo' -name 'Cargo package manager' -checkCommand 'cargo --version' | Out-Null
            Test-TauriCLI | Out-Null
            Test-WiX | Out-Null
            Pause
        }
    '2' { Start-DevMode }
        '3' { Build-Release }
        '4' {
            $msiFolder = Join-Path -Path $PSScriptRoot -ChildPath 'src-tauri\target\release\bundle\msi'
            if (Test-Path $msiFolder) { Start-Process explorer -ArgumentList $msiFolder } else { Write-Host 'Installer folder not found.' -ForegroundColor Yellow }
            Pause
        }
        '5' { Set-Version }
        '0' { Pop-Location; exit 0 }
        default { Write-Host 'Invalid option.'; Pause }
    }
}
