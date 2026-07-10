[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"

function Resolve-LauncherPath {
    if ($PSScriptRoot) {
        return $PSScriptRoot
     }

    return (Get-Location).Path
}

function Read-LauncherConfig {
    param(
         [Parameter(Mandatory = $true)]
         [string]$ConfigPath
     )

    if (-not (Test-Path -LiteralPath $ConfigPath)) {
        throw "Missing config file: $ConfigPath. Copy config.example.json to config.json and fill in your Ollama IP and API key."
     }

    try {
        return Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
     }
    catch {
        throw "Could not parse $ConfigPath as JSON. $($_.Exception.Message)"
     }
}

function Join-BaseUrl {
    param(
         [Parameter(Mandatory = $true)]
         [object]$Config
     )

    if ($Config.openaiBaseUrl) {
        return [string]$Config.openaiBaseUrl
     }

    if (-not $Config.ollamaIp) {
        throw "config.json must set either openaiBaseUrl or ollamaIp."
     }

    $scheme = if ($Config.ollamaScheme) { [string]$Config.ollamaScheme } else { "http" }
    $port = if ($Config.ollamaPort) { [int]$Config.ollamaPort } else { 11434 }
    $hostValue = ([string]$Config.ollamaIp).Trim()

    if ($hostValue -eq "100.64.0.10") {
        throw "config.json still has the example ollamaIp. Set it to your actual Tailscale IP or set openaiBaseUrl."
     }

    if ($hostValue -match "^https?://") {
         $base = $hostValue.TrimEnd("/")
        if ($base -notmatch "/v1$") {
             $base = "$base/v1"
         }
        return $base
     }

    if ($hostValue -match ":") {
         $hostValue = "[$hostValue]"
     }

    return "$scheme`://$hostValue`:$port/v1"
}

function Resolve-CodexCommand {
    param(
         [Parameter(Mandatory = $true)]
         [object]$Config
     )

    function Test-IsPackagedCodexResource {
        param([string]$Path)

        return $Path -match "\\WindowsApps\\OpenAI\.Codex_[^\\]+\\app\\resources\\codex(\.exe)?$"
     }

    function New-CodexPathTarget {
        param([string]$Path)

        return [pscustomobject]@{
            Type   = "Path"
            Value = $Path
         }
     }

    function New-CodexStartAppTarget {
        param([string]$AppID)

        return [pscustomobject]@{
            Type   = "StartAppID"
            Value = $AppID
         }
     }

    function Get-CodexStartAppID {
         $startApp = Get-StartApps Codex -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -eq "Codex" -or $_.Name -like "Codex*" } |
            Select-Object -First 1

        if ($startApp) {
            return [string]$startApp.AppID
         }

        return ""
     }

    if ($Config.codexCommand -and $Config.codexCommand.Trim().Length -gt 0) {
         $configuredCommand = [Environment]::ExpandEnvironmentVariables([string]$Config.codexCommand)

        if (Test-Path -LiteralPath $configuredCommand) {
            if (Test-IsPackagedCodexResource -Path $configuredCommand) {
                 $startAppID = Get-CodexStartAppID
                if ($startAppID) {
                    return New-CodexStartAppTarget -AppID $startAppID
                 }

                throw "Configured codexCommand points to Codex packaged resource alias, which Windows blocks from direct launch: $configuredCommand. Could not find a Codex Start menu AppID."
             }

            return New-CodexPathTarget -Path (Resolve-Path -LiteralPath $configuredCommand).Path
         }

         $configuredPathCommand = Get-Command $configuredCommand -ErrorAction SilentlyContinue
        if ($configuredPathCommand) {
            if (Test-IsPackagedCodexResource -Path $configuredPathCommand.Source) {
                 $startAppID = Get-CodexStartAppID
                if ($startAppID) {
                    return New-CodexStartAppTarget -AppID $startAppID
                 }

                throw "Configured codexCommand resolves to Codex packaged resource alias, which Windows blocks from direct launch: $($configuredPathCommand.Source). Could not find a Codex Start menu AppID."
             }

            return New-CodexPathTarget -Path $configuredPathCommand.Source
         }

        throw "Configured codexCommand was not found: $configuredCommand"
     }

     $pathCommandNames = @(
         "codex-app.exe",
         "codex-app.cmd",
         "codex-app",
         "codex.exe",
         "codex.cmd",
         "codex"
     )

    foreach ($name in $pathCommandNames) {
         $command = Get-Command $name -ErrorAction SilentlyContinue
        if ($command) {
            if (Test-IsPackagedCodexResource -Path $command.Source) {
                continue
             }

            return New-CodexPathTarget -Path $command.Source
         }
     }

     $localAppData = [Environment]::GetFolderPath("LocalApplicationData")
     $programFiles = [Environment]::GetFolderPath("ProgramFiles")
     $programFilesX86 = [Environment]::GetFolderPath("ProgramFilesX86")

     $candidatePaths = @(
         "$localAppData\Programs\Codex\Codex.exe",
         "$localAppData\Programs\OpenAI Codex\Codex.exe",
         "$localAppData\Programs\codex-app\Codex.exe",
         "$localAppData\Programs\codex-app\codex-app.exe",
         "$localAppData\Codex\Codex.exe",
         "$localAppData\OpenAI Codex\Codex.exe",
         "$localAppData\OpenAI\Codex\Codex.exe",
         "$localAppData\openai-codex-electron\Codex.exe",
         "$programFiles\Codex\Codex.exe",
         "$programFiles\codex-app\Codex.exe",
         "$programFiles\codex-app\codex-app.exe",
         "$programFilesX86\Codex\Codex.exe",
         "$programFilesX86\codex-app\Codex.exe",
         "$programFilesX86\codex-app\codex-app.exe"
     )

    foreach ($pattern in @(
         "$localAppData\Programs\Codex\app-*\Codex.exe",
         "$localAppData\Programs\OpenAI Codex\app-*\Codex.exe",
         "$localAppData\Codex\app-*\Codex.exe",
         "$localAppData\OpenAI Codex\app-*\Codex.exe",
         "$localAppData\OpenAI\Codex\app-*\Codex.exe",
         "$localAppData\openai-codex-electron\app-*\Codex.exe"
     )) {
         $candidatePaths += @(Get-ChildItem -Path $pattern -ErrorAction SilentlyContinue | ForEach-Object { $_.FullName })
     }

    foreach ($path in $candidatePaths) {
        if ($path -and (Test-Path -LiteralPath $path)) {
            return New-CodexPathTarget -Path $path
         }
     }

     $startAppID = Get-CodexStartAppID
    if ($startAppID) {
        return New-CodexStartAppTarget -AppID $startAppID
     }

    throw "Could not find Codex. Set codexCommand in config.json to the full path of Codex.exe or a command on PATH."
}

