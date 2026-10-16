/* gitools - warstwa interfejsu (czysty JS, bez zależności) */
'use strict';

// ---------------------------------------------------------- ustawienia ----
// (deklaracje muszą być przed `state`, inaczej padnie na TDZ)

const PALETTES = [
  { name: 'Fiolet', c1: '#6d7cff', c2: '#8b5cf6', c3: '#e05bd0' },
  { name: 'Zafir', c1: '#38bdf8', c2: '#3b82f6', c3: '#818cf8' },
  { name: 'Ember', c1: '#fbbf24', c2: '#f97316', c3: '#ec4899' },
  { name: 'Mięta', c1: '#34d399', c2: '#10b981', c3: '#3b82f6' },
];

const DEFAULT_SETTINGS = {
  palette: 0,
  custom: null,
  look: 'glass',
  aurora: 100,
  blur: 22,
  motion: true,
  density: 'normal',
  lang: 'pl',
};

const STORAGE_KEY = 'gitools.settings';

const LOOKS = ['glass', 'dark', 'light', 'sketch-dark', 'sketch-light'];

function loadSettings() {
  const defaults = { ...DEFAULT_SETTINGS };

  let saved = null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    saved = raw ? JSON.parse(raw) : null;
  } catch {
    saved = null;
  }

  if (!saved || typeof saved !== 'object') return defaults;

  // bierzemy tylko znane klucze - uszkodzone dane nie psują aplikacji
  const merged = { ...defaults };
  for (const key of Object.keys(defaults)) {
    if (key in saved && saved[key] !== undefined && saved[key] !== null) {
      merged[key] = saved[key];
    }
  }
  if (!LOOKS.includes(merged.look)) merged.look = defaults.look;
  if (typeof merged.palette !== 'number') merged.palette = defaults.palette;
  merged.aurora = Math.min(150, Math.max(0, Number(merged.aurora) || 0));
  merged.blur = Math.min(40, Math.max(0, Number(merged.blur) || 0));
  merged.motion = Boolean(merged.motion);
  if (!['normal', 'compact'].includes(merged.density)) merged.density = defaults.density;
  if (!LANGS.some((l) => l.code === merged.lang)) merged.lang = defaults.lang;

  return merged;
}

function saveSettings() {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state.settings));
  } catch {
    /* brak dostępu do pamięci - ustawienia zostaną tylko w tej sesji */
  }
}

// ------------------------------------------------------------ mostek IPC --

let seq = 1;
const pending = new Map();

/**
 * Koperta żądania ma postać `{ id, cmd, payload }`.
 *
 * Dane trzymamy w zagnieżdżonym `payload`, a nie „rozsypane" na górze:
 * wcześniejsza wersja składała `{ id, cmd, ...payload }`, więc każde
 * polecenie z kluczem `id` (migawka, własny szablon) nadpisywało numer
 * żądania — odpowiedź nigdy nie wracała i przycisk wyglądał na martwy.
 */
function call(cmd, payload = {}) {
  return new Promise((resolve, reject) => {
    const id = seq++;
    pending.set(id, { resolve, reject });
    window.ipc.postMessage(JSON.stringify({ id, cmd, payload }));
  });
}

window.__sg_reply = function (response) {
  const entry = pending.get(response.id);
  if (!entry) return;
  pending.delete(response.id);
  if (response.ok) entry.resolve(response.data);
  else entry.reject(new Error(response.error || 'nieznany błąd'));
};

// ----------------------------------------------------------------- stan ---

const state = {
  templates: [],
  userTemplateIds: [],
  selected: null,
  options: {},
  entries: [],
  pickerPath: '',
  generated: null,
  tab: 'templates',
  snapshots: [],
  snapSettings: { auto: true, keep: 20 },
  snapDir: '',
  openSnapshot: null,
  filePickerPath: '',
  ghUser: null,
  ghRepos: [],
  pickerOnPick: null,
  settings: loadSettings(),
};

// ------------------------------------------------------------- pomocnicze -

const $ = (id) => document.getElementById(id);

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

/** Powtarzalna animacja przy zmianie tekstu (np. licznik plików). */
function bump(node) {
  if (!node) return;
  node.classList.remove('bump');
  void node.offsetWidth;
  node.classList.add('bump');
}

/** Poświat szklanej karty podświetlający pozycję kursora. */
function trackSpotlight(root) {
  root.addEventListener(
    'pointermove',
    (event) => {
      const rect = root.getBoundingClientRect();
      root.style.setProperty('--mx', `${event.clientX - rect.left}px`);
      root.style.setProperty('--my', `${event.clientY - rect.top}px`);
    },
    { passive: true },
  );
}

const DIACRITICS = {
  ą: 'a', ć: 'c', ę: 'e', ł: 'l', ń: 'n', ó: 'o', ś: 's', ź: 'z', ż: 'z',
};

function words(input) {
  let normalized = '';
  for (const ch of String(input || '').toLowerCase()) {
    normalized += DIACRITICS[ch] || ch;
  }

  const parts = [];
  let current = '';
  let prevLower = false;

  for (const ch of normalized) {
    const isUpper = ch !== ch.toLowerCase();
    const isDigit = ch >= '0' && ch <= '9';
    const isSep = !/[a-z0-9]/.test(ch);

    if (isSep) {
      if (current) parts.push(current);
      current = '';
      prevLower = false;
      continue;
    }

    if (isUpper && prevLower && current) {
      parts.push(current);
      current = '';
    }

    current += ch;
    prevLower = !isUpper || isDigit;
  }

  if (current) parts.push(current);
  return parts;
}

const slugify = (input) => words(input).join('-') || 'nowy-projekt';
const snake = (input) => words(input).join('_') || 'nowy_projekt';
const pascal = (input) =>
  words(input)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join('') || 'NowyProjekt';

function humanSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' kB';
  return (bytes / 1024 / 1024).toFixed(2) + ' MB';
}

/** Skraca długą ścieżkę do dwóch ostatnich segmentów — reszta trafia w tooltip. */
function shortPath(path) {
  const parts = String(path || '').split(/[\\/]/).filter(Boolean);
  if (parts.length <= 2) return path || '';
  return '…\\' + parts.slice(-2).join('\\');
}

/** Wybiera pole zależne od języka, z bezpiecznym spadkiem do polskiego. */
function pick(obj, field) {
  if (lang === 'en') {
    const value = obj[`${field}_en`];
    if (value) return value;
  }
  return obj[field] ?? '';
}

const templateName = (template) => pick(template, 'name');
const templateTagline = (template) => pick(template, 'tagline');
const templateDescription = (template) => pick(template, 'description');
const optionLabel = (option) => pick(option, 'label');
const optionHelp = (option) => pick(option, 'help');
const optionGroup = (option) => pick(option, 'group');

// ---------------------------------------------------------------- toasty --

function toast(title, body, kind = 'ok', ttl = 5200) {
  const node = el('div', 'toast toast--' + kind);
  node.appendChild(el('div', 'toast__title', title));
  if (body) node.appendChild(el('div', 'toast__body', body));
  $('toasts').appendChild(node);
  if (ttl > 0) setTimeout(() => node.remove(), ttl);
  else node.querySelector('.toast__title').after(el('div', 'toast__body', t('toast.dismiss')));
  node.addEventListener('click', () => node.remove());
}

/** Błąd krytyczny - pokazujemy go na stałe, żeby nie zniknął po chwili. */
function fatal(message) {
  const box = $('fatal');
  box.hidden = false;
  box.textContent = String(message);
}

window.addEventListener('error', (event) => {
  fatal(event.message + (event.filename ? ` (${event.filename}:${event.lineno})` : ''));
});

window.addEventListener('unhandledrejection', (event) => {
  fatal(t('fatal.unhandled', { msg: event.reason?.message || event.reason }));
});

