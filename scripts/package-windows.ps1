param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not $Version) {
    $cargoToml = Get-Content "Cargo.toml" -Raw
    if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = "v$($Matches[1])"
    } else {
        throw "Unable to determine version from Cargo.toml"
    }
}

$packageRoot = Join-Path "dist" "AI-Movie-Player-$Version-windows-x64"
$zipPath = Join-Path "dist" "AI-Movie-Player-$Version-windows-x64.zip"

Remove-Item $packageRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $packageRoot | Out-Null

cargo build --release

Copy-Item "target/release/ai-movie-player.exe" "$packageRoot/AI-Movie-Player.exe"
Copy-Item "README.md", "readme-cn.md", "LICENSE" $packageRoot

Compress-Archive -Path "$packageRoot/*" -DestinationPath $zipPath -Force

Write-Host "Windows package created: $zipPath"