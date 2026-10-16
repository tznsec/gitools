# Wypycha wiki na https://github.com/tznsec/gitools.wiki.git
#
# GitHub NIE MA API do tworzenia wiki — repo `.wiki.git` powstaje dopiero po
# kliknięciu „Create the first page” w przeglądarce. Potem zwykły git wystarczy.
#
#   1. Wejdź na https://github.com/tznsec/gitools/wiki
#   2. Kliknij „Create the first page”, tytuł `Home`, treść `start`
#   3. „Save Page”
#   4. Uruchom: powershell -ExecutionPolicy Bypass -File tools\push_wiki.ps1
#
# Token nigdy nie trafia na dysk — idzie w nagłówku HTTP procesu git.

param(
    [string]$Token = $env:GITHUB_TOKEN
)

$ErrorActionPreference = 'Stop'

$root    = Resolve-Path (Join-Path $PSScriptRoot '..')
$wiki    = Join-Path $root 'wiki'
$remote  = 'https://github.com/tznsec/gitools.wiki.git'

if (-not $Token) {
    Write-Error 'Brak tokena. Ustaw GITHUB_TOKEN albo podaj -Token.'
    exit 1
}

$basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("x-access-token:$Token"))
$auth  = "http.https://github.com/.extraheader=AUTHORIZATION: basic $basic"

# --- 1. strony z programu -----------------------------------------------------
Write-Host '==> generowanie stron wiki'
node (Join-Path $PSScriptRoot 'gen_wiki.mjs') $wiki | Out-Host

# --- 2. repo ------------------------------------------------------------------
Push-Location $wiki

if (-not (Test-Path '.git')) {
    Write-Host '==> inicjalizacja repozytorium wiki'
    git init -q -b master
    git remote add origin $remote
}

Write-Host '==> pobieranie tego, co juz jest na GitHubie'
git -c $auth fetch origin master | Out-Null

# `reset --hard` (a nie `checkout`) radzi sobie z plikami, które mamy
# lokalnie, a GitHub już ma — inaczej checkout odmawia nadpisania.
git reset --hard origin/master | Out-Null

# --- 3. nasze strony na wierzch ------------------------------------------------
Write-Host '==> zapis stron'
node (Join-Path $PSScriptRoot 'gen_wiki.mjs') $wiki | Out-Null

git add -A
if (-not (git diff --cached --quiet)) {
    git -c 'user.name=tznsec' -c 'user.email=tznsec@users.noreply.github.com' `
        commit -q -m 'Wiki gitools: instalacja, CLI, szablony, migawki, GitHub, budowanie'
    Write-Host '==> commit gotowy'
} else {
    Write-Host '==> brak zmian'
}

git -c $auth push origin master
Pop-Location

Write-Host ''
Write-Host 'Wiki: https://github.com/tznsec/gitools/wiki'