// ------------------------------------------------------------ szablony ---

function renderTemplateList() {
  renderTemplates($('search').value);
}

function renderTemplates(filter = '') {
  const list = $('templates');
  list.textContent = '';

  const needle = filter.trim().toLowerCase();
  const found = state.templates.filter((t) => {
    if (!needle) return true;
    return (
      t.name.toLowerCase().includes(needle) ||
      t.id.includes(needle) ||
      t.tagline.toLowerCase().includes(needle) ||
      t.tags.some((tag) => tag.toLowerCase().includes(needle))
    );
  });

  if (!found.length) {
    list.appendChild(el('div', 'empty', t('empty.noTemplates')));
    return;
  }

  for (const [index, template] of found.entries()) {
    const button = el('button', 'template');
    button.type = 'button';
    button.style.setProperty('--i', String(index));
    if (state.selected && state.selected.id === template.id) {
      button.classList.add('template--active');
    }

    const icon = el('div', 'template__icon', template.icon);
    icon.style.borderColor = template.accent + '55';

    const body = el('div', 'template__body');
    const name = el('div', 'template__name');
    name.appendChild(document.createTextNode(template.name));
    if (state.userTemplateIds.includes(template.id)) {
      name.appendChild(el('span', 'template__own', t('badge.own')));
    }
    body.appendChild(name);
    body.appendChild(el('div', 'template__tagline', templateTagline(template)));

    button.appendChild(icon);
    button.appendChild(body);
    button.addEventListener('click', () => selectTemplate(template.id));
    list.appendChild(button);
  }
}

function selectTemplate(id) {
  const template = state.templates.find((t) => t.id === id);
  if (!template) return;

  state.selected = template;
  state.options = {};
  for (const option of template.options) {
    state.options[option.key] = option.default ?? '';
  }

  $('heroIcon').textContent = template.icon;
  $('heroTitle').textContent = templateName(template);
  $('heroDesc').textContent = templateDescription(template);

  const tags = $('heroTags');
  tags.textContent = '';
  for (const tag of template.tags) tags.appendChild(el('span', 'tag', tag));

  // Podpowiedź nazwy projektu, jeśli użytkownik jeszcze nic nie wpisał.
  if (!$('pname').dataset.touched) {
    $('pname').placeholder = t('field.namePlaceholder');
  }
  if (!$('pname').value.trim()) {
    suggestRoot();
  }

  renderTemplates($('search').value);
  renderOptions();
  renderSnapshotSource();
  refresh();
}

/** Aktualizuje opis źródła w widoku migawek po zmianie szablonu/katalogu. */
function renderSnapshotSource() {
  const node = $('snapSource');
  if (node) node.textContent = snapshotSourceLabel();
}

function suggestRoot() {
  const template = state.selected;
  if (!template) return;

  const name = $('pname').value.trim();
  const candidate = name ? slugify(name) : null;
  const root = $('root').value;

  if (candidate && (!root || root === '__AUTO__')) {
    $('root').value = state.home + '\\' + candidate;
  }
}

// --------------------------------------------------------------- opcje ---

function renderOptions() {
  const box = $('options');
  box.textContent = '';

  const template = state.selected;
  if (!template) return;

  const groups = new Map();
  for (const option of template.options) {
    const group = optionGroup(option);
    if (!groups.has(group)) groups.set(group, []);
    groups.get(group).push(option);
  }

  for (const [name, options] of groups) {
    const group = el('div', 'group');
    group.appendChild(el('div', 'group__title', name));

    for (const option of options) {
      group.appendChild(renderOption(option));
    }

    box.appendChild(group);
  }
}

function renderOption(option) {
  const row = el('div', 'opt');

  const left = el('div');
  left.appendChild(el('div', 'opt__label', optionLabel(option)));
  const help = optionHelp(option);
  if (help) left.appendChild(el('div', 'opt__help', help));

  if (option.kind.type === 'toggle') {
    const wrap = el('label', 'switch');
    const input = el('input');
    input.type = 'checkbox';
    input.checked = ['true', '1', 'yes', 'on'].includes(String(state.options[option.key]));
    input.addEventListener('change', () => {
      state.options[option.key] = input.checked ? 'true' : 'false';
      refresh();
    });
    wrap.appendChild(input);
    wrap.appendChild(el('span', 'switch__track'));
    row.appendChild(left);
    row.appendChild(wrap);
  } else {
    const segment = el('div', 'segment');
    for (const value of option.kind.values) {
      const button = el('button', state.options[option.key] === value.value ? 'on' : '', value.label);
      button.type = 'button';
      button.addEventListener('click', () => {
        state.options[option.key] = value.value;
        renderOptions();
        refresh();
      });
      segment.appendChild(button);
    }
    row.appendChild(left);
    row.appendChild(segment);
  }

  return row;
}

// -------------------------------------------------------------- podgląd --

function buildTree(entries) {
  const root = { name: '', dirs: new Map(), files: [] };

  for (const entry of entries) {
    const parts = entry.path.split('/');
    const name = parts.pop();
    let node = root;

    for (const part of parts) {
      if (!node.dirs.has(part)) node.dirs.set(part, { name: part, dirs: new Map(), files: [] });
      node = node.dirs.get(part);
    }

    if (entry.kind === 'dir') {
      if (!node.dirs.has(name)) node.dirs.set(name, { name, dirs: new Map(), files: [] });
    } else {
      node.files.push({ name, entry });
    }
  }

  return root;
}

function renderTree(node, depth, target, conflicts, onFile) {
  const items = [
    ...[...node.dirs.values()].map((dir) => ({ name: dir.name, dir })),
    ...node.files,
  ].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));

  for (const item of items) {
    const row = el('div', 'tree__row');
    row.style.setProperty('--i', String(Math.min(depth * 4 + state.entries.length % 7, 24)));

    if (item.dir) {
      row.classList.add('is-dir');
      row.textContent = '  '.repeat(depth) + '📁 ' + item.name;
      target.appendChild(row);
      renderTree(item.dir, depth + 1, target, conflicts, onFile);
    } else {
      row.classList.add('tree__row--file');
      if (conflicts.includes(item.entry.path)) row.classList.add('is-conflict');
      row.textContent = '  '.repeat(depth) + '📄 ' + item.name;
      row.addEventListener('click', () => {
        if (onFile) {
          document.querySelectorAll('.tree__row--active').forEach((node) => node.classList.remove('tree__row--active'));
          row.classList.add('tree__row--active');
          onFile(item.entry.path, item.entry.contents ?? '');
          return;
        }
        showEntry(row, item.entry);
      });
      target.appendChild(row);
    }
  }
}

function showEntry(row, entry) {
  document.querySelectorAll('.tree__row--active').forEach((node) => node.classList.remove('tree__row--active'));
  row.classList.add('tree__row--active');

  $('contentHead').textContent = entry.abs_path;
  $('content').textContent = entry.contents ?? t('content.empty');
}

let refreshTimer = null;

function refresh() {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(runPreview, 130);
}

