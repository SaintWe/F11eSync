Param(
  [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

$AppDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$RepoRoot = (Resolve-Path (Join-Path $AppDir "..")).Path

$DistDir = Join-Path $RepoRoot "dist"
$TargetRoot = Join-Path $DistDir "rust-target"
$LicOut = Join-Path $DistDir "THIRD_PARTY_LICENSES.txt"

New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

$Py = Get-Command python -ErrorAction SilentlyContinue
if (-not $Py) { $Py = Get-Command python3 -ErrorAction SilentlyContinue }
if (-not $Py) { throw "python not found (required to generate THIRD_PARTY_LICENSES.txt)" }
& $Py.Source (Join-Path $RepoRoot "scripts\\generate-third-party-licenses.py") | Out-Null
if (!(Test-Path $LicOut)) {
  throw "THIRD_PARTY_LICENSES.txt not found: $LicOut"
}

function Build-And-Zip {
  Param(
    [string]$Name,
    [string[]]$CargoArgs,
    [string]$BuiltExeName,
    [string]$ExeOutName,
    [string]$ZipOutName
  )

  Push-Location $RepoRoot
  try {
    if ($Profile -eq "release") {
      cargo build --manifest-path app\Cargo.toml --release @CargoArgs --target-dir $TargetRoot
    } else {
      cargo build --manifest-path app\Cargo.toml @CargoArgs --target-dir $TargetRoot
    }
  } finally {
    Pop-Location
  }

  $BuiltExe = Join-Path (Join-Path $TargetRoot $Profile) $BuiltExeName
  if (!(Test-Path $BuiltExe)) {
    throw "build output not found: $BuiltExe"
  }

  $ExeOut = Join-Path $DistDir $ExeOutName
  Copy-Item $BuiltExe $ExeOut

  $ZipOut = Join-Path $DistDir $ZipOutName
  if (Test-Path $ZipOut) { Remove-Item -Force $ZipOut }
  Compress-Archive -Path @($ExeOut, $LicOut) -DestinationPath $ZipOut -CompressionLevel Optimal
  Remove-Item -Force $ExeOut

  Write-Host "OK:" $ZipOut
}

Build-And-Zip -Name "Rust CLI" -CargoArgs @("--no-default-features", "--bin", "f11esync") -BuiltExeName "f11esync.exe" -ExeOutName "f11esync-rust-windows-x64.exe" -ZipOutName "f11esync-rust-windows-x64.zip"
Build-And-Zip -Name "Rust GUI" -CargoArgs @("--features", "gui", "--bin", "f11esync-gui") -BuiltExeName "f11esync-gui.exe" -ExeOutName "f11esync-gui-windows-x64.exe" -ZipOutName "f11esync-gui-windows-x64.zip"
