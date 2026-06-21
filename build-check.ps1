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
$webFiles = Get-ChildItem -Recurse -File $srcDir -ErrorAction SilentlyContinue
$distFiles = Get-ChildItem -Recurse -File $distDir -ErrorAction SilentlyContinue
$webNeedsBuild = $false

if ($distFiles.Count -eq 0) {
       $webNeedsBuild = $true
} else {
    foreach ($src in $webFiles) {
          $relPath = $src.FullName.Substring($srcDir.Length)
          $distFile = $distFiles | Where-Object { $_.FullName.EndsWith($relPath) } | Select-Object -First 1
        if ($src.LastWriteTime -gt $distFile.LastWriteTime) {
               $webNeedsBuild = $true
            break
           }
       }
}

if ($webNeedsBuild) {
    Write-Host "Building web UI..." -ForegroundColor Yellow
    Push-Location (Join-Path $root "web")
      npx vite build
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