function runPreview() {
  if (!state.selected) return;

  call('preview', {
    template: state.selected.id,
    name: $('pname').value.trim() || 'nowy-projekt',
    description: $('pdesc').value,
    author: $('pauthor').value,
    root: $('root').value.trim(),
    options: state.options,
    git: $('git').checked,
    force: $('force').checked,
  })
    .then((plan) => {
      state.entries = plan.entries;
      const tree = $('tree');
      tree.textContent = '';
      renderTree(buildTree(plan.entries), 0, tree, plan.conflicts);

      $('counts').textContent = t('counts.summary', {
        files: plan.file_count,
        dirs: plan.dir_count,
        size: humanSize(plan.total_bytes),
      });
      bump($('counts'));
      $('summary').textContent = plan.entries.length
        ? `${plan.project_name} → ${plan.root}`
        : t('summary.pick');
      $('problem').textContent = '';

      const problems = [...(plan.warnings || [])];
      if (plan.conflicts.length && !$('force').checked) {
        problems.push(t('problem.notEmpty', { n: plan.conflicts.length }));
      }
      $('problem').textContent = problems.join(' ');

      const derived = $('derived');
      derived.textContent = '';
      const chips = [
        ['slug', slugify($('pname').value || 'nowy-projekt')],
        ['snake', snake($('pname').value || 'nowy_projekt')],
        ['Pascal', pascal($('pname').value || 'NowyProjekt')],
      ];
      for (const [label, value] of chips) {
        const chip = el('span', 'chip', `${label}: ${value}`);
        derived.appendChild(chip);
      }

      $('contentHead').textContent = t('content.hint');
      $('content').textContent = '';

      renderSnapshotSource();
      updateTargetStatus();
    })
    .catch((error) => {
      $('tree').textContent = '';
      $('counts').textContent = '';
      $('summary').textContent = `${state.selected ? state.selected.name : ''} → ${$('root').value.trim()}`;
      $('problem').textContent = error.message;
    });
}

function updateTargetStatus() {
  call('check', { path: $('root').value.trim() })
    .then((status) => {
      const hint = $('rootHint');
      if (!status.exists) {
        hint.textContent = t('target.hint.willCreate');
      } else if (status.is_dir && status.is_empty) {
        hint.textContent = t('target.hint.empty');
      } else if (status.is_dir) {
        hint.textContent = t('target.hint.items', { n: status.entry_count });
      } else {
        hint.textContent = t('target.hint.isFile');
      }
      hint.style.color = status.exists && !status.is_dir ? 'var(--err)' : '';
    })
    .catch(() => {});
}

// --------------------------------------------------------- wybór folderu -

async function openPicker(startPath) {
  $('picker').hidden = false;
  syncModalFlag();
  await loadPicker(startPath || $('root').value.trim() || state.home);
}

function closePicker() {
  $('picker').hidden = true;
  state.pickerOnPick = null;
  syncModalFlag();
}

/** Wstrzymuje animacje tła, gdy otwarte jest jakiekolwiek okno modalne. */
function syncModalFlag() {
  const anyOpen = ['picker', 'settings', 'snapModal', 'filePicker'].some(
    (id) => $(id) && !$(id).hidden,
  );
  document.documentElement.classList.toggle('modal-open', anyOpen);
}

async function loadPicker(path) {
  try {
    const listing = await call('browse', { path });
    state.pickerPath = listing.path;

    $('currentPath').textContent = listing.path;

    const shortcuts = $('shortcuts');
    shortcuts.textContent = '';
    for (const shortcut of listing.shortcuts) {
      const button = el('button', 'picker__link', shortcut.label);
      button.type = 'button';
      button.title = shortcut.path;
      button.addEventListener('click', () => loadPicker(shortcut.path));
      shortcuts.appendChild(button);
    }

    const dirs = $('dirs');
    dirs.textContent = '';

    if (listing.parent) {
      const up = el('button', 'dir');
      up.appendChild(el('span', 'dir__icon', '↰'));
      up.appendChild(el('span', '', '.. (' + listing.parent + ')'));
      up.addEventListener('click', () => loadPicker(listing.parent));
      dirs.appendChild(up);
    }

    if (!listing.dirs.length) {
      dirs.appendChild(el('div', 'empty', t('picker.noDirs')));
    }

    for (const dir of listing.dirs) {
      const button = el('button', 'dir');
      button.style.setProperty('--i', String(listing.dirs.indexOf(dir)));
      button.appendChild(el('span', 'dir__icon', '📁'));
      button.appendChild(el('span', '', dir.name));
      if (dir.has_children) button.appendChild(el('span', 'dir__arrow', '›'));
      button.addEventListener('click', () => loadPicker(dir.path));
      button.addEventListener('dblclick', () => loadPicker(dir.path));
      dirs.appendChild(button);
    }
  } catch (error) {
    toast('Nie udało się otworzyć katalogu', error.message, 'err');
  }
}

// ------------------------------------------------------------ generowanie -

async function generate() {
  if (!state.selected) return;

  const button = $('create');
  const progress = $('progress');
  button.disabled = true;
  button.textContent = t('action.creating');
  progress.classList.add('progress--on');
  progress.style.width = '40%';

  try {
    const result = await call('generate', {
      template: state.selected.id,
      name: $('pname').value.trim() || slugify(state.selected.name),
      description: $('pdesc').value,
      author: $('pauthor').value,
      root: $('root').value.trim(),
      options: state.options,
      git: $('git').checked,
      force: $('force').checked,
    });

    progress.style.width = '100%';

    state.generated = result.root;
    const report = result.report;

    button.classList.add('is-done');
    button.textContent = t('action.done');

    const notes = [
      report.overwritten.length ? t('toast.overwritten', { n: report.overwritten.length }) : null,
      report.git_initialized ? t('toast.git') : null,
      result.snapshot ? t('toast.snapshot', { name: result.snapshot.name }) : null,
    ].filter(Boolean);

    toast(
      t('toast.done', { n: report.created_files.length }),
      `${result.root}${notes.length ? ` · ${notes.join(' · ')}` : ''}`,
      'ok',
      9000,
    );

    if (result.snapshot) refreshSnapshots();

    updateTargetStatus();
  } catch (error) {
    progress.style.width = '100%';
    toast(t('toast.createFail'), error.message, 'err', 9000);
  } finally {
    setTimeout(() => {
      button.disabled = false;
      button.classList.remove('is-done');
      button.textContent = t('action.create');
      progress.classList.remove('progress--on', 'progress--full');
      progress.style.width = '0%';
    }, 1400);
  }
}

// ------------------------------------------------------------ ustawienia -

/** Przesuwa kolor w strumieniu HSL (do wyliczania odcieni akcentu). */
function shiftHue(hex, dh, ds = 0, dl = 0) {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
    else if (max === g) h = ((b - r) / d + 2) / 6;
    else h = ((r - g) / d + 4) / 6;
  }

  h = (h + dh + 1) % 1;
  s = Math.min(1, Math.max(0, s + ds));
  const l2 = Math.min(1, Math.max(0, l + dl));

  const q = l2 < 0.5 ? l2 * (1 + s) : l2 + s - l2 * s;
  const p = 2 * l2 - q;
  const hue = (t) => {
    let tt = t;
    if (tt < 0) tt += 1;
    if (tt > 1) tt -= 1;
    if (tt < 1 / 6) return p + (q - p) * 6 * tt;
    if (tt < 1 / 2) return q;
    if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6;
    return p;
  };

  const to255 = (v) => Math.round(v * 255);
  return `#${[hue(h + 1 / 3), hue(h), hue(h - 1 / 3)].map((v) => to255(v).toString(16).padStart(2, '0')).join('')}`;
}

function currentAccent() {
  if (state.settings.custom) return state.settings.custom;
  return PALETTES[state.settings.palette].c1;
}

function applySettings() {
  const root = document.documentElement;
  const s = state.settings;

  const c1 = state.settings.custom ? shiftHue(state.settings.custom, 0, 0.06, 0.08) : PALETTES[s.palette].c1;
  const c2 = state.settings.custom ? state.settings.custom : PALETTES[s.palette].c2;
  const c3 = state.settings.custom ? shiftHue(state.settings.custom, 0.09, 0.02, -0.06) : PALETTES[s.palette].c3;

  root.style.setProperty('--c1', c1);
  root.style.setProperty('--c2', c2);
  root.style.setProperty('--c3', c3);
  root.style.setProperty('--accent', c2);
  root.style.setProperty('--aurora', String(s.aurora / 100));
  root.style.setProperty('--glass-blur', `${s.blur}px`);

  root.dataset.look = s.look;
  root.dataset.density = s.density;
  root.dataset.motion = s.motion ? 'on' : 'off';
}

