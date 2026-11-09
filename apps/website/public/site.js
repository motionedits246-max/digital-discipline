const pill = document.querySelector('#pill');
const panel = document.querySelector('#quick');
const workspace = document.querySelector('.workspace');
const views = [...document.querySelectorAll('[data-view]')];

function showPanel(open) {
  panel.hidden = !open;
  pill.setAttribute('aria-expanded', String(open));
  pill.querySelector('.plus').textContent = open ? '−' : '+';
}

function showView(id) {
  for (const link of views) {
    const active = link.dataset.view === id;
    link.setAttribute('aria-current', String(active));
    document.getElementById(link.dataset.view).hidden = !active;
  }
}

function restoreView() {
  const match = views.find(link => link.hash === location.hash);
  if (match) showView(match.dataset.view);
}

showView('activity');
restoreView();
window.addEventListener('hashchange', restoreView);
for (const link of views) {
  link.addEventListener('click', event => {
    if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    showView(link.dataset.view);
    history.replaceState(null, '', link.hash);
  });
}

pill.addEventListener('click', () => showPanel(panel.hidden));
document.addEventListener('pointerdown', event => {
  if (!panel.hidden && !event.target.closest('.controller')) showPanel(false);
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && !panel.hidden) {
    showPanel(false);
    pill.focus();
  }
});
document.querySelector('#expand').addEventListener('click', () => {
  showPanel(false);
  showView('activity');
  workspace.classList.add('expanding');
  views[0].focus({preventScroll: true});
});
workspace.addEventListener('animationend', event => {
  if (event.target === workspace) workspace.classList.remove('expanding');
});

if (matchMedia('(prefers-reduced-motion: no-preference)').matches && 'IntersectionObserver' in window) {
  const observer = new IntersectionObserver(entries => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add('reveal');
        observer.unobserve(entry.target);
      }
    }
  }, {threshold: 0.12});
  for (const element of document.querySelectorAll('.proof article,.privacy-list article')) {
    observer.observe(element);
  }
}
