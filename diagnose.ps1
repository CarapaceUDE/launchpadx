[CmdletBinding()]
param(
    [string]$ConfigPath = $(Join-Path $PSScriptRoot "config.json")
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Write-Section {
    param([Parameter(Mandatory = $true)][string]$Title)

    Write-Host ""
    Write-Host "== $Title =="
}

function Format-ConfiguredSecret {
    param([AllowNull()][string]$Value)

    if ([string]::IsNullOrWhiteSpace($Value) -or $Value -eq "replace-with-your-api-key") {
        return "(not set)"
    }

    return "Configured (redacted)"
}

function Read-LauncherConfig {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Missing config file: $Path. Copy config.example.json to config.json and fill in your endpoint settings."
    }

    try {
        return Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    } catch {
        throw "Could not parse $Path as JSON. $($_.Exception.Message)"
    }
}

function Get-BaseUrl {
    param([Parameter(Mandatory = $true)][object]$Config)

    if ($Config.openaiBaseUrl) {
        $base = ([string]$Config.openaiBaseUrl).Trim().TrimEnd("/")
        if ($base -notmatch "/v1$") {
            $base = "$base/v1"
        }
        return $base
    }

    if (-not $Config.ollamaIp) {
        throw "config.json must set either openaiBaseUrl or ollamaIp."
    }

    $hostValue = ([string]$Config.ollamaIp).Trim()
    if ($hostValue -eq "100.64.0.10") {
        throw "config.json still has the example ollamaIp. Set it to your real endpoint or set openaiBaseUrl."
    }

    $scheme = if ($Config.ollamaScheme) { [string]$Config.ollamaScheme } else { "http" }
    $port = if ($Config.ollamaPort) { [int]$Config.ollamaPort } else { 11434 }

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

function Get-OllamaTagsUrl {
    param([Parameter(Mandatory = $true)][string]$BaseUrl)

    $trimmed = $BaseUrl.TrimEnd("/")
    if ($trimmed -match "/api$") {
        return "$trimmed/tags"
    }
    if ($trimmed -match "/v1$") {
        return (($trimmed -replace "/v1$", "/api") + "/tags")
    }
    return "$trimmed/api/tags"
}

function Get-CodexApiBaseUrl {
    param([Parameter(Mandatory = $true)][object]$Config)

    $scheme = if ($Config.codexApiScheme) { [string]$Config.codexApiScheme } else { "http" }
    $port = if ($Config.codexApiPort) { [int]$Config.codexApiPort } else { 4000 }
    return "$scheme`://127.0.0.1`:$port"
}

function Get-WorkingDirectory {
    param(
        [Parameter(Mandatory = $true)][object]$Config,
        [Parameter(Mandatory = $true)][string]$DefaultPath
    )

    if ($Config.workingDirectory -and [string]$Config.workingDirectory -ne "") {
        return [Environment]::ExpandEnvironmentVariables([string]$Config.workingDirectory)
    }

    return $DefaultPath
}

function Invoke-DiagnosticRequest {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Uri,
        [hashtable]$Headers = @{}
    )

    try {
        $response = Invoke-WebRequest -Uri $Uri -Headers $Headers -TimeoutSec 5 -UseBasicParsing
        return [pscustomobject]@{
            Name = $Name
            Success = $true
            Status = [int]$response.StatusCode
            Detail = "OK"
        }
    } catch {
        $statusCode = $null
        if ($_.Exception.Response -and $_.Exception.Response.StatusCode) {
            $statusCode = [int]$_.Exception.Response.StatusCode
        }
        $detail = $_.Exception.Message
        return [pscustomobject]@{
            Name = $Name
            Success = $false
            Status = $statusCode
            Detail = $detail
        }
    }
}

function Write-CheckResult {
    param([Parameter(Mandatory = $true)][object]$Result)

    $statusText = if ($Result.Success) { "PASS" } else { "FAIL" }
    $statusCode = if ($null -ne $Result.Status) { " [$($Result.Status)]" } else { "" }
    Write-Host ("{0,-5} {1}{2}" -f $statusText, $Result.Name, $statusCode)
    if (-not $Result.Success -and $Result.Detail) {
        Write-Host "      $($Result.Detail)"
    }
}

try {
    $root = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
    $config = Read-LauncherConfig -Path $ConfigPath
    $baseUrl = Get-BaseUrl -Config $config
    $codexApiBaseUrl = Get-CodexApiBaseUrl -Config $config
    $workingDirectory = Get-WorkingDirectory -Config $config -DefaultPath $root
    $tagsUrl = Get-OllamaTagsUrl -BaseUrl $baseUrl

    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace([string]$config.apiKey) -and [string]$config.apiKey -ne "replace-with-your-api-key") {
        $headers["Authorization"] = "Bearer $([string]$config.apiKey)"
    }

    Write-Host "=== Codex Launchpad Diagnostic ==="

    Write-Section "Configuration"
    Write-Host "Config path      : $ConfigPath"
    Write-Host "Endpoint         : $baseUrl"
    Write-Host "Tags endpoint    : $tagsUrl"
    Write-Host "Codex API        : $codexApiBaseUrl"
    Write-Host "API key          : $(Format-ConfiguredSecret -Value ([string]$config.apiKey))"
    Write-Host "Working directory: $workingDirectory"
    Write-Host "Codex command    : $(if ($config.codexCommand) { [string]$config.codexCommand } else { '(auto-detect)' })"

    Write-Section "Local Checks"
    Write-CheckResult ([pscustomobject]@{
        Name = "Config file parses"
        Success = $true
        Status = $null
        Detail = ""
    })
    Write-CheckResult ([pscustomobject]@{
        Name = "Working directory exists"
        Success = (Test-Path -LiteralPath $workingDirectory)
        Status = $null
        Detail = $(if (Test-Path -LiteralPath $workingDirectory) { "" } else { "Missing path: $workingDirectory" })
    })

    $releaseBinary = Join-Path $root "target\release\codex-launchpad.exe"
    $debugBinary = Join-Path $root "target\debug\codex-launchpad.exe"
    $binaryPath = if (Test-Path -LiteralPath $releaseBinary) { $releaseBinary } elseif (Test-Path -LiteralPath $debugBinary) { $debugBinary } else { $null }
    Write-CheckResult ([pscustomobject]@{
        Name = "Launcher binary present"
        Success = ($null -ne $binaryPath)
        Status = $null
        Detail = $(if ($binaryPath) { $binaryPath } else { "Build the project first with .\\build.cmd or cargo build --release" })
    })

    Write-Section "Network Checks"
    Write-CheckResult (Invoke-DiagnosticRequest -Name "Ollama-compatible tags endpoint" -Uri $tagsUrl -Headers $headers)
    Write-CheckResult (Invoke-DiagnosticRequest -Name "Codex API health" -Uri "$codexApiBaseUrl/health")

    Write-Host ""
    Write-Host "Diagnostic complete."
} catch {
    Write-Host ""
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