function renderSettingsUI() {
  const s = state.settings;

  const swatches = $('swatches');
  swatches.textContent = '';
  PALETTES.forEach((palette, index) => {
    const button = el('button', 'swatch');
    button.type = 'button';
    button.title = palette.name;
    button.style.background = `linear-gradient(135deg, ${palette.c1}, ${palette.c2} 55%, ${palette.c3})`;
    if (!s.custom && s.palette === index) button.classList.add('on');
    button.addEventListener('click', () => {
      state.settings.palette = index;
      state.settings.custom = null;
      $('customColor').value = PALETTES[index].c1;
      commitSettings();
    });
    swatches.appendChild(button);
  });

  for (const button of $('lookSegment').children) {
    button.classList.toggle('on', button.dataset.value === s.look);
  }

  for (const button of $('densitySegment').children) {
    button.classList.toggle('on', button.dataset.value === s.density);
  }

  $('auroraRange').value = String(s.aurora);
  $('blurRange').value = String(s.blur);
  $('motionToggle').checked = s.motion;
  $('auroraValue').textContent = `${s.aurora}%`;
  $('blurValue').textContent = `${s.blur} px`;
  $('customColor').value = state.settings.custom || PALETTES[s.palette].c1;

  renderSnapshotSettings();
  renderUserTemplates();
  updateAccHints();
}

// --------------------------------------------------------- harmonijka ----

const ACC_KEY = 'gitools.settingsSection';
const ACC_SECTIONS = ['appearance', 'snapshots', 'templates'];

function openSection(name) {
  if (!ACC_SECTIONS.includes(name)) return;

  const box = $('settings');
  box.querySelectorAll('.acc__body').forEach((body) => {
    body.classList.toggle('is-open', body.dataset.accBody === name);
  });
  box.querySelectorAll('.acc__head').forEach((head) => {
    head.classList.toggle('is-on', head.dataset.acc === name);
  });

  try {
    localStorage.setItem(ACC_KEY, name);
  } catch {
    /* pamięć niedostępna - sekcja i tak się otworzyła */
  }

  const body = box.querySelector(`[data-acc-body="${name}"]`);
  if (body) body.querySelector('input, textarea, button')?.focus({ preventScroll: true });
}

function bindAccordion() {
  const box = $('settings');

  box.addEventListener('click', (event) => {
    const head = event.target.closest('.acc__head');
    if (!head) return;
    // kliknięta sekcja się zwija, otwarta pozostaje otwarta
    const isOpen = head.classList.contains('is-on');
    openSection(isOpen ? '__none__' : head.dataset.acc);
  });

  let saved = null;
  try {
    saved = localStorage.getItem(ACC_KEY);
  } catch {
    saved = null;
  }
  openSection(ACC_SECTIONS.includes(saved) ? saved : 'appearance');
}

/** Krótkie podsumowanie w nagłówku zwiniętej sekcji. */
function updateAccHints() {
  const lookKey = {
    glass: 'look.glass',
    dark: 'look.dark',
    light: 'look.light',
    'sketch-dark': 'look.sketch-dark',
    'sketch-light': 'look.sketch-light',
  }[state.settings.look];

  $('accHintAppearance').textContent =
    t(lookKey) + ' · ' + t(`density.${state.settings.density}`);

  const limit = state.snapSettings.keep || '∞';
  $('accHintSnapshots').textContent = state.snapSettings.auto
    ? t('acc.snapshots.autoOn', { n: limit })
    : t('acc.snapshots.autoOff', { n: limit });

  $('accHintTemplates').textContent = state.userTemplateIds.length
    ? t('acc.templates.hint', { n: state.userTemplateIds.length })
    : t('acc.templates.empty');
}

// -------------------------------------------------------------- język ----

/** Przełącza język i odświeża wszystkie teksty w interfejsie. */
function setLang(code) {
  if (!LANGS.some((l) => l.code === code)) return;

  state.settings.lang = code;
  lang = code;
  document.documentElement.lang = code;

  // tytuł okna też ma być przetłumaczony
  call('set_title', { title: `gitools — ${t('app.title')}` }).catch(() => {});

  applyI18n();
  renderLangSegment();
  renderSettingsUI();
  renderTemplates($('search').value);
  renderOptions();
  renderSnapshotSource();
  renderSnapshots();
  renderUserTemplates();
  updateAccHints();
  setTab(state.tab);
  refresh();
}

/** Buduje przełącznik języków (tylko przyciski, bez tłumaczenia własnych etykiet). */
function renderLangSegment() {
  const box = $('langSegment');
  box.textContent = '';

  for (const item of LANGS) {
    const button = el(
      'button',
      state.settings.lang === item.code ? 'on' : '',
      `${item.flag} ${item.label}`,
    );
    button.type = 'button';
    button.addEventListener('click', () => {
      if (state.settings.lang === item.code) return;
      commitSettings();
      setLang(item.code);
    });
    box.appendChild(button);
  }
}

function commitSettings() {
  saveSettings();
  applySettings();
  renderSettingsUI();
  renderTemplateList();
}

function bindSettings() {
  $('openSettings').addEventListener('click', () => {
    $('settings').hidden = false;
    syncModalFlag();
    renderSettingsUI();
  });

  $('settings').addEventListener('click', (event) => {
    if (event.target.dataset.close) {
      $('settings').hidden = true;
      syncModalFlag();
    }
  });

  $('lookSegment').addEventListener('click', (event) => {
    const value = event.target.dataset.value;
    if (!value) return;
    state.settings.look = value;
    commitSettings();
  });

  $('densitySegment').addEventListener('click', (event) => {
    const value = event.target.dataset.value;
    if (!value) return;
    state.settings.density = value;
    commitSettings();
  });

  $('auroraRange').addEventListener('input', (event) => {
    state.settings.aurora = Number(event.target.value);
    commitSettings();
  });

  $('blurRange').addEventListener('input', (event) => {
    state.settings.blur = Number(event.target.value);
    commitSettings();
  });

  $('motionToggle').addEventListener('change', (event) => {
    state.settings.motion = event.target.checked;
    commitSettings();
  });

  $('customColor').addEventListener('input', (event) => {
    state.settings.custom = event.target.value;
    commitSettings();
  });

  $('resetSettings').addEventListener('click', () => {
    state.settings = { ...DEFAULT_SETTINGS };
    commitSettings();
    toast(t('toast.reset'), t('toast.resetBody'), 'ok');
  });
}

// --------------------------------------------------------------- zakładki

const TABS = ['templates', 'snapshots', 'github'];

/** `templates` → `Templates` — nazwy elementów są budowane z nazwy zakładki. */
function cap(name) {
  return name.charAt(0).toUpperCase() + name.slice(1);
}

function setTab(tab) {
  if (!TABS.includes(tab)) return;
  state.tab = tab;

  for (const button of $('tabs').children) {
    button.classList.toggle('is-on', button.dataset.tab === tab);
  }

  // Każda zakładka ma swój panel w boczku, widok w środku i pasek na dole.
  // Trzymamy nazwy w jednym miejscu, żeby dodanie kolejnej zakładki
  // nie wymagało kopiowania warunków.
  for (const name of TABS) {
    const on = name === tab;

    for (const prefix of ['panel', 'view', 'bar']) {
      const node = $(prefix + cap(name));
      if (!node) continue;
      node.hidden = !on;
      node.classList.toggle('is-on', on);
    }
  }

  $('gearHint').innerHTML =
    tab === 'templates'
      ? '<kbd>Ctrl</kbd>+<kbd>Enter</kbd> utwórz'
      : tab === 'snapshots'
        ? '<kbd>Ctrl</kbd>+<kbd>N</kbd> migawka'
        : t('gh.gearHint');

  document.querySelectorAll('.main .card').forEach((card) => trackSpotlight(card));

  if (tab === 'snapshots') refreshSnapshots();
  if (tab === 'github') refreshGithub();
}

