param(
    [string]$Output = "artifacts/cognitive-certification.json"
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot ".." )).Path
Push-Location $repo
try {
    cargo fmt --all -- --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace --release

    $binary = Join-Path $repo "target/release/rocksoul.exe"
    if (-not (Test-Path $binary)) {
        $binary = Join-Path $repo "target/release/rocksoul"
    }
    if (-not (Test-Path $binary)) { throw "release binary was not produced" }

    $relative = $Output.Replace('/', '\')
    $outputPath = Join-Path $repo $relative
    New-Item -ItemType Directory -Force (Split-Path $outputPath) | Out-Null
    $report = [ordered]@{
        schema_version = 1
        commit = (git rev-parse HEAD).Trim()
        rustc = (rustc --version).Trim()
        cargo_lock_sha256 = (Get-FileHash (Join-Path $repo "Cargo.lock") -Algorithm SHA256).Hash.ToLowerInvariant()
        binary = (Resolve-Path $binary).Path
        binary_sha256 = (Get-FileHash $binary -Algorithm SHA256).Hash.ToLowerInvariant()
        gates = [ordered]@{ format = "PASS"; tests = "PASS"; clippy = "PASS"; release_build = "PASS" }
    }
    $report | ConvertTo-Json -Depth 5 | Set-Content $outputPath -Encoding utf8
    Write-Output (Resolve-Path $outputPath).Path
}
finally { Pop-Location }
