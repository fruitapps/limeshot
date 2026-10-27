<#
.SYNOPSIS
  Build LimeShot for Windows x64: a staged folder, a portable ZIP and an Inno Setup installer (PF-006).

.DESCRIPTION
  scripts\package-windows.ps1 [-Version 0.1.0-alpha.1] [-SkipBuild] [-NoInstaller]

  Output (dist\):
    windows\LimeShot\                              staged files (limeshot.exe, limeshot-cli.exe, samples, plugins, licences)
    LimeShot-<version>-windows-x64-portable.zip   unzip and run; the Lime VST3s sit in plugins\ beside the exe
    LimeShot-<version>-windows-x64-setup.exe      installer (needs Inno Setup 6: ISCC.exe on PATH, in the default
                                                  install folder, or named by $env:ISCC)

  The version defaults to $env:LIMESHOT_VERSION, then to the workspace version in Cargo.toml. Binaries are not
  code-signed: SmartScreen warns on first start until a signing certificate is configured.
  Before packaging, the staged limeshot-cli renders the demo song through the staged Lime VST3s and fails when
  the render is silent.
#>
[CmdletBinding()]
param(
    [string]$Version = $env:LIMESHOT_VERSION,
    [switch]$SkipBuild,
    [switch]$NoInstaller
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $root

function Invoke-Checked([string]$what, [scriptblock]$block) {
    & $block
    if ($LASTEXITCODE -ne 0) { throw "$what failed with exit code $LASTEXITCODE" }
}

if (-not $Version) {
    $Version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches[0].Groups[1].Value
}
$Version = $Version.TrimStart('v')
# Windows file versions are numeric: 0.1.0-alpha.1 -> 0.1.0.
$numeric = ($Version -split '[-+]')[0]
$dist = Join-Path $root 'dist'
$stage = Join-Path $dist 'windows\LimeShot'
$cargoTarget = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }

if (-not $SkipBuild) {
    Invoke-Checked 'cargo build' { cargo build --release --locked -p limeshot-app --bins }
    Push-Location native-plugins
    try { Invoke-Checked 'native-plugins/build.py' { python build.py } } finally { Pop-Location }
}

Write-Host "Staging LimeShot $Version in $stage"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force $stage, "$stage\plugins", "$stage\licenses\fonts", "$stage\licenses\plugins" | Out-Null
Copy-Item "$cargoTarget\release\limeshot.exe", "$cargoTarget\release\limeshot-cli.exe" $stage
Copy-Item assets\samples "$stage\samples" -Recurse
# LimeShot's instruments and effects are the Lime VST3s (D39); the app finds them in plugins\ beside the exe
# or in the shared VST3 folders.
$bundles = Get-ChildItem native-plugins\dist -Directory -Filter *.vst3
if ($bundles.Count -ne 12) { throw "expected 12 Lime VST3 bundles in native-plugins\dist, found $($bundles.Count)" }
$bundles | ForEach-Object { Copy-Item $_.FullName "$stage\plugins\$($_.Name)" -Recurse }
Copy-Item LICENSE, LICENSE-MIT, LICENSE-APACHE, ATTRIBUTION.md $stage
# Fonts are embedded in the executable; their licence texts travel with the app, like the plugins' licences.
Copy-Item assets\fonts\*.txt "$stage\licenses\fonts"
Copy-Item native-plugins\dist\licenses\* "$stage\licenses\plugins" -Recurse
# Cargo's registry unpacks some crate files with the Unix epoch (1970); ZIP timestamps begin at 1980.
$now = Get-Date
Get-ChildItem $stage -Recurse -File | Where-Object { $_.LastWriteTime.Year -lt 1980 } | ForEach-Object { $_.LastWriteTime = $now }
Copy-Item packaging\windows\README.txt "$stage\README.txt"
Copy-Item packaging\windows\LimeShot.ico "$stage\LimeShot.ico"
& "$PSScriptRoot\check-windows-icons.ps1" -Executables @("$stage\limeshot.exe", "$stage\limeshot-cli.exe")

# Smoke test: the staged CLI renders the demo through the staged bundles.
$check = Join-Path $dist 'windows\check'
New-Item -ItemType Directory -Force $check | Out-Null
$env:LIMESHOT_LIME_PLUGINS = "$stage\plugins"
$render = & "$stage\limeshot-cli.exe" demo "$check\demo.lime" --render "$check\demo.wav" 2>&1 | Out-String
$code = $LASTEXITCODE
Remove-Item Env:LIMESHOT_LIME_PLUGINS
Write-Host $render.Trim()
if ($code -ne 0) { throw "staged limeshot-cli failed ($code)" }
if ($render -match 'peak -inf') { throw 'silent render: the staged Lime VST3s did not load' }
Remove-Item $check -Recurse -Force

$zip = Join-Path $dist "LimeShot-$Version-windows-x64-portable.zip"
if (Test-Path $zip) { Remove-Item $zip }
Compress-Archive -Path $stage -DestinationPath $zip -CompressionLevel Optimal
Write-Host "built $zip"

if ($NoInstaller) { return }
$iscc = $env:ISCC
if (-not $iscc) {
    $cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($cmd) { $iscc = $cmd.Source }
}
foreach ($candidate in @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe")) {
    if (-not $iscc -and (Test-Path $candidate)) { $iscc = $candidate }
}
if (-not $iscc) { throw 'Inno Setup 6 (ISCC.exe) not found: install it (winget install JRSoftware.InnoSetup) or pass -NoInstaller' }
$base = "LimeShot-$Version-windows-x64-setup"
Invoke-Checked 'ISCC' {
    & $iscc /Q "/DAppVersion=$Version" "/DNumericVersion=$numeric" "/DStage=$stage" "/DOutputDir=$dist" "/DOutputBase=$base" packaging\windows\limeshot.iss
}
Write-Host "built $(Join-Path $dist "$base.exe")"