// ---------------------------------------------------------------- GitHub ---

/** Odświeża stan logowania i listę repozytoriów. */
async function refreshGithub() {
  try {
    const status = await call('github_status');
    state.ghUser = status.logged_in ? status.user : null;

    renderGithubUser();
    renderGithubSummary();

    if (state.ghUser) await refreshGhRepos();
    else renderGhRepos([]);
  } catch (error) {
    toast(t('toast.ghStatusFail'), error.message, 'err');
  }
}

/** Przełącza formularz logowania i kartę zalogowanego użytkownika. */
function renderGithubUser() {
  const signedIn = Boolean(state.ghUser);

  $('ghSignedOut').hidden = signedIn;
  $('ghSignedIn').hidden = !signedIn;
  $('ghBadge').hidden = !signedIn;
  $('ghSideHint').textContent = signedIn
    ? t('gh.sideSignedIn', { login: state.ghUser.login })
    : t('gh.sideSignedOut');

  if (!signedIn) {
    // nie zostawiamy tokenu w polu po wylogowaniu
    $('ghToken').value = '';
    $('ghToken').type = 'password';
    $('ghShowToken').checked = false;
    $('ghOpenRepo').hidden = true;
    return;
  }

  const { login, name, avatar_url, html_url } = state.ghUser;

  $('ghLoginName').textContent = name ? `${name} (${login})` : login;
  $('ghLoginLink').textContent = html_url || `https://github.com/${login}`;
  $('ghAvatar').src = avatar_url || '';
  $('ghAvatar').hidden = !avatar_url;

  // podpowiedź: nazwa repo prosto z nazwy projektu
  if (!$('ghRepoName').value.trim()) $('ghRepoName').placeholder = slugify($('pname').value.trim()) || 'moj-projekt';
}

/** Wczytuje repozytoria i wypełnia listę wyboru. */
async function refreshGhRepos() {
  try {
    const data = await call('github_repos');
    state.ghRepos = data.repos || [];
    renderGhRepos(state.ghRepos);
  } catch (error) {
    renderGhRepos([]);
    toast(t('toast.ghReposFail'), error.message, 'err');
  }
}

function renderGhRepos(repos) {
  const select = $('ghRepo');
  const previous = select.value;
  select.innerHTML = '';

  if (!repos.length) {
    const option = document.createElement('option');
    option.value = '';
    option.textContent = t('gh.noRepos');
    select.append(option);
    $('ghRepoHint').textContent = '';
    renderGithubSummary();
    return;
  }

  for (const repo of repos) {
    const option = document.createElement('option');
    option.value = repo.full_name;
    option.textContent = repo.private ? `🔒 ${repo.full_name}` : repo.full_name;
    select.append(option);
  }

  if (repos.some((r) => r.full_name === previous)) select.value = previous;
  $('ghRepoHint').textContent = t('gh.repoCount', { n: repos.length });
  renderGithubSummary();
}

function renderGithubSummary() {
  const signedIn = Boolean(state.ghUser);

  $('ghSummary').textContent = signedIn
    ? t('gh.signedInAs', { login: state.ghUser.login })
    : t('gh.signedOut');
  $('ghProblem').textContent = '';

  // przyciski wymagające konta gasną, żeby nie kusły na kliknięcie
  for (const id of ['ghCreate', 'ghPush', 'ghPushBottom']) {
    $(id).disabled = !signedIn;
  }
  $('ghLogoutBottom').hidden = !signedIn;
}

/** Logowanie tokenem. */
async function ghLogin() {
  const token = $('ghToken').value.trim();
  if (!token) {
    toast(t('gh.loginTitle'), t('gh.tokenEmpty'), 'err');
    return;
  }

  $('ghLogin').disabled = true;

  try {
    const data = await call('github_login', { token });
    state.ghUser = data.user;
    renderGithubUser();
    renderGithubSummary();
    await refreshGhRepos();
    toast(t('gh.loggedIn', { login: data.user.login }), t('gh.loggedInBody'), 'ok');
  } catch (error) {
    toast(t('gh.loginFailed'), error.message, 'err');
  } finally {
    $('ghLogin').disabled = false;
  }
}

/** Wylogowanie: kasuje token z dysku i czyści stan. */
async function ghLogout() {
  try {
    await call('github_logout');
  } catch (error) {
    toast(t('gh.logoutFailed'), error.message, 'err');
    return;
  }

  state.ghUser = null;
  state.ghRepos = [];
  renderGithubUser();
  renderGhRepos([]);
  renderGithubSummary();
  toast(t('gh.loggedOut'), t('gh.loggedOutBody'), 'ok');
}

/** Zakłada nowe repozytorium i od razu wybiera je na liście. */
async function ghCreateRepo() {
  const name = $('ghRepoName').value.trim();
  if (!name) {
    toast(t('gh.createTitle'), t('gh.nameEmpty'), 'err');
    return;
  }

  $('ghCreate').disabled = true;

  try {
    const data = await call('github_create_repo', {
      name,
      description: $('ghRepoDesc').value.trim(),
      private: $('ghRepoPrivate').value === '1',
    });

    await refreshGhRepos();
    $('ghRepo').value = data.repo.full_name;
    $('ghBranch').value = data.repo.default_branch || 'main';
    $('ghOpenRepo').href = data.repo.html_url;
    $('ghOpenRepo').hidden = false;
    renderGithubSummary();
    toast(t('gh.created', { name: data.repo.full_name }), t('gh.createdBody'), 'ok');
  } catch (error) {
    toast(t('gh.createFailed'), error.message, 'err');
  } finally {
    $('ghCreate').disabled = false;
  }
}

/** Wysyła katalog projektu jednym commitem. */
async function ghPush() {
  const repo = $('ghRepo').value;
  const dir = $('ghDir').value.trim();

  if (!repo) {
    toast(t('gh.pushTitle'), t('gh.pickRepo'), 'err');
    return;
  }
  if (!dir) {
    toast(t('gh.pushTitle'), t('gh.pickDir'), 'err');
    return;
  }

  $('ghPush').disabled = true;
  $('ghPushBottom').disabled = true;
  $('ghResult').innerHTML = '';

  try {
    const report = await call('github_push', {
      repo,
      branch: $('ghBranch').value.trim() || 'main',
      dir,
      message: $('ghMessage').value.trim(),
      prefix: $('ghRepoName').value.trim() ? slugify($('ghRepoName').value.trim()) : '',
    });

    const link = document.createElement('a');
    link.href = report.url;
    link.target = '_blank';
    link.rel = 'noreferrer';
    link.textContent = `${report.commit.slice(0, 7)}`;

    $('ghResult').innerHTML = '';
    const line = document.createElement('div');
    line.className = 'ghResult__line';
    line.append(`${t('gh.sent', { n: report.files, repo: report.repo, branch: report.branch })} `, link);
    $('ghResult').append(line);

    if (report.skipped?.length) {
      const warn = document.createElement('div');
      warn.className = 'ghResult__warn';
      warn.textContent = t('gh.skipped', { n: report.skipped.length });
      $('ghResult').append(warn);
    }

    $('ghMessage').value = '';
    $('ghOpenRepo').href = `https://github.com/${report.repo}`;
    $('ghOpenRepo').hidden = false;
    toast(t('gh.pushed'), t('gh.pushedBody'), 'ok');
  } catch (error) {
    toast(t('gh.pushFailed'), error.message, 'err');
  } finally {
    $('ghPush').disabled = false;
    $('ghPushBottom').disabled = false;
    renderGithubSummary();
  }
}

