function Get-CargoCommand {
    $pathCargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($pathCargo) {
        return $pathCargo.Source
    }

    $userCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path -LiteralPath $userCargo) {
        return $userCargo
    }

    throw "Could not find cargo. Add %USERPROFILE%\.cargo\bin to PATH or install Rust."
}
