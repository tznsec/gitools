# Buduje instalator gitools-setup.exe.
#
# Kolejnosc jest istotna: instalator osadza gotowy gitools.exe, wiec najpierw
# budujemy aplikacje, potem kopiujemy ja do `build/`, a dopiero na koncu
# kompilujemy instalator.
#
#   powershell -ExecutionPolicy Bypass -File installer\build.ps1

$ErrorActionPreference = 'Stop'

$root     = Split-Path -Parent $PSScriptRoot
$instDir  = $PSScriptRoot
$buildDir = Join-Path $instDir 'build'
$distDir  = Join-Path $root 'dist'

Write-Host '==> 1/3  cargo build --release (aplikacja)'
Push-Location $root
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'build aplikacji zakonczony bledem' }
} finally { Pop-Location }

Write-Host '==> 2/3  kopiowanie plikow do instalatora'
New-Item -ItemType Directory -Force -Path $buildDir | Out-Null
New-Item -ItemType Directory -Force -Path $distDir  | Out-Null

Copy-Item (Join-Path $root 'target\release\gitools.exe') (Join-Path $buildDir 'gitools.exe') -Force

Write-Host '==> 3/3  cargo build --release (instalator)'
Push-Location $instDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw 'build instalatora zakonczony bledem' }
} finally { Pop-Location }

# --- wynik -------------------------------------------------------------------
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
$out = Join-Path $distDir "gitools-setup-$version.exe"
Copy-Item (Join-Path $instDir 'target\release\gitools-setup.exe') $out -Force

# przenośna wersja bez instalowania — dla tych, którzy nie chcą instalować
$zip = Join-Path $distDir "gitools-$version-portable.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path (Join-Path $root 'target\release\gitools.exe'), (Join-Path $root 'LICENSE'), (Join-Path $root 'README.md') -DestinationPath $zip

# katalog z instalatorem dla GitHub Pages
$dl = Join-Path $root 'docs\downloads'
New-Item -ItemType Directory -Force -Path $dl | Out-Null
Copy-Item $out $dl -Force
Copy-Item $zip $dl -Force

Write-Host ''
Write-Host 'Gotowe:'
Get-ChildItem $distDir | Select-Object Name, @{n='MB';e={[math]::Round($_.Length/1MB,2)}} | Format-Table -AutoSize