/** Podpina przyciski zakładki GitHub. */
function bindGithub() {
  $('ghLogin').addEventListener('click', ghLogin);
  $('ghLogout').addEventListener('click', ghLogout);
  $('ghLogoutBottom').addEventListener('click', ghLogout);
  $('ghCreate').addEventListener('click', ghCreateRepo);
  $('ghPush').addEventListener('click', ghPush);
  $('ghPushBottom').addEventListener('click', ghPush);
  $('ghRefreshSide').addEventListener('click', refreshGithub);

  $('ghShowToken').addEventListener('change', () => {
    $('ghToken').type = $('ghShowToken').checked ? 'text' : 'password';
  });

  $('ghToken').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') ghLogin();
  });

  $('ghBrowse').addEventListener('click', () => {
    state.pickerOnPick = (path) => {
      $('ghDir').value = path;
    };
    openPicker($('ghDir').value.trim());
  });

  $('ghTokenPage').addEventListener('click', () => {
    call('open_url', { url: 'https://github.com/settings/tokens' })
      .catch((error) => toast(t('toast.openFail'), error.message, 'err'));
  });

  $('ghRepo').addEventListener('change', () => {
    const repo = state.ghRepos.find((r) => r.full_name === $('ghRepo').value);
    if (repo) {
      $('ghBranch').value = repo.default_branch || 'main';
      $('ghOpenRepo').href = repo.html_url;
      $('ghOpenRepo').hidden = !repo.html_url;
    }
  });
}

// --------------------------------------------------------------- migawki --

function snapshotSourceLabel() {
  const template = state.selected;
  const root = $('root').value.trim();
  if (!template) return t('snap.sourceNone');
  return t('snap.source', { template: template.name, path: root ? ` → ${root}` : '' });
}

async function refreshSnapshots() {
  try {
    const data = await call('snapshots_list');
    state.snapshots = data.snapshots;
    state.snapSettings = data.settings;
    state.snapDir = data.dir;

    renderSnapshots();
    renderSnapshotSettings();
  } catch (error) {
    toast(t('toast.snapListFail'), error.message, 'err');
  }
}

function filteredSnapshots() {
  const needle = ($('snapSearch')?.value || '').trim().toLowerCase();
  if (!needle) return state.snapshots;
  return state.snapshots.filter((snap) =>
    [snap.name, snap.project_name, snap.template_name, snap.source]
      .join(' ')
      .toLowerCase()
      .includes(needle),
  );
}

function renderSnapshots() {
  const list = filteredSnapshots();
  const total = state.snapshots.reduce((sum, snap) => sum + snap.total_bytes, 0);

  $('snapCounts').textContent = state.snapshots.length
    ? `${state.snapshots.length} migawek · ${humanSize(total)}`
    : '';
  $('snapBadge').hidden = !state.snapshots.length;
  $('snapBadge').textContent = String(state.snapshots.length);

  $('snapSummary').textContent = state.snapshots.length
    ? t('snap.summary', { name: state.snapshots[0].name, date: state.snapshots[0].created_at })
    : t('snap.summaryNone');
  $('snapProblem').textContent = '';

  $('snapSource').textContent = snapshotSourceLabel();

  // panel boczny
  const side = $('snapList');
  side.textContent = '';
  for (const [index, snap] of state.snapshots.slice(0, 40).entries()) {
    const button = el('button', 'snapItem');
    button.type = 'button';
    button.style.setProperty('--i', String(index));
    button.appendChild(el('div', 'snapItem__name', snap.name));
    button.appendChild(
      el('div', 'snapItem__meta', t('snap.itemMeta', { n: snap.file_count, size: humanSize(snap.total_bytes) })),
    );
    button.addEventListener('click', () => openSnapshot(snap.id));
    side.appendChild(button);
  }
  if (!state.snapshots.length) side.appendChild(el('div', 'empty', t('snap.emptyList')));

  // siatka
  const grid = $('snapGrid');
  grid.textContent = '';

  if (!state.snapshots.length) {
    const empty = el('div', 'snapEmpty');
    empty.appendChild(el('div', 'snapEmpty__icon', '📸'));
    empty.appendChild(el('div', 'snapEmpty__title', t('snap.emptyTitle')));
    empty.appendChild(el('div', 'snapEmpty__body', t('snap.emptyBody')));
    grid.appendChild(empty);
    return;
  }

  if (!list.length) {
    grid.appendChild(el('div', 'empty', t('snap.emptySearch')));
    return;
  }

  for (const [index, snap] of list.entries()) {
    const card = el('article', 'snapCard');
    card.style.setProperty('--i', String(index));

    const head = el('div', 'snapCard__head');
    head.appendChild(el('div', 'snapCard__icon', snap.template_id ? '📸' : '📁'));
    const titles = el('div', 'snapCard__titles');
    titles.appendChild(el('div', 'snapCard__name', snap.name));
    titles.appendChild(el('div', 'snapCard__meta', snap.created_at));
    head.appendChild(titles);
    card.appendChild(head);

    const badges = el('div', 'snapCard__badges');
    badges.appendChild(el('span', 'chip', snap.template_name));
    badges.appendChild(el('span', 'chip', t('snap.chip.files', { n: snap.file_count })));
    badges.appendChild(el('span', 'chip', humanSize(snap.total_bytes)));
    card.appendChild(badges);

    card.appendChild(el('div', 'snapCard__source', shortPath(snap.source)));
    card.title = snap.source;

    const actions = el('div', 'snapCard__actions');
    const open = el('button', 'btn btn--small', t('snap.open'));
    open.type = 'button';
    open.addEventListener('click', () => openSnapshot(snap.id));
    actions.appendChild(open);

    const restore = el('button', 'btn btn--small btn--ghost', t('snap.restore'));
    restore.type = 'button';
    restore.addEventListener('click', () => restoreSnapshot(snap));
    actions.appendChild(restore);

    const reveal = el('button', 'btn btn--small btn--ghost', t('snap.sourceBtn'));
    reveal.type = 'button';
    reveal.addEventListener('click', () =>
      call('reveal', { path: snap.source }).catch((error) => toast('Nie udało się otworzyć', error.message, 'err')),
    );
    actions.appendChild(reveal);

    const remove = el('button', 'btn btn--small btn--danger', t('snap.delete'));
    remove.type = 'button';
    remove.addEventListener('click', () => deleteSnapshot(snap));
    actions.appendChild(remove);

    card.appendChild(actions);
    grid.appendChild(card);
  }
}

function renderSnapshotSettings() {
  const s = state.snapSettings;
  $('snapAuto').checked = s.auto;
  $('snapKeep').value = String(s.keep);
  $('snapKeepValue').textContent = s.keep ? String(s.keep) : '∞';
  $('snapDirHint').textContent = state.snapDir;
  $('setSnapAuto').checked = s.auto;
  $('setSnapKeep').value = String(s.keep);
  $('setSnapKeepValue').textContent = s.keep ? String(s.keep) : '∞';
  $('setSnapDir').textContent = state.snapDir;
}

async function saveSnapshotSettings(auto, keep) {
  try {
    const data = await call('snapshot_settings', { auto, keep });
    state.snapSettings = data.settings;
    if (data.pruned?.length) {
      toast(t('toast.snapPruned'), data.pruned.join(', '), 'ok');
    }
    renderSnapshotSettings();
    updateAccHints();
    refreshSnapshots();
  } catch (error) {
    toast(t('toast.snapSettingsFail'), error.message, 'err');
  }
}

