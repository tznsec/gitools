// Generuje strone GitHub Pages z prawdziwych danych programu.
// Uruchomienie: node tools\gen_site.mjs
//
// Szablony, liczby plików i opcje bierzemy z `gitools list --json` oraz
// `gitools info <id> --json`, wiec strona nie rozjezdza się z programem.

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// Sciezki od pliku skryptu, nie od katalogu roboczego.
const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..');

const EXE = path.join(ROOT, 'target', 'release', 'gitools.exe');
const DOCS = path.join(ROOT, 'docs');

const VERSION = execFileSync(EXE, ['--version'], { encoding: 'utf8' }).trim().replace('gitools ', '');
const templates = JSON.parse(execFileSync(EXE, ['list', '--json'], { encoding: 'utf8' }));
const readme = readFileSync(path.join(ROOT, 'README.md'), 'utf8');

function info(id) {
  return JSON.parse(execFileSync(EXE, ['info', id, '--json'], { encoding: 'utf8' }));
}

/** Plan przy domyślnych opcjach — z niego liczymy pliki i katalogi. */
function plan(id) {
  try {
    const raw = execFileSync(EXE, ['init', id, 'site-probe', '--json', '--dry-run'], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    });
    const parsed = JSON.parse(raw);
    const entries = parsed.entries || [];
    return {
      files: entries.filter((e) => e.kind === 'file'),
      dirs: entries.filter((e) => e.kind === 'dir'),
    };
  } catch {
    return { files: [], dirs: [] };
  }
}

const plural = (n, one, few, many) => {
  const last = n % 10;
  const teen = n % 100 >= 12 && n % 100 <= 14;
  if (n === 1) return one;
  if (last >= 2 && last <= 4 && !teen) return few;
  return many;
};
const files = (n) => `${n} ${plural(n, 'plik', 'pliki', 'plików')}`;
const dirs = (n) => `${n} ${plural(n, 'katalog', 'katalogi', 'katalogów')}`;

/** Bezpieczne wstawianie do HTML. */
const esc = (s) =>
  String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

// ---------------------------------------------------------------- dane ----

const cards = templates.map((t) => {
  const full = info(t.id);
  const { files: f, dirs: d } = plan(t.id);
  const options = (full.options || []).length;
  const choices = (full.options || []).filter((o) => o.kind?.type === 'choice');

  return {
    id: t.id,
    icon: t.icon,
    name: t.name,
    tagline: t.tagline,
    accent: t.accent || '#8b7bff',
    tags: full.tags || [],
    options,
    choices: choices.length,
    fileCount: f.length,
    dirCount: d.length,
    sample: f.slice(0, 3).map((e) => e.path),
  };
});

const totalFiles = cards.reduce((sum, c) => sum + c.fileCount, 0);
const totalOptions = cards.reduce((sum, c) => sum + c.options, 0);
const totalChoices = cards.reduce((sum, c) => sum + c.choices, 0);

// ------------------------------------------------------------- fragmenty ---

const templateCards = cards
  .map(
    (c) => `      <article class="tpl reveal tilt" style="--tint:${esc(c.accent)}">
        <div class="tpl__icon">${c.icon}</div>
        <h3>${esc(c.name)}</h3>
        <p class="tpl__tag">${esc(c.tagline)}</p>
        <div class="tpl__stats">
          <span>${files(c.fileCount)}</span>
          <span>${c.options} ${plural(c.options, 'opcja', 'opcje', 'opcji')}</span>
          ${c.choices ? `<span>${c.choices} ${plural(c.choices, 'lista', 'listy', 'list')}</span>` : ''}
        </div>
        <ul class="tpl__files">${c.sample.map((p) => `<li><code>${esc(p)}</code></li>`).join('')}</ul>
        <code class="tpl__id">gitools init ${esc(c.id)} ./projekt</code>
      </article>`,
  )
  .join('\n');

const navLinks = [
  ['#moznosci', 'Możliwości'],
  ['#szablony', 'Szablony'],
  ['#zrzuty', 'Wygląd'],
  ['#pobieranie', 'Pobieranie'],
  ['#terminal', 'Terminal'],
  ['#faq', 'Pytania'],
];

