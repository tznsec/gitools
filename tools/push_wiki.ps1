# Wypycha wiki na https://github.com/tznsec/gitools.wiki.git
#
# GitHub NIE MA API do tworzenia wiki — repo `.wiki.git` powstaje dopiero po
# kliknięciu „Create the first page” w przeglądarce. Potem zwykły git wystarczy.
#
#   1. Wejdź na https://github.com/tznsec/gitools/wiki
#   2. Kliknij „Create the first page”, wpisz tytuł `Home`, treść `start`
#   3. Kliknij „Save Page”
#   4. Uruchom ten skrypt
#
# Token nigdy nie trafia na dysk — idzie w nagłówku HTTP procesu git.

param(
    [string]$Token = $env:GITHUB_TOKEN
)

$ErrorActionPreference = 'Stop'
$wiki = Join-Path $PSScriptRoot '..\wiki'

if (-not $Token) {
    Write-Error "Brak tokena. Ustaw GITHUB_TOKEN albo podaj -Token."
    exit 1
}

Push-Location $wiki

# Treść generujemy z programu, żeby opcje szablonów nie rozjechały się z kodem.
Write-Host '==> generowanie stron wiki'
node (Join-Path $PSScriptRoot 'gen_wiki.mjs') $wiki | Out-Host

git add -A
git -c 'user.name=tznsec' -c 'user.email=tznsec@users.noreply.github.com' `
    commit -q -m 'Wiki gitools: instalacja, CLI, szablony, migawki, GitHub, budowanie'

# x-access-token zamiast hasła w URL — token nie zapisuje się w .git/config
$basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("x-access-token:$Token"))

git -c "http.https://github.com/.extraheader=AUTHORIZATION: basic $basic" `
    push origin master

Pop-Location

Write-Host ''
Write-Host 'Wiki: https://github.com/tznsec/gitools/wiki'