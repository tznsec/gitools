/* Animacje strony gitools.
 *
 * Wszystko startuje dopiero po `DOMContentLoaded`, a `prefers-reduced-motion`
 * wyłącza warstwę ruchu — użytkownik widzi wtedy statyczną, w pełni czytelną
 * stronę. Żadna animacja nie blokuje treści: `reveal` ustawiamy tylko wtedy,
 * gdy przeglądarka umie je animować.
 */

(() => {
  'use strict';

  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const fine = window.matchMedia('(hover: hover) and (pointer: fine)').matches;
  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

  // Zdejmujemy znak `no-js` pierwszą rzeczą — od tego momentu CSS może
  // ukrywać treść oczekując na animację wejścia.
  document.documentElement.classList.remove('no-js');

  // ------------------------------------------------------------- splash ----

  function splash() {
    const node = $('#splash');
    if (!node) return;

    // Litery nazwy po kolei — opóźnienie rośnie z indeksem.
    const name = $('.splash__name');
    if (name) {
      [...name.textContent].forEach((char, i) => {
        const span = document.createElement('span');
        span.textContent = char;
        span.style.animationDelay = `${420 + i * 62}ms`;
        name.replaceChildren(span);
      });
    }

    let done = false;
    const finish = () => {
      if (done) return;
      done = true;
      node.classList.add('is-done');
      setTimeout(() => node.remove(), 700);
      document.body.style.overflow = '';
    };

    document.body.style.overflow = 'hidden';

    //Nigdy nie blokujemy strony: 2,6 s to górna granica.
    const min = reduced ? 250 : 2600;
    const started = performance.now();
    const ready = () =>
      setTimeout(finish, Math.max(0, min - (performance.now() - started)));

    if (document.readyState === 'complete') ready();
    else window.addEventListener('load', ready, { once: true });

    // Utrata połączenia ze zdjęciami nie może zostawić splashu na ekranie.
    setTimeout(finish, min + 1200);
  }

  // ------------------------------------------------------------- reveal ----

  function reveal() {
    const items = $$('.reveal');

    if (reduced || !('IntersectionObserver' in window)) {
      items.forEach((n) => n.classList.add('is-in'));
      return;
    }

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add('is-in');
          io.unobserve(entry.target);
        }
      },
      { rootMargin: '0px 0px -12% 0px', threshold: 0.08 },
    );

    items.forEach((node, i) => {
      // Kaskada: rodzeństwo w karcie pojawia się po sobie.
      if (!node.style.getPropertyValue('--d')) {
        const group = node.parentElement?.children?.length ?? 1;
        node.style.setProperty('--d', `${Math.min(i % Math.max(group, 1), 8) * 55}ms`);
      }
      io.observe(node);
    });
  }

  // ------------------------------------------------------------ liczby ----

  function counters() {
    const nodes = $$('[data-count]');
    if (!nodes.length) return;

    if (reduced) {
      nodes.forEach((n) => (n.textContent = n.dataset.count));
      return;
    }

    const run = (node) => {
      const target = Number(node.dataset.count);
      const suffix = node.dataset.suffix || '';
      if (!Number.isFinite(target)) return;

      const duration = 900;
      const start = performance.now();

      const step = (now) => {
        const t = Math.min(1, (now - start) / duration);
        const eased = 1 - Math.pow(1 - t, 3);
        node.textContent = Math.round(target * eased) + suffix;
        if (t < 1) requestAnimationFrame(step);
      };

      requestAnimationFrame(step);
    };

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          run(entry.target);
          io.unobserve(entry.target);
        }
      },
      { threshold: 0.5 },
    );

    nodes.forEach((n) => io.observe(n));
  }

  // ------------------------------------------------------- poświata kart ----

  function spotlight() {
    if (reduced || !fine) return;

    for (const card of $$('.card, .tpl')) {
      card.addEventListener(
        'pointermove',
        (event) => {
          const box = card.getBoundingClientRect();
          card.style.setProperty('--mx', `${event.clientX - box.left}px`);
          card.style.setProperty('--my', `${event.clientY - box.top}px`);
        },
        { passive: true },
      );
    }
  }

  // --------------------------------------------------------------- tilt ----

  function tilt() {
    if (reduced || !fine) return;

    for (const card of $$('.tilt')) {
      card.addEventListener(
        'pointermove',
        (event) => {
          const box = card.getBoundingClientRect();
          const x = (event.clientX - box.left) / box.width - 0.5;
          const y = (event.clientY - box.top) / box.height - 0.5;
          card.style.transform =
            `perspective(900px) rotateX(${(-y * 5).toFixed(2)}deg) ` +
            `rotateY(${(x * 6).toFixed(2)}deg) translateY(-4px)`;
        },
        { passive: true },
      );

      card.addEventListener('pointerleave', () => {
        card.style.transform = '';
      });
    }
  }

  // ----------------------------------------------------- kursor i parallax ----

  function pointer() {
    if (reduced || !fine) return;

    const glow = $('.cursor-glow');
    if (glow) {
      document.body.classList.add('has-pointer');
      let x = 0;
      let y = 0;
      let tx = 0;
      let ty = 0;

      window.addEventListener(
        'pointermove',
        (event) => {
          tx = event.clientX;
          ty = event.clientY;
        },
        { passive: true },
      );

      // eased follow — kursor nie skacze, tylko zostawia ślad
      const loop = () => {
        x += (tx - x) * 0.12;
        y += (ty - y) * 0.12;
        glow.style.transform = `translate3d(${x}px, ${y}px, 0)`;
        requestAnimationFrame(loop);
      };
      requestAnimationFrame(loop);
    }

    const aurora = $('.aurora');
    if (aurora) {
      window.addEventListener(
        'scroll',
        () => {
          aurora.style.setProperty('--par', `${window.scrollY * 0.06}px`);
        },
        { passive: true },
      );
    }
  }

  // --------------------------------------------------------------- nav ----

  function navbar() {
    const nav = $('.nav');
    const bar = $('.progress');
    if (!nav) return;

    const onScroll = () => {
      const y = window.scrollY;
      nav.classList.toggle('is-scrolled', y > 12);
      if (bar) {
        const max = document.documentElement.scrollHeight - window.innerHeight;
        bar.style.width = `${max > 0 ? (y / max) * 100 : 0}%`;
      }
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    onScroll();
  }

  // ----------------------------------------------------------- nagłówek ----

  function headline() {
    const h1 = $('.hero h1');
    if (!h1 || reduced) return;

    const text = h1.textContent.trim();
    h1.replaceChildren();
    [...text].forEach((char, i) => {
      const span = document.createElement('span');
      span.className = 'ch';
      span.style.setProperty('--i', i);
      span.textContent = char;
      h1.append(span);
    });
  }

  // ---------------------------------------------------------- terminal ----

  function terminal() {
    const code = $('.code');
    if (!code) return;

    const button = $('.code__bar button', code);
    if (button) {
      button.addEventListener('click', async () => {
        const text = $('pre', code)?.innerText ?? '';
        try {
          await navigator.clipboard.writeText(text);
        } catch {
          // Schowek bywa zablokowany (brak HTTPS, brak zgody) — wtedy
          // zaznaczamy treść, żeby user sam skopiował.
          const pre = $('pre', code);
          const range = document.createRange();
          range.selectNodeContents(pre);
          const sel = getSelection();
          sel.removeAllRanges();
          sel.addRange(range);
        }

        const label = button.textContent;
        button.textContent = 'skopiowane ✓';
        code.classList.add('is-copied');
        setTimeout(() => {
          button.textContent = label;
          code.classList.remove('is-copied');
        }, 1800);
      });
    }

    if (reduced) return;

    // Kursor terminala na końcu ostatniego wiersza.
    const pre = $('pre', code);
    if (pre && !pre.querySelector('.cur')) {
      const cursor = document.createElement('span');
      cursor.className = 'cur';
      pre.append(cursor);
    }
  }

  // ------------------------------------------------------------- FAQ ----

  function faq() {
    for (const item of $$('details')) {
      item.addEventListener('toggle', () => {
        if (!item.open) return;
        // Otwarta odpowiedź wypycha sąsiedzi — animacja jest na .reveal,
        // ale sama wysokość skacze. Delikatne rozjaśnienie sygnalizuje zmianę.
        item.animate(
          [
            { boxShadow: '0 0 0 0 rgba(139,92,246,0)' },
            { boxShadow: '0 0 34px -6px rgba(139,92,246,.55)' },
            { boxShadow: '0 0 0 0 rgba(139,92,246,0)' },
          ],
          { duration: 700, easing: 'cubic-bezier(.22,1,.36,1)' },
        );
      });
    }
  }

  // ------------------------------------------------------- scramble tekstu ----

  const GLYPHS = '!<>-_\\/[]{}—=+*^?#ABCDEFGHIJKLMNOPQRSTUVWXYZ';

  /** Krótki efekt „dekodowania” tekstu — litery losowe, aż do właściwych. */
  function scramble(element, text) {
    const queue = [...text].map((char) => ({ char, from: Math.floor(Math.random() * 18) }));
    const solid = new Set([' ', '/', '—', '-', '.']);
    let frame = 0;

    return new Promise((resolve) => {
      const tick = () => {
        let done = 0;
        let output = '';

        for (const item of queue) {
          if (solid.has(item.char) || frame >= item.from + 2) {
            output += item.char;
            done += 1;
          } else {
            output += GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
          }
        }

        element.textContent = output;
        frame += 1;

        if (done === queue.length) resolve();
        else requestAnimationFrame(tick);
      };

      requestAnimationFrame(tick);
    });
  }

  function scrambleHeadings() {
    const targets = $$('.scramble');
    if (!targets.length || reduced || !('IntersectionObserver' in window)) return;

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;

          const element = entry.target;
          const text = element.dataset.text ?? element.textContent;
          element.dataset.text = text;
          element.classList.add('is-working');

          scramble(element, text).finally(() => element.classList.remove('is-working'));
          io.unobserve(element);
        }
      },
      { threshold: 0.6 },
    );

    targets.forEach((n) => io.observe(n));
  }

  // --------------------------------------------------- magnetyczne przyciski --

  function magnet() {
    if (reduced || !fine) return;

    for (const button of $$('.magnet')) {
      const strength = Number(button.dataset.magnet || 9);

      button.addEventListener(
        'pointermove',
        (event) => {
          const box = button.getBoundingClientRect();
          const dx = (event.clientX - (box.left + box.width / 2)) * (strength / 100);
          const dy = (event.clientY - (box.top + box.height / 2)) * (strength / 100);
          button.style.setProperty('--mx', `${dx.toFixed(1)}px`);
          button.style.setProperty('--my', `${dy.toFixed(1)}px`);
        },
        { passive: true },
      );

      button.addEventListener('pointerleave', () => {
        button.classList.add('is-stuck');
        button.style.setProperty('--mx', '0px');
        button.style.setProperty('--my', '0px');
        setTimeout(() => button.classList.remove('is-stuck'), 520);
      });
    }
  }

  // --------------------------------------------------------------- ripple ----

  function ripple() {
    if (reduced || !fine) return;

    document.addEventListener(
      'pointerdown',
      (event) => {
        const host = event.target.closest('.btn, .dl__item, .code, details, summary');
        if (!host || getComputedStyle(host).position === 'static') return;
        if (getComputedStyle(host).overflow === 'hidden' && !host.classList.contains('btn')) return;

        const box = host.getBoundingClientRect();
        const size = Math.max(box.width, box.height) * 2.2;

        const dot = document.createElement('span');
        dot.className = 'ripple';
        dot.style.width = `${size}px`;
        dot.style.height = `${size}px`;
        dot.style.left = `${event.clientX - box.left}px`;
        dot.style.top = `${event.clientY - box.top}px`;

        host.append(dot);
        setTimeout(() => dot.remove(), 680);
      },
      { passive: true },
    );
  }

  // ----------------------------------------------------------- cząsteczki ----

  function particles() {
    if (reduced) return;

    const layer = $('.particles');
    if (!layer) return;

    // Gęstość zależy od pola powierzchni — na dużym ekranie nie robimy 200 kół.
    const count = Math.round(Math.min(46, (window.innerWidth * window.innerHeight) / 48000));

    for (let i = 0; i < count; i += 1) {
      const dot = document.createElement('span');
      const size = 1.5 + Math.random() * 3.5;

      dot.className = 'particle';
      dot.style.width = `${size}px`;
      dot.style.height = `${size}px`;
      dot.style.left = `${Math.random() * 100}%`;
      dot.style.top = `${Math.random() * 100}%`;
      dot.style.animationDuration = `${16 + Math.random() * 22}s`;
      dot.style.animationDelay = `${-Math.random() * 30}s`;
      dot.style.opacity = String(0.2 + Math.random() * 0.5);

      layer.append(dot);
    }
  }

  // ------------------------------------------------- parallax i przyciemnienie --

  function heroScroll() {
    const hero = $('.hero');
    const logo = $('.hero__logo');
    const scrim = $('.hero-scrim');
    if (!hero || !logo) return;

    let ticking = false;

    const update = () => {
      const rect = hero.getBoundingClientRect();
      const scrolled = Math.max(0, -rect.top);
      const k = Math.min(scrolled / Math.max(rect.height, 1), 1);

      logo.style.transform = `translate3d(0, ${(-k * 90).toFixed(1)}px, 0) scale(${(1 - k * 0.45).toFixed(3)})`;
      logo.style.opacity = String(1 - k * 0.9);

      if (scrim) scrim.style.opacity = String(Math.min(k * 1.15, 0.94));

      ticking = false;
    };

    window.addEventListener(
      'scroll',
      () => {
        if (ticking) return;
        ticking = true;
        requestAnimationFrame(update);
      },
      { passive: true },
    );

    update();
  }

  // --------------------------------------------------------- aktywna zakładka --

  function scrollSpy() {
    const links = $$('.nav__links a');
    const sections = links
      .map((a) => document.querySelector(a.getAttribute('href')))
      .filter(Boolean);

    if (!sections.length || !('IntersectionObserver' in window)) return;

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          const id = `#${entry.target.id}`;
          for (const link of links) {
            link.classList.toggle('is-active', link.getAttribute('href') === id);
          }
        }
      },
      { rootMargin: '-45% 0px -50% 0px' },
    );

    sections.forEach((s) => io.observe(s));
  }

  // ---------------------------------------------------------- start ----

  function start() {
    splash();
    headline();
    reveal();
    counters();
    spotlight();
    tilt();
    pointer();
    navbar();
    terminal();
    faq();
    scrambleHeadings();
    magnet();
    ripple();
    particles();
    heroScroll();
    scrollSpy();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start, { once: true });
  } else {
    start();
  }
})();