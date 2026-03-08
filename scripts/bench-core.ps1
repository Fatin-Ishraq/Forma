param(
    [string]$OutDir = "benchmark"
)

$ErrorActionPreference = "Stop"

New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$outFile = Join-Path $OutDir "core-$stamp.txt"

"CellForge core benchmark run: $(Get-Date -Format o)" | Set-Content $outFile
"command: cargo run --release --example perf_baseline" | Add-Content $outFile
"" | Add-Content $outFile

$benchOutput = cargo run --release --example perf_baseline
$benchOutput | Add-Content $outFile

Write-Host ""
Write-Host "Saved benchmark: $outFile"
Write-Host ""
$benchOutput
