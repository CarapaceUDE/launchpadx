# Test script to diagnose health check and model loading issues
$root = 'C:\app\codex-local-launcher'

Write-Host "=== Codex Local Launcher Diagnostic ==="
Write-Host ""

# Get config
$configPath = Join-Path $root "config.json"
if (Test-Path $configPath) {
    $config = Get-Content $configPath -Raw | ConvertFrom-Json
    Write-Host "Config loaded from: $configPath"
    Write-Host "  openaiBaseUrl: $($config.openaiBaseUrl)"
    Write-Host "  ollamaIp: $($config.ollamaIp)"
    Write-Host "  ollamaPort: $($config.ollamaPort)"
    Write-Host "  ollamaScheme: $($config.ollamaScheme)"
    Write-Host ""
} else {
    Write-Host "ERROR: config.json not found at $configPath"
    exit 1
}

# Determine the endpoint
$baseUrl = $config.openaiBaseUrl
if (-not $baseUrl -and $config.ollamaIp) {
    $scheme = if ($config.ollamaScheme) { $config.ollamaScheme } else { "http" }
     $port = if ($config.ollamaPort) { $config.ollamaPort } else { 11434 }
     $baseUrl = [string]::Format("{0}://{1}:{2}/v1", $scheme, $config.ollamaIp, $port)
}

if (-not $baseUrl) {
    Write-Host "ERROR: No endpoint configured (set ollamaIp or openaiBaseUrl)"
    exit 1
}

Write-Host "Endpoint: $baseUrl"
Write-Host ""

# Test 1: Check if the RPC endpoint is reachable
$rpcUrl = "http://127.0.0.1:5180/rpc"
Write-Host "Test 1: RPC endpoint ($rpcUrl)"
try {
     $resp = Invoke-WebRequest -Uri $rpcUrl -Method POST -ContentType "application/json" -Body '{"method":"healthCheck","params":{}}' -TimeoutSec 5 -UseBasicParsing
     $json = $resp.Content | ConvertFrom-Json
     Write-Host "  Status: $($resp.StatusCode)"
     Write-Host "  Response: $($resp.Content)"
     Write-Host ""
} catch {
    Write-Host "  FAILED: $($_.Exception.Message)"
    Write-Host "  The backend server may not be running, or is on a different port."
    Write-Host ""
}

# Test 2: Direct Ollama health check
$healthUrl = "$baseUrl/health"
Write-Host "Test 2: Direct Ollama health ($healthUrl)"
try {
     $resp = Invoke-WebRequest -Uri $healthUrl -TimeoutSec 5 -UseBasicParsing
     Write-Host "  Status: $($resp.StatusCode)"
     Write-Host "  Response: $($resp.Content)"
} catch {
    Write-Host "  FAILED: $($_.Exception.Message)"
    if ($_.Exception.InnerException -and $_.Exception.InnerException.Message) {
        Write-Host "  Inner: $($_.Exception.InnerException.Message)"
    }
}

# Test 3: Ollama tags
$tagsUrl = "$baseUrl/tags"
Write-Host "Test 3: Ollama tags ($tagsUrl)"
try {
     $resp = Invoke-WebRequest -Uri $tagsUrl -TimeoutSec 5 -UseBasicParsing
     $json = $resp.Content | ConvertFrom-Json
     Write-Host "  Status: $($resp.StatusCode)"
     Write-Host "  Models: $($json.models.Count)"
     $json.models | ForEach-Object { Write-Host "   - $($_.name) ($([math]::Round($_.size/1GB,2)) GB)" }
} catch {
    Write-Host "  FAILED: $($_.Exception.Message)"
}

Write-Host ""
Write-Host "=== Diagnostic complete ==="