async function makeSnapshot(kind) {
  const name = ($('snapName').value || '').trim();

  try {
    if (kind === 'dir') {
      const root = $('root').value.trim();
      if (!root) throw new Error(t('err.needRoot'));
      await call('snapshot_create_dir', { path: root, name });
    } else {
      if (!state.selected) throw new Error(t('err.needTemplate'));
      await call('snapshot_create_plan', {
        template: state.selected.id,
        name: $('pname').value.trim() || slugify(state.selected.name),
        description: $('pdesc').value,
        author: $('pauthor').value,
        root: $('root').value.trim(),
        options: state.options,
        git: $('git').checked,
        force: $('force').checked,
        snapshotName: name,
      });
    }

    $('snapName').value = '';
    toast(t('toast.snapSaved'), t('toast.snapSavedBody'), 'ok');
    refreshSnapshots();
  } catch (error) {
    toast(t('toast.snapFail'), error.message, 'err');
  }
}

async function openSnapshot(id) {
  try {
    const snap = await call('snapshot_load', { id });
    state.openSnapshot = snap;

    $('snapModalTitle').textContent = snap.name;

    const meta = $('snapMeta');
    meta.textContent = '';
    for (const chip of [
      snap.created_at,
      snap.template_name,
      snap.project_name,
      `${snap.file_count} plików`,
      humanSize(snap.total_bytes),
    ]) {
      if (chip) meta.appendChild(el('span', 'chip', chip));
    }

    const tree = $('snapTree');
    tree.textContent = '';
    renderTree(
      buildTree(snap.files.map((file) => ({ path: file.path, kind: 'file', contents: file.contents }))),
      0,
      tree,
      [],
      (path, contents) => {
        $('snapContentHead').textContent = path;
        $('snapContent').textContent = contents;
      },
    );

    $('snapContentHead').textContent = t('content.hint');
    $('snapContent').textContent = '';
    $('snapRestorePath').value = $('root').value.trim() || state.home;

    $('snapModal').hidden = false;
    syncModalFlag();
  } catch (error) {
    toast(t('toast.snapOpenFail'), error.message, 'err');
  }
}

function closeSnapshot() {
  $('snapModal').hidden = true;
  state.openSnapshot = null;
  syncModalFlag();
}

async function restoreSnapshot(snap) {
  const target = ($('snapRestorePath').value || $('root').value || state.home).trim();
  if (!target) {
    toast(t('toast.noTarget'), t('toast.noTargetBody'), 'err');
    return;
  }

  try {
    const data = await call('snapshot_restore', { id: snap.id, target });
    toast(
      t('toast.snapRestored', { n: data.restored }),
      data.target,
      'ok',
      8000,
    );
  } catch (error) {
    toast(t('toast.snapRestoreFail'), error.message, 'err');
  }
}

async function deleteSnapshot(snap) {
  if (!window.confirm(t('confirm.deleteSnap', { name: snap.name }))) return;

  try {
    await call('snapshot_delete', { id: snap.id });
    if (state.openSnapshot && state.openSnapshot.id === snap.id) closeSnapshot();
    toast(t('toast.snapDeleted'), snap.name, 'ok');
    refreshSnapshots();
  } catch (error) {
    toast(t('toast.snapDeleteFail'), error.message, 'err');
  }
}

async function clearSnapshots() {
  if (!state.snapshots.length) {
    toast(t('toast.snapNothingToClear'), '', 'ok', 2500);
    return;
  }
  if (!window.confirm(t('confirm.clearSnaps', { n: state.snapshots.length }))) {
    return;
  }

  try {
    const data = await call('snapshot_clear');
    toast(t('toast.snapCleared', { n: data.removed }), '', 'ok');
    refreshSnapshots();
  } catch (error) {
    toast(t('toast.snapClearFail'), error.message, 'err');
  }
}

// -------------------------------------------------- import własnych szablonów

function renderUserTemplates() {
  const box = $('tplList');
  box.textContent = '';

  const mine = state.templates.filter((t) => state.userTemplateIds.includes(t.id));
  if (!mine.length) {
    box.appendChild(el('div', 'empty', t('tpl.empty')));
    return;
  }

  for (const [index, template] of mine.entries()) {
    const row = el('div', 'tplRow');
    row.style.setProperty('--i', String(index));

    row.appendChild(el('span', 'tplRow__icon', template.icon));
    const body = el('div', 'tplRow__body');
    body.appendChild(el('div', 'tplRow__name', template.name));
    body.appendChild(
      el('div', 'tplRow__meta', `${template.id} · ${template.tagline || t('tpl.ownTemplate')}`),
    );
    row.appendChild(body);

    const remove = el('button', 'btn btn--small btn--danger', t('snap.delete'));
    remove.type = 'button';
    remove.addEventListener('click', () => deleteUserTemplate(template));
    row.appendChild(remove);

    box.appendChild(row);
  }
}

async function deleteUserTemplate(template) {
  if (!window.confirm(t('confirm.deleteTpl', { name: template.name }))) return;
  try {
    await call('user_template_delete', { id: template.id });
    toast(t('toast.tplDeleted'), template.name, 'ok');
    await reloadUserTemplates();
  } catch (error) {
    toast(t('toast.tplDeleteFail'), error.message, 'err');
  }
}

async function reloadUserTemplates() {
  const data = await call('bootstrap');
  state.templates = data.templates;
  state.userTemplateIds = data.userTemplate_ids;
  renderTemplates($('search').value);
  renderUserTemplates();
  updateAccHints();
}

async function addTemplateFromJson(raw) {
  try {
    const data = await call('user_template_add', { json: raw });
    $('tplJson').value = '';
    toast(t('toast.tplAdded'), `${data.template.name} → ${data.path}`, 'ok', 7000);
    await reloadUserTemplates();

    if (data.template) {
      selectTemplate(data.template.id);
    }
  } catch (error) {
    toast(t('toast.tplFail'), error.message, 'err', 9000);
  }
}

async function addTemplateFromFile(path) {
  try {
    const data = await call('user_template_add_file', { path });
    toast(t('toast.tplAdded'), `${data.template.name} → ${data.path}`, 'ok', 7000);
    await reloadUserTemplates();
    selectTemplate(data.template.id);
  } catch (error) {
    toast(t('toast.tplFileFail'), error.message, 'err', 9000);
  }
}

// --------------------------------------------------- wybór pliku (JSON) ----

async function loadFilePicker(path) {
  try {
    const listing = await call('browse_files', { path });
    state.filePickerPath = listing.path;

    $('fileCurrentPath').textContent = listing.path;

    const shortcuts = $('fileShortcuts');
    shortcuts.textContent = '';
    for (const shortcut of listing.shortcuts) {
      const button = el('button', 'picker__link', shortcut.label);
      button.type = 'button';
      button.addEventListener('click', () => loadFilePicker(shortcut.path));
      shortcuts.appendChild(button);
    }

    const dirs = $('fileDirs');
    dirs.textContent = '';

    if (listing.parent) {
      const up = el('button', 'dir');
      up.appendChild(el('span', 'dir__icon', '↰'));
      up.appendChild(el('span', '', '.. (' + listing.parent + ')'));
      up.addEventListener('click', () => loadFilePicker(listing.parent));
      dirs.appendChild(up);
    }

    for (const dir of listing.dirs) {
      const button = el('button', 'dir');
      button.appendChild(el('span', 'dir__icon', '📁'));
      button.appendChild(el('span', '', dir.name));
      if (dir.has_children) button.appendChild(el('span', 'dir__arrow', '›'));
      button.addEventListener('click', () => loadFilePicker(dir.path));
      button.addEventListener('dblclick', () => loadFilePicker(dir.path));
      dirs.appendChild(button);
    }

    const items = $('fileItems');
    items.textContent = '';
    if (!listing.files.length) {
      items.appendChild(el('div', 'empty', t('picker.noFiles')));
    }

    for (const file of listing.files) {
      const button = el('button', 'dir');
      button.appendChild(el('span', 'dir__icon', '📄'));
      button.appendChild(el('span', '', file.name));
      button.addEventListener('click', () => {
        $('filePicker').hidden = true;
        addTemplateFromFile(file.path);
      });
      items.appendChild(button);
    }
  } catch (error) {
    toast('Nie udało się otworzyć katalogu', error.message, 'err');
  }
}