function Convert-Args {
    param([object]$ArgsValue)

    if (-not $ArgsValue) {
        return @()
     }

    if ($ArgsValue -is [array]) {
        return @($ArgsValue | ForEach-Object { [string]$_ })
     }

    return @([string]$ArgsValue)
}

$launcherRoot = Resolve-LauncherPath
$configPath = Join-Path $launcherRoot "config.json"
$config = Read-LauncherConfig -ConfigPath $configPath

$baseUrl = Join-BaseUrl -Config $config
if (-not $config.apiKey -or [string]::IsNullOrWhiteSpace($config.apiKey)) {
    throw "config.json must set apiKey."
}
if ([string]$config.apiKey -eq "replace-with-your-api-key") {
    throw "config.json still has the example apiKey. Set it to the key your Ollama endpoint expects."
}

$codexTarget = Resolve-CodexCommand -Config $config
$codexArgs = Convert-Args -ArgsValue $config.codexArgs
$workingDirectory = if ($config.workingDirectory -and $config.workingDirectory.Trim().Length -gt 0) {
     [Environment]::ExpandEnvironmentVariables([string]$config.workingDirectory)
}
else {
     $launcherRoot
}

if (-not (Test-Path -LiteralPath $workingDirectory)) {
    throw "workingDirectory does not exist: $workingDirectory"
}

$env:OPENAI_BASE_URL = $baseUrl
$env:OPENAI_API_KEY = [string]$config.apiKey

Write-Host "Launching Codex"
Write-Host "OPENAI_BASE_URL=$baseUrl"
Write-Host "LaunchTarget=$($codexTarget.Type):$($codexTarget.Value)"

if ($codexTarget.Type -eq "StartAppID") {
    if ($codexArgs.Count -gt 0) {
        throw "codexArgs cannot be passed when launching the Microsoft Store packaged Codex app through its Start menu AppID."
     }

    Write-Warning "Launching the Microsoft Store packaged Codex app through shell:AppsFolder. Windows may not pass process-local environment variables to packaged app activation; use Codex config file for persistent local model settings."
    Start-Process "shell:AppsFolder\$($codexTarget.Value)"
    return
}

$startInfo = @{
    FilePath          = $codexTarget.Value
    WorkingDirectory = $workingDirectory
}

if ($codexArgs.Count -gt 0) {
     $startInfo.ArgumentList = $codexArgs
}

Start-Process @startInfo
