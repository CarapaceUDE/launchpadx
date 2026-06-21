$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Split-Path -Parent $scriptDir
$srcDir = Join-Path $root "web\src"
$distDir = Join-Path $root "web\dist"

$srcFiles = Get-ChildItem -Recurse -File $srcDir -ErrorAction SilentlyContinue
$distFiles = Get-ChildItem -Recurse -File $distDir -ErrorAction SilentlyContinue

$needsRebuild = $false

if ($distFiles.Count -eq 0) {
     $needsRebuild = $true
} else {
    foreach ($src in $srcFiles) {
         $relPath = $src.FullName.Substring($srcDir.Length)
         $distFile = $distFiles | Where-Object { $_.FullName.EndsWith($relPath) } | Select-Object -First 1
        if ($src.LastWriteTime -gt $distFile.LastWriteTime) { $needsRebuild = $true; break }
     }
}

if ($needsRebuild) { Write-Output "" } else { Write-Output "UP_TO_DATE" }