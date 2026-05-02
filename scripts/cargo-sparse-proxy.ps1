param(
    [int]$Port = 7879
)

$ErrorActionPreference = "Stop"

& py -3 "$PSScriptRoot/cargo-sparse-proxy.py" $Port