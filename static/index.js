addEventListener("DOMContentLoaded", () => {
  for (let time of document.body.getElementsByTagName('time')) {
    time.setAttribute('title', new Date(time.textContent));
  }

  let ordinalsLink = document.getElementById('ordinals-link');
  if (ordinalsLink) {
    ordinalsLink.href = 'https://ordinals.com' + location.pathname + location.search;
  }

  let titleLinks = document.querySelector('.title-links');
  if (titleLinks) {
    let urls = [];
    for (let dt of document.querySelectorAll('dt')) {
      if (dt.textContent.trim().toLowerCase() === 'links') {
        let dd = dt.nextElementSibling;
        if (dd) {
          for (let item of dd.querySelectorAll('li, a')) {
            let raw = item.tagName === 'A' ? item.href : item.textContent.trim();
            try {
              let u = new URL(raw);
              if ((u.protocol === 'http:' || u.protocol === 'https:') && u.hostname) {
                urls.push(u);
              }
            } catch (_) {}
          }
        }
        break;
      }
    }
    let x = null, website = null;
    for (let u of urls) {
      let host = u.hostname.replace(/^www\./, '');
      if (!x && (host === 'x.com' || host === 'twitter.com')) x = u.href;
      else if (!website && host !== 'x.com' && host !== 'twitter.com') website = u.href;
    }
    let entries = [];
    if (website) entries.push({href: website, src: '/static/link.svg', label: 'website'});
    if (x) entries.push({href: x, src: '/static/x.svg', label: 'X'});
    for (let e of entries) {
      let a = document.createElement('a');
      a.href = e.href;
      a.target = '_blank';
      a.rel = 'noopener noreferrer';
      a.title = e.label;
      let img = document.createElement('img');
      img.className = 'icon';
      img.src = e.src;
      img.alt = e.label;
      a.appendChild(img);
      titleLinks.appendChild(a);
    }
  }

  const GALLERY_PAGE_SIZE = 20;

  for (let row of document.querySelectorAll('.gallery-row')) {
    let track = row.querySelector('.thumbnails');
    let prevBtn = row.querySelector('.gallery-prev');
    let nextBtn = row.querySelector('.gallery-next');
    if (!track) continue;

    let items = track.querySelectorAll(':scope > a');
    let total = items.length;
    let toolbar = row.parentElement.previousElementSibling;
    let viewBtns = toolbar && toolbar.classList.contains('with-toolbar')
      ? toolbar.querySelectorAll('.gallery-view-btn')
      : [];
    let mode = 'scroll';
    let page = 0;
    let pageCount = Math.max(1, Math.ceil(total / GALLERY_PAGE_SIZE));

    function applyPage() {
      let start = page * GALLERY_PAGE_SIZE;
      let end = start + GALLERY_PAGE_SIZE;
      items.forEach((item, idx) => {
        item.toggleAttribute('hidden', mode === 'all' && (idx < start || idx >= end));
      });
    }

    function updateArrows() {
      if (mode === 'scroll') {
        let max = track.scrollWidth - track.clientWidth;
        if (prevBtn) prevBtn.disabled = track.scrollLeft <= 1;
        if (nextBtn) nextBtn.disabled = track.scrollLeft >= max - 1;
      } else {
        if (prevBtn) prevBtn.disabled = page <= 0;
        if (nextBtn) nextBtn.disabled = page >= pageCount - 1;
      }
    }

    function setMode(nextMode) {
      if (mode === nextMode) return;
      mode = nextMode;
      row.classList.toggle('gallery-mode-scroll', mode === 'scroll');
      row.classList.toggle('gallery-mode-all', mode === 'all');
      viewBtns.forEach(btn => btn.classList.toggle('active', btn.dataset.mode === mode));
      page = 0;
      applyPage();
      updateArrows();
    }

    if (prevBtn) prevBtn.addEventListener('click', () => {
      if (mode === 'scroll') {
        track.scrollBy({left: -track.clientWidth, behavior: 'smooth'});
      } else if (page > 0) {
        page--; applyPage(); updateArrows();
      }
    });
    if (nextBtn) nextBtn.addEventListener('click', () => {
      if (mode === 'scroll') {
        track.scrollBy({left: track.clientWidth, behavior: 'smooth'});
      } else if (page < pageCount - 1) {
        page++; applyPage(); updateArrows();
      }
    });
    viewBtns.forEach(btn => btn.addEventListener('click', () => setMode(btn.dataset.mode)));

    track.addEventListener('scroll', updateArrows);
    addEventListener('resize', updateArrows);
    updateArrows();

    if (matchMedia('(max-width: 38rem)').matches && track.scrollWidth > track.clientWidth) {
      let observer = new IntersectionObserver((entries) => {
        for (let entry of entries) {
          if (!entry.isIntersecting) continue;
          observer.disconnect();
          setTimeout(() => row.classList.add('scroll-hint'), 1200);
        }
      }, {threshold: 0.5});
      observer.observe(track);
    }
  }

  let next = document.querySelector('a.next');
  let prev = document.querySelector('a.prev');

  window.addEventListener('keydown', e => {
    if (document.activeElement.tagName == 'INPUT') {
      return;
    }

    switch (e.key) {
      case 'ArrowRight':
        if (next) {
          window.location = next.href;
        }
        return;
      case 'ArrowLeft':
        if (prev) {
          window.location = prev.href;
        }
        return;
    }
  });

  const search = document.querySelector('form[action="/search"]');
  const query = search.querySelector('input[name="query"]');

  search.addEventListener('submit', (e) => {
    if (!query.value) {
      e.preventDefault();
    }
  });

  let collapse = document.getElementsByClassName('collapse');

  let context = document.createElement('canvas').getContext('2d');

  function resize() {
    for (let node of collapse) {
      if (!('original' in node.dataset)) {
        node.dataset.original = node.textContent.trim();
      }
      let original = node.dataset.original;
      let length = original.length;
      let width = node.clientWidth;
      if (width == 0) {
        width = node.parentNode.getBoundingClientRect().width;
      }
      context.font = window.getComputedStyle(node).font;
      let capacity = width / (context.measureText(original).width / length);
      if (capacity >= length) {
        node.textContent = original
      } else {
        let count = Math.floor((capacity - 1) / 2);
        let start = original.substring(0, count);
        let end = original.substring(length - count);
        node.textContent = `${start}…${end}`;
      }
    }
  }

  function copy(e) {
    if ('original' in e.target.dataset && window.getSelection().toString().includes('…')) {
      e.clipboardData.setData('text/plain', e.target.dataset.original);
      e.preventDefault();
    }
  }

  addEventListener('resize', resize);

  addEventListener('copy', copy);

  document
    .querySelectorAll(`nav a[href="${CSS.escape(window.location.pathname)}"]`)
    .forEach(a => a.classList.add('active'));

  resize();
});
