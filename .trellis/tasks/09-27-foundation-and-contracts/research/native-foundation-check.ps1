param([Parameter(Mandatory = $true)][string]$Workspace, [string]$SourceName = 'foundation')
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Set-StrictMode -Version Latest

function Invoke-Cargo([string[]] $Arguments) {
    & $script:cargo @Arguments
    if ($LASTEXITCODE -ne 0) { throw ('Native Cargo check failed: ' + ($Arguments -join ' ')) }
    Write-Output ('PASS: cargo ' + ($Arguments -join ' '))
}

if ($SourceName -notin 'foundation', 'foundation-v2') { throw 'Unknown owned source directory.' }
$source = Join-Path $Workspace $SourceName
$tools = Join-Path $Workspace 'rust\bin'
$script:cargo = Join-Path $tools 'cargo.exe'
$env:PATH = $tools + ';' + $env:PATH
$env:RUSTC = Join-Path $tools 'rustc.exe'
$env:CARGO_HOME = Join-Path $Workspace 'cargo-cache'
$env:CARGO_NET_OFFLINE = 'true'
$env:CARGO_TARGET_DIR = Join-Path $Workspace 'foundation-target'
Set-Location -LiteralPath $source

Write-Output ('Cargo.lock SHA256: ' + (Get-FileHash -LiteralPath (Join-Path $source 'Cargo.lock') -Algorithm SHA256).Hash)
Invoke-Cargo -Arguments @('fmt', '--all', '--', '--check')
Invoke-Cargo -Arguments @('check', '--workspace', '--all-targets', '--locked', '--offline')
Invoke-Cargo -Arguments @('clippy', '--workspace', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings')
Invoke-Cargo -Arguments @('test', '--workspace', '--locked', '--offline')
Invoke-Cargo -Arguments @('build', '--workspace', '--locked', '--offline')

$config_path = Join-Path $Workspace 'foundation-full-check.json'
$config = @{
    schema_version = 1; role = 'full'; source = 'en'; target = 'zh-Hans'
    capture = @{ mode = 'follow_default' }
    inference = @{ asr_model = 'models/asr.gguf'; translation_model = 'models/translation.gguf'; device = @{ backend = 'cpu' } }
}
[System.IO.File]::WriteAllText($config_path, ($config | ConvertTo-Json -Depth 5))
Set-Location -LiteralPath $Workspace
foreach ($role in @('full', 'server', 'client')) {
    $executable = Join-Path $env:CARGO_TARGET_DIR ('debug\synesthesia-' + $role + '.exe')
    & $executable --help
    if ($LASTEXITCODE -ne 0) { throw 'Native executable help failed.' }
    & $executable --version
    if ($LASTEXITCODE -ne 0) { throw 'Native executable version failed.' }
}
$full = Join-Path $env:CARGO_TARGET_DIR 'debug\synesthesia-full.exe'
& $full --check-config $config_path --runtime-root $source
if ($LASTEXITCODE -ne 0) { throw 'Native CWD-independent config validation failed.' }
Write-Output 'PASS: native entries and CWD-independent structural config validation; models/runtime remain unverified.'
