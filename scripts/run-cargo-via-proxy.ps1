param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

if (-not $CargoArgs -or $CargoArgs.Count -eq 0) {
    Write-Host "Usage: pwsh ./scripts/run-cargo-via-proxy.ps1 <cargo-args>"
    Write-Host "Example: pwsh ./scripts/run-cargo-via-proxy.ps1 check"
    exit 1
}

$listening = Get-NetTCPConnection -LocalPort 7879 -State Listen -ErrorAction SilentlyContinue
if (-not $listening) {
    Write-Error "Cargo proxy is not running on 127.0.0.1:7879. Start it with: pwsh ./scripts/cargo-sparse-proxy.ps1"
}

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = "cargo"
$startInfo.UseShellExecute = $false

foreach ($arg in $CargoArgs) {
    [void]$startInfo.ArgumentList.Add($arg)
}

$startInfo.Environment["CARGO_REGISTRIES_CRATES_IO_PROTOCOL"] = "sparse"
$startInfo.Environment["CARGO_REGISTRIES_CRATES_IO_INDEX"] = "sparse+http://127.0.0.1:7879/"
$startInfo.Environment["CARGO_NET_GIT_FETCH_WITH_CLI"] = "true"
$startInfo.Environment["CARGO_HTTP_MULTIPLEXING"] = "false"
$startInfo.Environment["CARGO_HTTP_CHECK_REVOKE"] = "false"

$process = [System.Diagnostics.Process]::Start($startInfo)
$process.WaitForExit()
exit $process.ExitCode