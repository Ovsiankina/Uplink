# msg-test.ps1 — Run only the message-sending E2E tests
#
# What this does:
#   1. Creates 2 fresh Uplink accounts (ChatUserA and ChatUserB)
#   2. Sends friend requests between them and accepts
#   3. Runs the message input tests (send, empty, max length, emoji, paste, etc.)
#
# Accounts use separate data dirs:
#   UserA -> ~/.uplink
#   UserB -> ~/.uplinkUserB
#
# Do NOT start Uplink manually. This script manages everything.

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

function Write-Step { param([string]$Text); Write-Host ""; Write-Host "==> $Text" -ForegroundColor Cyan }
function Write-Ok   { param([string]$Text); Write-Host "  [OK] $Text" -ForegroundColor Green }

Write-Host ""
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host "         Uplink Message Test Runner" -ForegroundColor Cyan
Write-Host "=====================================================" -ForegroundColor Cyan

Write-Step "Checking prerequisites"
$nodeVersion = & node --version 2>$null
if (-not $nodeVersion) { Write-Host "  [ERROR] Node.js not found" -ForegroundColor Red; exit 1 }
Write-Ok "Node.js $nodeVersion"

$UplinkExe = "C:\Users\Ovsiankina\Documents\epsic\Uplink\target\debug\uplink.exe"
if (-not (Test-Path $UplinkExe)) {
    Write-Host "  [ERROR] uplink.exe not found at $UplinkExe" -ForegroundColor Red
    exit 1
}
Write-Ok "Uplink found"

Write-Step "Stopping any leftover processes"
foreach ($port in @(4723, 4724)) {
    $conn = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue
    if ($conn) {
        $conn | Select-Object -ExpandProperty OwningProcess | Sort-Object -Unique | ForEach-Object {
            Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
        }
        Write-Ok "Cleared port $port"
    }
}
Get-Process -Name "uplink" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Write-Step "Running message tests (2 Uplink instances will launch automatically)"
Write-Host "  Suite: Create accounts -> Add friends -> Send messages" -ForegroundColor Gray
Write-Host ""

Set-Location $ProjectRoot
$env:DRIVER = "windows"
npx cross-env DRIVER=windows npx wdio config/wdio.windows.chats.conf.ts

$result = $LASTEXITCODE

Write-Host ""
Write-Host "=====================================================" -ForegroundColor Cyan
if ($result -eq 0) {
    Write-Host "  RESULT: PASSED" -ForegroundColor Green
} else {
    Write-Host "  RESULT: FAILED (exit code $result)" -ForegroundColor Red
    Write-Host "  Check appium.log for details." -ForegroundColor Yellow
}
Write-Host "=====================================================" -ForegroundColor Cyan
Write-Host ""

exit $result