const site = `<!doctype html>
<html lang="pl" class="no-js">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>gitools — generator struktur plików</title>
<meta name="description" content="gitools tworzy kompletny szkielet projektu jednym poleceniem: ${templates.length} szablonów, migawki, własne szablony JSON i wysyłka na GitHub. Projekt open source autorstwa tznsec." />
<meta property="og:title" content="gitools — generator struktur plików" />
<meta property="og:description" content="Od szablonu do repozytorium. ${templates.length} szablonów, migawki, własne szablony JSON, dwa języki. Projekt open source autorstwa tznsec." />
<meta property="og:type" content="website" />
<meta name="theme-color" content="#0b0c12" />
<link rel="icon" href="logo.svg" type="image/svg+xml" />
<link rel="preload" as="image" href="logo.svg" />
<style>
:root{
  --c1:#6d7cff; --c2:#8b5cf6; --c3:#e05bd0;
  --bg:#0b0c12; --bg-2:#10121b;
  --text:#eef0fa; --muted:#9aa2bb; --dim:#6f7791;
  --line:rgba(255,255,255,.09); --line-2:rgba(255,255,255,.16);
  --glass:rgba(255,255,255,.045);
  --r:18px; --r-sm:11px;
  --mono:'Cascadia Code',Consolas,ui-monospace,SFMono-Regular,Menlo,monospace;
  --sans:'Segoe UI Variable Text','Segoe UI',system-ui,-apple-system,Inter,sans-serif;
  --max:1180px;
}
*{box-sizing:border-box}
html{scroll-behavior:smooth;scroll-padding-top:86px}
body{
  margin:0;font-family:var(--sans);color:var(--text);background:var(--bg);
  line-height:1.65;-webkit-font-smoothing:antialiased;overflow-x:hidden;
}
img{max-width:100%;display:block}
a{color:inherit}

/* --- tło --- */
.aurora{position:fixed;inset:0;z-index:-1;overflow:hidden;pointer-events:none}
.aurora i{
  position:absolute;display:block;border-radius:50%;filter:blur(90px);opacity:.5;
  animation:float 26s ease-in-out infinite;
}
.aurora i:nth-child(1){width:52vw;height:52vw;left:-12vw;top:-14vw;background:radial-gradient(circle,rgba(109,124,255,.85),transparent 70%)}
.aurora i:nth-child(2){width:46vw;height:46vw;right:-10vw;top:-8vw;background:radial-gradient(circle,rgba(224,91,208,.7),transparent 70%);animation-delay:-9s}
.aurora i:nth-child(3){width:60vw;height:50vw;left:24vw;bottom:-30vw;background:radial-gradient(circle,rgba(139,92,246,.55),transparent 70%);animation-delay:-17s}
@keyframes float{0%,100%{transform:translate3d(0,0,0) scale(1)}33%{transform:translate3d(4vw,5vh,0) scale(1.12)}66%{transform:translate3d(-4vw,3vh,0) scale(.94)}}
@media (prefers-reduced-motion:reduce){.aurora i{animation:none}}

/* --- układ --- */
.wrap{max-width:var(--max);margin:0 auto;padding:0 24px}
section{padding:76px 0;border-top:1px solid var(--line)}
h2{font-size:clamp(26px,3.6vw,36px);letter-spacing:-.025em;margin:0 0 10px;font-weight:650}
.lead{color:var(--muted);margin:0 0 34px;max-width:66ch;font-size:17px}
.eyebrow{color:var(--c1);font:600 12px/1 var(--mono);letter-spacing:.18em;text-transform:uppercase;margin:0 0 14px}

/* --- nav --- */
.nav{
  position:sticky;top:0;z-index:50;
  background:rgba(11,12,18,.72);backdrop-filter:blur(18px) saturate(160%);
  border-bottom:1px solid var(--line);
}
.nav__in{display:flex;align-items:center;gap:20px;height:64px}
.nav__brand{display:flex;align-items:center;gap:10px;font-weight:650;letter-spacing:-.01em;text-decoration:none}
.nav__brand img{width:30px;height:30px;border-radius:8px}
.nav__links{display:flex;gap:4px;margin-left:auto;flex-wrap:wrap}
.nav__links a{
  color:var(--muted);text-decoration:none;font-size:14px;padding:7px 12px;border-radius:8px;
  transition:color .18s,background .18s;
}
.nav__links a:hover{color:var(--text);background:var(--glass)}
.nav .btn{padding:9px 16px;font-size:14px}

/* --- przyciski --- */
.btn{
  display:inline-flex;align-items:center;gap:8px;padding:13px 26px;border-radius:var(--r-sm);
  font-weight:600;font-size:15px;text-decoration:none;border:1px solid var(--line-2);
  background:var(--glass);color:var(--text);cursor:pointer;font-family:inherit;
  transition:transform .16s cubic-bezier(.22,1,.36,1),border-color .16s,box-shadow .16s;
}
.btn:hover{transform:translateY(-2px);border-color:rgba(255,255,255,.3)}
.btn--primary{
  background:linear-gradient(100deg,var(--c1),var(--c2) 50%,var(--c3));
  border-color:transparent;color:#fff;box-shadow:0 12px 34px rgba(139,92,246,.34);
}
.btn--ghost{background:transparent}
.btn--sm{padding:9px 16px;font-size:14px}

/* --- hero --- */
.hero{padding:92px 0 76px;text-align:center}
.hero__logo{width:112px;height:112px;margin:0 auto 26px;border-radius:26px;box-shadow:0 24px 70px rgba(0,0,0,.55)}
h1{
  font-size:clamp(42px,8vw,76px);line-height:1.02;letter-spacing:-.04em;margin:0 0 18px;font-weight:700;
  background:linear-gradient(100deg,var(--c1),var(--c2) 45%,var(--c3));
  -webkit-background-clip:text;background-clip:text;color:transparent;
}
.hero__sub{font-size:clamp(17px,2.4vw,22px);color:var(--muted);margin:0 auto 14px;max-width:60ch}
.hero__by{color:var(--dim);font-size:14px;margin:0 0 32px}
.hero__by a{color:var(--muted)}
.hero .cta{display:flex;gap:12px;justify-content:center;flex-wrap:wrap}

/* --- pasek statystyk --- */
.stats{display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:1px;background:var(--line);border:1px solid var(--line);border-radius:var(--r);overflow:hidden;margin-top:52px}
.stats div{background:var(--bg);padding:22px 18px;text-align:center}
.stats b{display:block;font-size:30px;letter-spacing:-.03em;background:linear-gradient(120deg,var(--c1),var(--c3));-webkit-background-clip:text;background-clip:text;color:transparent}
.stats span{font-size:12.5px;color:var(--dim)}

/* --- karty --- */
.grid{display:grid;gap:14px}
.grid--3{grid-template-columns:repeat(auto-fit,minmax(280px,1fr))}
.card{
  padding:24px;border-radius:var(--r);border:1px solid var(--line);background:var(--glass);
  transition:transform .18s cubic-bezier(.22,1,.36,1),border-color .18s;
}
.card:hover{transform:translateY(-3px);border-color:var(--line-2)}
.card h3{margin:0 0 8px;font-size:17px;display:flex;align-items:center;gap:10px}
.card p{margin:0;color:var(--muted);font-size:14.5px}
.card__ico{
  width:38px;height:38px;border-radius:10px;display:grid;place-items:center;font-size:19px;flex:none;
  background:linear-gradient(140deg,color-mix(in srgb,var(--c1) 26%,transparent),color-mix(in srgb,var(--c3) 20%,transparent));
}

/* --- szablony --- */
.tpls{display:grid;grid-template-columns:repeat(auto-fill,minmax(258px,1fr));gap:14px}
.tpl{
  padding:20px;border-radius:var(--r);border:1px solid var(--line);
  background:linear-gradient(180deg,color-mix(in srgb,var(--tint) 7%,transparent),transparent 60%),var(--glass);
  transition:transform .18s cubic-bezier(.22,1,.36,1),border-color .18s;
  display:flex;flex-direction:column;
}
.tpl:hover{transform:translateY(-4px);border-color:color-mix(in srgb,var(--tint) 45%,var(--line))}
.tpl__icon{
  width:42px;height:42px;border-radius:11px;display:grid;place-items:center;font-size:21px;margin-bottom:13px;
  background:color-mix(in srgb,var(--tint) 18%,transparent);
  border:1px solid color-mix(in srgb,var(--tint) 30%,transparent);
}
.tpl h3{margin:0 0 4px;font-size:16.5px}
.tpl__tag{margin:0 0 13px;color:var(--muted);font-size:13.5px;min-height:2.6em}
.tpl__stats{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:13px}
.tpl__stats span{font:11.5px/1 var(--mono);color:var(--dim);border:1px solid var(--line);padding:5px 9px;border-radius:999px}
.tpl__files{margin:0 0 14px;padding:0;list-style:none;flex:1}
.tpl__files li{font:11.5px/1.7 var(--mono);color:var(--dim);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.tpl__files code{background:none}
.tpl__id{
  font:11.5px/1 var(--mono);color:var(--muted);background:rgba(0,0,0,.34);
  border:1px solid var(--line);border-radius:8px;padding:9px 10px;display:block;overflow-x:auto;white-space:nowrap;
}

/* --- zrzuty --- */
.shots{display:grid;gap:18px}
.shot{border:1px solid var(--line);border-radius:var(--r);overflow:hidden;background:var(--bg-2)}
.shot img{width:100%;border:0}
.shot figcaption{padding:13px 18px;border-top:1px solid var(--line);color:var(--muted);font-size:14px}
.shot figcaption b{color:var(--text)}

/* --- pobieranie --- */
.dl{display:grid;gap:12px}
.dl__item{
  display:flex;align-items:center;gap:18px;flex-wrap:wrap;justify-content:space-between;
  padding:20px 22px;border-radius:var(--r);border:1px solid var(--line);background:var(--glass);
}
.dl__item b{display:block;font-size:16.5px;margin-bottom:2px}
.dl__item small{color:var(--dim);font:12.5px/1.6 var(--mono)}
.req{display:flex;flex-wrap:wrap;gap:8px;margin-top:26px}
.req span{
  font:12px/1 var(--mono);color:var(--muted);border:1px solid var(--line);
  background:var(--glass);padding:7px 12px;border-radius:999px;
}

/* --- kod --- */
.code{
  position:relative;background:rgba(0,0,0,.42);border:1px solid var(--line);border-radius:var(--r);
  overflow:hidden;
}
.code__bar{display:flex;gap:6px;align-items:center;padding:11px 14px;border-bottom:1px solid var(--line)}
.code__bar i{width:11px;height:11px;border-radius:50%;background:var(--line-2)}
.code__bar i:nth-child(1){background:#ff5f57}.code__bar i:nth-child(2){background:#febc2e}.code__bar i:nth-child(3){background:#28c840}
.code__bar span{margin-left:8px;color:var(--dim);font:12px var(--mono)}
.code__bar button{
  margin-left:auto;background:var(--glass);border:1px solid var(--line);color:var(--muted);
  border-radius:7px;padding:5px 11px;font:12px var(--sans);cursor:pointer;transition:all .16s;
}
.code__bar button:hover{color:var(--text);border-color:var(--line-2)}
pre{margin:0;padding:18px 20px;overflow-x:auto;font:13.5px/1.75 var(--mono);color:#d7dbf0}
pre .c{color:var(--dim)}pre .p{color:#c4a7ff}pre .s{color:#8ee6b8}

/* --- FAQ --- */
.faq{display:grid;gap:10px}
details{
  border:1px solid var(--line);border-radius:var(--r-sm);background:var(--glass);overflow:hidden;
}
summary{padding:16px 20px;cursor:pointer;font-weight:600;font-size:15px;list-style:none;display:flex;justify-content:space-between;gap:14px}
summary::-webkit-details-marker{display:none}
summary::after{content:'+';color:var(--dim);font:400 20px/1 var(--mono);transition:transform .2s}
details[open] summary::after{content:'−'}
details p{margin:0;padding:0 20px 18px;color:var(--muted);font-size:14.5px}
details code{font:13px var(--mono);background:rgba(0,0,0,.3);padding:2px 6px;border-radius:5px}

footer{padding:46px 0 60px;border-top:1px solid var(--line);text-align:center;color:var(--dim);font-size:14px}
footer a{color:var(--muted)}
footer .lic{margin-top:8px;font:12.5px var(--mono)}

@media (max-width:720px){
  .nav__links{display:none}
  .hero{padding:60px 0 50px}
  section{padding:54px 0}
}
</style>
</head>
<body>

<!-- Splash: logo rysowane od zera, litery po kolei, pasek postepu. -->
<div class="splash" id="splash" role="status" aria-label="Ładowanie gitools">
  <div class="splash__inner">
    <svg class="splash__logo" viewBox="0 0 64 64" aria-hidden="true">
      <g fill="none" stroke="#eef0fa" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="32" cy="18" r="8.2" style="--len:52" />
        <path d="M32 24 L32 42" style="--len:18" />
        <path d="M21 46 L43 46" style="--len:22" />
        <path d="M24 46 L24 50" style="--len:4" />
        <path d="M32 46 L32 52.5" style="--len:6.5" />
        <path d="M40 46 L40 49.5" style="--len:3.5" />
      </g>
    </svg>
    <div class="splash__name">gitools</div>
    <div class="splash__tag">generator struktur plików</div>
    <div class="splash__bar"><i></i></div>
  </div>
</div>

<div class="progress" aria-hidden="true"></div>
<div class="hero-scrim" aria-hidden="true"></div>
<div class="particles" aria-hidden="true"></div>
<div class="cursor-glow" aria-hidden="true"></div>
<div class="aurora"><i></i><i></i><i></i></div>

<nav class="nav">
  <div class="wrap nav__in">
    <a class="nav__brand" href="#top"><img src="logo.svg" alt="" /> gitools</a>
    <div class="nav__links">
${navLinks.map(([h, l]) => `      <a href="${h}">${l}</a>`).join('\n')}
    </div>
    <a class="btn btn--primary btn--sm magnet" data-magnet="14" href="downloads/gitools-setup-${VERSION}.exe">⬇ Pobierz</a>
  </div>
</nav>

<header class="hero wrap" id="top">
  <img class="hero__logo" src="logo.svg" alt="gitools" />
  <h1>gitools</h1>
  <p class="hero__sub">Generator struktur plików — od szablonu do repozytorium.</p>
  <p class="hero__by">Projekt open source autorstwa <a href="https://github.com/tznsec">tznsec</a> · wersja ${VERSION} · MIT</p>

  <div class="cta">
    <a class="btn btn--primary magnet" data-magnet="10" href="downloads/gitools-setup-${VERSION}.exe">⬇ Pobierz instalator</a>
    <a class="btn" href="https://github.com/tznsec/gitools/releases">Wszystkie wydania</a>
    <a class="btn btn--ghost" href="https://github.com/tznsec/gitools">Kod źródłowy</a>
  </div>

  <div class="stats reveal">
    <div class="stat"><b data-count="${templates.length}">0</b><span>szablonów</span></div>
    <div class="stat"><b data-count="${totalFiles}">0</b><span>plików w domyślnych wariantach</span></div>
    <div class="stat"><b data-count="${totalOptions}">0</b><span>opcji do przekonfigurowania</span></div>
    <div class="stat"><b data-count="${totalChoices}">0</b><span>list wyboru</span></div>
    <div class="stat"><b data-count="5">0</b><span>motywów interfejsu</span></div>
  </div>
</header>

<!-- Pasek przewijanych szablonów -->
<div class="marquee" aria-hidden="true">
  <div class="marquee__row">
${cards.concat(cards).map((c) => `    <span class="marquee__item">${c.icon} <b>${esc(c.name)}</b> · ${files(c.fileCount)}</span>`).join('\n')}
  </div>
</div>

<section class="wrap reveal" id="moznosci">
  <p class="eyebrow reveal">Możliwości</p>
  <h2 class="reveal scramble">Robi całą robotę, nie tylko część</h2>
  <p class="lead reveal">
    Plan powstaje osobno od zapisu, więc podgląd, <code style="font-family:var(--mono);font-size:.9em">--dry-run</code>
    i prawdziwe tworzenie plików idą tą samą drogą. Nie ma osobnej ścieżki kodu,
    która mogłaby się rozjechać z tym, co naprawdę powstaje na dysku.
  </p>

  <div class="grid grid--3">
    <article class="card reveal tilt"><div class="card__ico">🧩</div><h3>${templates.length} szablonów</h3>
      <p>Rust, Node.js, React, Vue, Svelte, Astro, Electron, Tauri, .NET, Flutter, Python, Go i pusty katalog bazowy. Każdy z własnymi opcjami.</p></article>
    <article class="card reveal tilt"><div class="card__ico">📸</div><h3>Migawki</h3>
      <p>Kopia wygenerowanego projektu przed każdą zmianą. Podgląd plik po pliku, przywracanie do wybranego katalogu, tryb automatyczny z limitem.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🐙</div><h3>GitHub</h3>
      <p>Zaloguj się tokenem, załóż repozytorium i wrzuć cały projekt <b>jednym commitem</b> — także pliki binarne i instalator.</p></article>
    <article class="card reveal tilt"><div class="card__ico">📄</div><h3>Własne szablony JSON</h3>
      <p>Dopisz definicję w JSON — GUI i terminal, bez ruszania kodu. Zły szablon nie blokuje aplikacji.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🎨</div><h3>Pięć wyglądów</h3>
      <p>Szkło, ciemny, jasny i dwa kreskówkowe, cztery palety akcentu, sterowana gęstość i animacje.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🌍</div><h3>Dwa języki</h3>
      <p>Cały interfejs po polsku i angielsku — także nazwy szablonów, opcji i grupy ustawień.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🔒</div><h3>Bezpieczne nadpisywanie</h3>
      <p>Bez <code style="font-family:var(--mono);font-size:.9em">--force</code> żaden istniejący plik nie zostanie ruszony. Uprawnienia katalogu sprawdzane wprost.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🌱</div><h3>Polskie nazwy</h3>
      <p>„Łąka Żubra” daje <code style="font-family:var(--mono);font-size:.9em">laka-zubra</code>, <code style="font-family:var(--mono);font-size:.9em">laka_zubra</code> i <code style="font-family:var(--mono);font-size:.9em">LakaZubra</code>.</p></article>
    <article class="card reveal tilt"><div class="card__ico">🧪</div><h3>Testowalny rdzeń</h3>
      <p>77 testów obejmuje plan, wszystkie szablony, migawki, logo i klienta GitHub — w tym przypadek, że token wyciekłby do logów.</p></article>
  </div>
</section>

<section class="wrap reveal" id="szablony">
  <p class="eyebrow reveal">Szablony</p>
  <h2 class="reveal scramble">Co dokładnie dostajesz</h2>
  <p class="lead reveal">
    Liczby plików policzone są z prawdziwego planu wygenerowanego przez
    <code style="font-family:var(--mono);font-size:.9em">gitools init &lt;id&gt; --json --dry-run</code>,
    więc są zgodne z tym, co naprawdę wyląduje na dysku.
  </p>
  <div class="tpls">
${templateCards}
  </div>
</section>

<section class="wrap reveal" id="zrzuty">
  <p class="eyebrow reveal">Wygląd</p>
  <h2 class="reveal scramble">Jedno okno, trzy zakładki</h2>
  <p class="lead reveal">Aplikacja to natywny okno WebView2 z interfejsem w jednym pliku HTML — bez Electrona i bez bundlera.</p>

  <div class="shots">
    <figure class="shot" style="margin:0">
      <img src="img/templates.jpg" alt="Zakładka Szablony — lista po lewej, opcje i podgląd struktury po prawej" loading="lazy" />
      <figcaption><b>Szablony</b> — lista z wyszukiwaniem po lewej, a po prawej katalog docelowy, dane projektu, opcje szablonu i podgląd struktury plików z zawartością.</figcaption>
    </figure>
    <figure class="shot" style="margin:0">
      <img src="img/snapshots.jpg" alt="Zakładka Migawki — siatka zapisanych stanów projektu" loading="lazy" />
      <figcaption><b>Migawki</b> — zapisane stany projektów, podgląd zawartości, przywracanie i ustawienia migawek automatycznych.</figcaption>
    </figure>
    <figure class="shot" style="margin:0">
      <img src="img/github.jpg" alt="Zakładka GitHub — logowanie tokenem, zakładanie repozytorium i wysyłka projektu" loading="lazy" />
      <figcaption><b>GitHub</b> — logowanie tokenem dostępu, zakładanie repozytorium i wysyłka całego katalogu jednym commitem.</figcaption>
    </figure>
  </div>
</section>

<section class="wrap reveal" id="pobieranie">
  <p class="eyebrow reveal">Pobieranie</p>
  <h2 class="reveal scramble">Instalator albo wersja przenośna</h2>
  <p class="lead reveal">
    Instalator to jeden plik <code style="font-family:var(--mono);font-size:.9em">.exe</code> napisany w Rust —
    rozpakowuje program, zakłada skrót w Menu Start i dodaje wpis na liście zainstalowanych aplikacji.
    Nie wymaga Inno Setup, NSIS ani WiX.
  </p>

  <div class="dl">
    <div class="dl__item">
      <div><b>Instalator (.exe)</b><small>gitools-setup-${VERSION}.exe · ~4,8 MB · Windows 10/11</small></div>
      <a class="btn btn--primary" href="downloads/gitools-setup-${VERSION}.exe">⬇ Pobierz</a>
    </div>
    <div class="dl__item">
      <div><b>Wersja przenośna (.zip)</b><small>gitools-${VERSION}-portable.zip · ~1,9 MB · rozpakuj i uruchom</small></div>
      <a class="btn" href="downloads/gitools-${VERSION}-portable.zip">⬇ Pobierz</a>
    </div>
    <div class="dl__item">
      <div><b>Kod źródłowy</b><small>git clone https://github.com/tznsec/gitools.git</small></div>
      <a class="btn btn--ghost" href="https://github.com/tznsec/gitools">GitHub</a>
    </div>
  </div>

  <div class="req">
    <span>Windows 10 / 11</span><span>WebView2 Runtime</span><span>Rust 1.85+ przy budowaniu</span><span>MIT</span><span>Bez konta</span><span>Działa offline</span>
  </div>
</section>

<section class="wrap reveal" id="terminal">
  <p class="eyebrow reveal">Terminal</p>
  <h2 class="reveal scramble">Wszystko, co robi okno, robi też powłoka</h2>
  <p class="lead reveal">Te same pliki, ten sam plan. Do skryptów i CI bez okienek.</p>

  <div class="code">
    <div class="code__bar"><i></i><i></i><i></i><span>terminal</span><button onclick="navigator.clipboard?.writeText(this.closest('.code').querySelector('pre').innerText)">kopiuj</button></div>
<pre><span class="c"># lista szablonów</span>
gitools list

<span class="c"># projekt w Rust z testami i licencją MIT</span>
gitools init rust ./moj-projekt -N <span class="s">"Mój Projekt"</span> -o kind=library -o tests

<span class="c"># sprawdzić co się stanie, bez pisania czegokolwiek</span>
gitools init node ./api --dry-run

<span class="c"># migawki</span>
gitools snapshots take ./moj-projekt
gitools snapshots list

<span class="c"># własny szablon</span>
gitools templates example &gt; moj.json
gitools templates add moj.json

<span class="c"># GitHub — cały projekt jednym commitem</span>
gitools github login
gitools github create moj-projekt -d <span class="s">"Opis"</span>
gitools github push tznsec/moj-projekt ./moj-projekt -m <span class="s">"Pierwszy commit"</span></pre>
  </div>
</section>

<section class="wrap reveal" id="faq">
  <p class="eyebrow reveal">Pytania</p>
  <h2 class="reveal scramble">Co warto wiedzieć</h2>
  <div class="faq">
    <details class="reveal"><summary>Czy muszę się logować do konta?</summary>
      <p>Nie. Wszystko działa offline i bez konta. Token GitHub jest potrzebny dopiero wtedy, kiedy chcesz zakładać repozytoria i wysyłać projekty.</p></details>
    <details class="reveal"><summary>Czy nadpisze mi pliki?</summary>
      <p>Nie. Bez <code>--force</code> gitools kończy się listą konfliktów i zapisuje tylko to, czego nie ma. Z <code>--force</code> nadpisuje świadomie — i mówi o tym w podsumowaniu.</p></details>
    <details class="reveal"><summary>Co to jest migawka i po mi?</summary>
      <p>Migawka to zwykły katalog z plikami projektu plus metadane (użyty szablon, data, opcje). Dzięki temu przetrwa usunięcie samego projektu. Możesz podejrzeć pliki przed przywróceniem i wybrać katalog docelowy inny niż oryginalny.</p></details>
    <details class="reveal"><summary>Jak dodać własny szablon?</summary>
      <p>Plik JSON wklejony w <b>Ustawienia → Własne szablony</b> albo <code>gitools templates add plik.json</code>. Pełny format opisany jest w <a href="https://github.com/tznsec/gitools/wiki/Wlasne-szablony" style="color:var(--c1)">wiki</a>. Szablon widzi zmienne <code>{{project_name}}</code>, <code>{{slug}}</code>, <code>{{snake}}</code>, <code>{{Pascal}}</code> i warunki <code>{{#if opcja}}</code>.</p></details>
    <details class="reveal"><summary>Czy wysyłka na GitHub jest bezpieczna dla tokenu?</summary>
      <p>Token to hasło do konta, więc nigdy nie trafia w komunikaty błędów ani logi — <code>github whoami</code> pokazuje tylko liczbę znaków. Przechowujemy go lokalnie w <code>%LOCALAPPDATA%\\gitools\\github.json</code>. Szczegóły w <a href="https://github.com/tznsec/gitools/wiki/GitHub" style="color:var(--c1)">wiki</a>.</p></details>
    <details class="reveal"><summary>Czy jest dokumentacja?</summary>
      <p>Tak — <a href="https://github.com/tznsec/gitools/wiki" style="color:var(--c1)">wiki projektu</a>: instalacja, wszystkie polecenia, 13 szablonów z opcjami, migawki, własne szablony, GitHub, budowanie ze źródeł i rozwiązywanie problemów.</p></details>
    <details class="reveal"><summary>Czy mogę się przyczynić?</summary>
      <p>Tak — issues i pull requesty na <a href="https://github.com/tznsec/gitools" style="color:var(--c1)">GitHubie</a>. Projekt jest na licencji MIT.</p></details>
  </div>
</section>

<footer class="wrap">
  <p>
    <a href="https://github.com/tznsec/gitools">github.com/tznsec/gitools</a> ·
    <a href="https://github.com/tznsec/gitools/wiki">wiki</a> ·
    <a href="https://github.com/tznsec/gitools/releases">wydania</a> ·
    autor: <a href="https://github.com/tznsec">tznsec</a>
  </p>
  <p class="lic">gitools ${VERSION} · MIT · kod, zrzuty i strona wygenerowane z tego samego źródła co program</p>
</footer>

</body>
</html>
`;

// CSS wypisujemy do osobnego pliku: w HTML zostają tylko dwa <link>.
// Dzięki temu przeglądarka cache'uje arkusz, a animacje z motion.css
// dociągają się w jednym miejscu.
const styleStart = site.indexOf('<style>');
const styleEnd = site.indexOf('</style>');
const css = site.slice(styleStart + '<style>'.length, styleEnd);
writeFileSync(path.join(DOCS, 'style.css'), css.trim() + '\n', 'utf8');

const html = [
  site.slice(0, styleStart),
  '<link rel="stylesheet" href="style.css" />\n<link rel="stylesheet" href="motion.css" />\n<script src="motion.js" defer></script>',
  site.slice(styleEnd + '</style>'.length),
].join('');

writeFileSync(path.join(DOCS, 'index.html'), html, 'utf8');

console.log(`docs/index.html — ${templates.length} szablonow, ${files(totalFiles)}, ${totalOptions} opcji`);
console.log(`docs/style.css — ${css.length} B, docs/motion.css — animacje i splash`);
console.log('README.skrot:', readme.length, 'znakow (uzywany jako zrodlo wersji)');