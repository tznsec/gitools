// Skrypt weryfikacyjny: sprawdza spójność UI bez uruchamiania aplikacji.
// Uruchomienie:  node check_ui.js   (z katalogu projektu)

const fs = require('fs');
const path = require('path');

const root = __dirname;
const ui = path.join(root, 'crates', 'gitools', 'ui');

const app = fs.readFileSync(path.join(ui, 'app.js'), 'utf8');
const i18n = fs.readFileSync(path.join(ui, 'i18n.js'), 'utf8');
const html = fs.readFileSync(path.join(ui, 'index.html'), 'utf8');
const gui = fs.readFileSync(path.join(root, 'crates', 'gitools', 'src', 'gui.rs'), 'utf8');

let problems = 0;

// --- 1. Wszystkie id użyte w JS muszą istnieć w HTML ------------------------
const htmlIds = new Set([...html.matchAll(/\bid="([^"]+)"/g)].map((m) => m[1]));
const used = new Set([...app.matchAll(/\$\('([A-Za-z0-9_]+)'\)/g)].map((m) => m[1]));

// elementy budowane dynamicznie w setTab()
for (const name of ['templates', 'snapshots', 'github']) {
  const cap = name.charAt(0).toUpperCase() + name.slice(1);
  for (const prefix of ['panel', 'view', 'bar']) used.add(prefix + cap);
}

const missingIds = [...used].filter((id) => !htmlIds.has(id)).sort();
console.log(`id uzytych w JS: ${used.size}, brakujacych w HTML: ${missingIds.length}`);
missingIds.forEach((id) => console.log(`  BRAK id: ${id}`));
problems += missingIds.length;

// --- 2. Klucze i18n użyte w JS i HTML ---------------------------------------
const usedKeys = new Set([
  ...app.matchAll(/\bt\('([A-Za-z0-9._]+)'/g).map((m) => m[1]),
  ...[...html.matchAll(/data-i18n(?:-[a-z]+)?="([^"]+)"/g)].map((m) => m[1]),
]);

const missingKeys = [...usedKeys].filter((k) => !i18n.includes(`'${k}':`)).sort();
console.log(`kluczy i18n uzytych: ${usedKeys.size}, nieobecnych w i18n.js: ${missingKeys.length}`);
missingKeys.forEach((k) => console.log(`  BRAK klucza: ${k}`));
problems += missingKeys.length;

// --- 3. pl i en muszą mieć te same klucze -----------------------------------
const enStart = i18n.indexOf('en: {');
const plBlock = i18n.slice(i18n.indexOf('pl: {'), enStart);
const enBlock = i18n.slice(enStart, i18n.indexOf('};', enStart));

const keysOf = (block) => new Set([...block.matchAll(/'([A-Za-z0-9._]+)':/g)].map((m) => m[1]));
const plKeys = keysOf(plBlock);
const enKeys = keysOf(enBlock);

const noEn = [...plKeys].filter((k) => !enKeys.has(k)).sort();
const noPl = [...enKeys].filter((k) => !plKeys.has(k)).sort();

console.log(`klucze: pl=${plKeys.size}, en=${enKeys.size}`);
console.log(`  brakujace w en: ${noEn.length}${noEn.length ? ' -> ' + noEn.join(', ') : ''}`);
console.log(`  brakujace w pl: ${noPl.length}${noPl.length ? ' -> ' + noPl.join(', ') : ''}`);
problems += noEn.length + noPl.length;

// --- 4. Komendy IPC używane w JS muszą być obsłużone w Rust ------------------
const cmds = new Set([...app.matchAll(/\bcall\('([a-z_0-9]+)'/g)].map((m) => m[1]));
const handled = new Set([...gui.matchAll(/"([a-z_0-9]+)"\s*=>/g)].map((m) => m[1]));
const noCmd = [...cmds].filter((c) => !handled.has(c)).sort();

console.log(`komend IPC: ${cmds.size}, nieobslugiwanych w gui.rs: ${noCmd.length}`);
noCmd.forEach((c) => console.log(`  BRAK komendy: ${c}`));
problems += noCmd.length;

console.log(problems === 0 ? '\nOK - UI spojne' : `\nPROBLEMOW: ${problems}`);
process.exit(problems === 0 ? 0 : 1);