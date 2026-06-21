# === Rust binary build check ===
$root = $PSScriptRoot
$binPath = Join-Path $root "target\release\codex-local-launcher.exe"
$srcFiles = Get-ChildItem -Recurse -File (Join-Path $root "src") -ErrorAction SilentlyContinue
$rustNeedsBuild = $false

if (-not (Test-Path $binPath)) {
    Write-Host "Release binary not found, building..." -ForegroundColor Yellow
    $rustNeedsBuild = $true
} else {
    $binTime = (Get-Item $binPath).LastWriteTime
    foreach ($src in $srcFiles) {
        if ($src.LastWriteTime -gt $binTime) {
            Write-Host "Rust source $($src.Name) is newer than binary, rebuilding..." -ForegroundColor Yellow
            $rustNeedsBuild = $true
            break
        }
    }
}

if ($rustNeedsBuild) {
    cargo build --release --bin codex-local-launcher
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Rust build failed!" -ForegroundColor Red
        exit 1
    }
    Write-Host "Rust binary built successfully." -ForegroundColor Green
} else {
    Write-Host "Rust binary is up to date." -ForegroundColor Gray
}

# === Web UI build check ===
$srcDir = Join-Path $root "web\src"
$distDir = Join-Path $root "web\dist"
$distBundle = Join-Path $distDir "assets\index.js"
$webFiles = Get-ChildItem -Recurse -File $srcDir -ErrorAction SilentlyContinue
$webNeedsBuild = $false

if (-not (Test-Path $distBundle)) {
    $webNeedsBuild = $true
} else {
    $bundleTime = (Get-Item $distBundle).LastWriteTime
    foreach ($src in $webFiles) {
        if ($src.LastWriteTime -gt $bundleTime) {
            Write-Host "Web source $($src.Name) is newer than bundle, rebuilding..." -ForegroundColor Yellow
            $webNeedsBuild = $true
            break
        }
    }
}

if ($webNeedsBuild) {
    Write-Host "Building web UI..." -ForegroundColor Yellow
    Push-Location (Join-Path $root "web")
    npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Web build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "Web UI built successfully." -ForegroundColor Green
} else {
    Write-Host "Web UI is up to date." -ForegroundColor Gray
}

# === Stage web/dist next to the release binary ===
$releaseDir = Join-Path $root "target\release"
$stageDist = Join-Path $releaseDir "web\dist"
if (Test-Path $distDir) {
    if (Test-Path $stageDist) {
        Remove-Item $stageDist -Recurse -Force
    }
    New-Item -ItemType Directory -Path (Split-Path $stageDist -Parent) -Force | Out-Null
    Copy-Item $distDir $stageDist -Recurse -Force
    Write-Host "Staged web UI to $stageDist" -ForegroundColor Gray
}

$configSrc = Join-Path $root "config.json"
$configDst = Join-Path $releaseDir "config.json"
if ((Test-Path $configSrc) -and -not (Test-Path $configDst)) {
    Copy-Item $configSrc $configDst -Force
    Write-Host "Staged config.json next to release binary" -ForegroundColor Gray
}