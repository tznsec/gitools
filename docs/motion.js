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
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start, { once: true });
  } else {
    start();
  }
})();