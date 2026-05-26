# Build TrayLink for Windows (.exe + NSIS installer)
param(
    [ValidateSet("native", "x64")]
    [string]$Target = "native"
)

$ErrorActionPreference = "Stop"
$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RootDir

Write-Host "==> TrayLink build (Windows)" -ForegroundColor Cyan
Write-Host "Root: $RootDir"

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "Thiếu npm. Cài Node.js trước."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Thiếu Rust/cargo. Cài rustup trước."
}

Write-Host "==> Cài dependencies"
npm ci

if (Test-Path "public/icon.png") {
    Write-Host "==> Generate icons từ public/icon.png"
    npm run tauri icon public/icon.png
}

$BuildArgs = @()
if ($Target -eq "x64") {
    Write-Host "==> Build x86_64-pc-windows-msvc"
    rustup target add x86_64-pc-windows-msvc 2>$null
    $BuildArgs += "--target", "x86_64-pc-windows-msvc"
}

Write-Host "==> Tauri build (release)"
npm run tauri build -- @BuildArgs

$ReleaseDir = Join-Path $RootDir "release/windows"
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

$BundleCandidates = @(
    (Join-Path $RootDir "src-tauri/target/release/bundle"),
    (Join-Path $RootDir "src-tauri/target/x86_64-pc-windows-msvc/release/bundle")
)

$BundleDir = $null
foreach ($candidate in $BundleCandidates) {
    if (Test-Path $candidate) {
        $BundleDir = $candidate
        break
    }
}

if (-not $BundleDir) {
    throw "Không tìm thấy bundle Windows."
}

Write-Host "==> Copy artifacts -> $ReleaseDir"
Copy-Item -Path (Join-Path $BundleDir "*") -Destination $ReleaseDir -Recurse -Force

Write-Host ""
Write-Host "✅ Build Windows xong!" -ForegroundColor Green
Write-Host "Output:"
Get-ChildItem $ReleaseDir -Recurse | Select-Object FullName

Write-Host ""
Write-Host "Autostart: bật trong Dashboard -> Settings -> Autostart khi boot"
Write-Host "  Windows: ghi Registry Run key (tauri-plugin-autostart)"
Write-Host "Cài đặt: chạy file .exe trong thư mục nsis/"