// --------------------------------------------------------------- start --

async function start() {
  try {
    const data = await call('bootstrap');
    state.templates = data.templates;
    state.userTemplateIds = data.userTemplate_ids || [];
    state.home = data.home;
    state.snapDir = data.snapshotDir || '';

    // Język ustawiony wcześniej musi zadziałać przed pierwszym renderem.
    lang = state.settings.lang;
    document.documentElement.lang = lang;
    applyI18n();
    renderLangSegment();

    $('version').textContent = 'v' + data.version + ' · ' + data.platform;
    $('root').value = data.home;

    renderTemplates();
    renderUserTemplates();
    selectTemplate('rust');
    setTab('templates');
    refresh();

    // Migawki wczytujemy w tle, żeby aplikacja startowała szybko.
    refreshSnapshots();
  } catch (error) {
    fatal(t('fatal.bootstrap', { msg: error.message }));
  }
}

function bind() {
  $('search').addEventListener('input', (event) => renderTemplates(event.target.value));

  $('tabs').addEventListener('click', (event) => {
    const tab = event.target.closest('.tab');
    if (tab) setTab(tab.dataset.tab);
  });

  $('pname').addEventListener('input', (event) => {
    event.target.dataset.touched = 'true';
    if (event.target.value.trim()) suggestRoot();
    refresh();
  });
  $('pdesc').addEventListener('input', refresh);
  $('pauthor').addEventListener('input', refresh);
  $('root').addEventListener('input', refresh);
  $('git').addEventListener('change', refresh);
  $('force').addEventListener('change', refresh);

  $('browse').addEventListener('click', () => openPicker());
  $('up').addEventListener('click', async () => {
    const listing = await call('browse', { path: state.pickerPath });
    if (listing.parent) loadPicker(listing.parent);
  });

  $('createDir').addEventListener('click', async () => {
    const name = $('newDirName').value.trim();
    if (!name) return;
    try {
      const listing = await call('mkdir', { parent: state.pickerPath, name });
      $('newDirName').value = '';
      loadPicker(listing.path);
    } catch (error) {
      toast('Nie udało się utworzyć katalogu', error.message, 'err');
    }
  });

  $('newDirName').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') $('createDir').click();
  });

  $('pickCurrent').addEventListener('click', () => {
    const path = state.pickerPath;
    const onPick = state.pickerOnPick;
    closePicker();

    // Picker służy do kilku pól (katalog docelowy, katalog do wysyłki),
    // więc odbiorca jest ustawiany przy otwarciu okna.
    if (onPick) onPick(path);
    else {
      $('root').value = path;
      refresh();
    }
  });

  $('picker').addEventListener('click', (event) => {
    if (event.target.dataset.close) closePicker();
  });

  $('create').addEventListener('click', generate);

  $('openFolder').addEventListener('click', () => {
    const target = state.generated || $('root').value.trim();
    if (!target) return;
    call('open', { path: target }).catch((error) => toast(t('toast.openFail'), error.message, 'err'));
  });

  bindSnapshots();
  bindUserTemplates();
  bindGithub();

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      if (state.tab === 'templates') generate();
      else if (state.tab === 'github') ghPush();
      else makeSnapshot('plan');
    }
    if (event.key === 'n' && (event.ctrlKey || event.metaKey) && state.tab === 'snapshots') {
      event.preventDefault();
      makeSnapshot('plan');
    }
    if (event.key === 'Escape') {
      if (!$('snapModal').hidden) closeSnapshot();
      else if (!$('filePicker').hidden) {
        $('filePicker').hidden = true;
        syncModalFlag();
      } else if (!$('picker').hidden) closePicker();
    }
  });
}

// ------------------------------------------------------------- migawki ----

function bindSnapshots() {
  $('snapFromPlan').addEventListener('click', () => {
    setTab('snapshots');
    makeSnapshot('plan');
  });
  $('snapFromDir').addEventListener('click', () => {
    setTab('snapshots');
    makeSnapshot('dir');
  });
  $('snapNewPlan').addEventListener('click', () => makeSnapshot('plan'));
  $('snapNewDir').addEventListener('click', () => makeSnapshot('dir'));
  $('snapMakePlan').addEventListener('click', () => makeSnapshot('plan'));
  $('snapRefresh').addEventListener('click', refreshSnapshots);
  $('snapSearch').addEventListener('input', renderSnapshots);
  $('snapClear').addEventListener('click', clearSnapshots);
  $('setSnapClear').addEventListener('click', clearSnapshots);

  $('snapAuto').addEventListener('change', (event) => {
    saveSnapshotSettings(event.target.checked, state.snapSettings.keep);
  });
  $('setSnapAuto').addEventListener('change', (event) => {
    saveSnapshotSettings(event.target.checked, state.snapSettings.keep);
  });

  for (const id of ['snapKeep', 'setSnapKeep']) {
    $(id).addEventListener('input', (event) => {
      const keep = Number(event.target.value);
      $('snapKeepValue').textContent = keep ? String(keep) : '∞';
      $('setSnapKeepValue').textContent = keep ? String(keep) : '∞';
    });
    $(id).addEventListener('change', (event) => {
      saveSnapshotSettings(state.snapSettings.auto, Number(event.target.value));
    });
  }

  const openDir = () => {
    if (!state.snapDir) return;
    call('open', { path: state.snapDir }).catch((error) => toast('Nie udało się otworzyć', error.message, 'err'));
  };
  $('snapOpenDir').addEventListener('click', openDir);
  $('setSnapOpen').addEventListener('click', openDir);

  $('snapRestoreLast').addEventListener('click', () => {
    const snap = state.snapshots[0];
    if (!snap) {
      toast(t('toast.snapNone'), t('toast.snapNoneBody'), 'err');
      return;
    }
    restoreSnapshot(snap);
  });

  $('snapRestore').addEventListener('click', () => {
    if (!state.openSnapshot) return;
    restoreSnapshot({ id: state.openSnapshot.id, name: state.openSnapshot.name });
  });

  $('snapModal').addEventListener('click', (event) => {
    if (event.target.dataset.close) closeSnapshot();
  });
}

// ------------------------------------------------- własne szablony (JSON) --

function bindUserTemplates() {
  $('tplAdd').addEventListener('click', () => {
    const raw = $('tplJson').value.trim();
    if (!raw) {
      toast(t('toast.tplEmpty'), t('toast.tplEmptyBody'), 'err');
      return;
    }
    addTemplateFromJson(raw);
  });

  $('tplExample').addEventListener('click', async () => {
    try {
      const data = await call('user_template_example');
      $('tplJson').value = data.json;
      $('tplJson').focus();
    } catch (error) {
      toast(t('toast.tplExampleFail'), error.message, 'err');
    }
  });

  $('tplClear').addEventListener('click', () => {
    $('tplJson').value = '';
  });

  $('tplFromFile').addEventListener('click', () => {
    $('filePicker').hidden = false;
    syncModalFlag();
    loadFilePicker(state.home);
  });

  $('fileUp').addEventListener('click', async () => {
    if (!state.filePickerPath) return;
    try {
      const listing = await call('browse_files', { path: state.filePickerPath });
      if (listing.parent) loadFilePicker(listing.parent);
    } catch {
      /* zostajemy w bieżącym katalogu */
    }
  });

  $('filePicker').addEventListener('click', (event) => {
    if (event.target.dataset.close) {
      $('filePicker').hidden = true;
      syncModalFlag();
    }
  });

  $('filePickCurrent').addEventListener('click', () => {
    $('filePicker').hidden = true;
    $('tplJson').focus();
  });
}

bind();
for (const card of document.querySelectorAll('.card')) trackSpotlight(card);
bindSettings();
bindAccordion();
applySettings();
start();