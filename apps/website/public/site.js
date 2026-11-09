/**
 * Digital Discipline — Public Product Website Script
 * Interactive product simulation, focus timer, workspace tabs, and accessible controls.
 */

(() => {
  'use strict';

  // --- 1. Interactive Focus Timer (Hero Instrument) ---
  const heroTimerEl = document.getElementById('hero-timer');
  const pillTimerEl = document.getElementById('pill-timer');
  const playPauseBtn = document.getElementById('hero-play-pause');
  const iconPause = playPauseBtn?.querySelector('.icon-pause');
  const iconPlay = playPauseBtn?.querySelector('.icon-play');
  const addTimeBtn = document.getElementById('btn-add-time');

  let remainingSeconds = 18 * 60 + 42; // 18:42
  let isTimerRunning = false;
  let timerInterval = null;

  function formatTime(seconds) {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  }

  function updateTimerDisplays() {
    const timeStr = formatTime(remainingSeconds);
    if (heroTimerEl) heroTimerEl.textContent = timeStr;
    if (pillTimerEl) pillTimerEl.textContent = timeStr;
  }

  function startTimer() {
    if (isTimerRunning) return;
    isTimerRunning = true;
    if (iconPause) iconPause.hidden = false;
    if (iconPlay) iconPlay.hidden = true;
    if (playPauseBtn) playPauseBtn.setAttribute('aria-label', 'Pause timer');

    timerInterval = setInterval(() => {
      if (remainingSeconds > 0) {
        remainingSeconds--;
        updateTimerDisplays();
      } else {
        pauseTimer();
      }
    }, 1000);
  }

  function pauseTimer() {
    isTimerRunning = false;
    clearInterval(timerInterval);
    if (iconPause) iconPause.hidden = true;
    if (iconPlay) iconPlay.hidden = false;
    if (playPauseBtn) playPauseBtn.setAttribute('aria-label', 'Resume timer');
  }

  playPauseBtn?.addEventListener('click', () => {
    if (isTimerRunning) {
      pauseTimer();
    } else {
      startTimer();
    }
  });

  addTimeBtn?.addEventListener('click', () => {
    remainingSeconds += 5 * 60;
    updateTimerDisplays();
    if (!isTimerRunning) startTimer();
  });

  // --- 2. Floating Controller Pill & Quick Panel ---
  const pillBtn = document.getElementById('pill');
  const quickPanel = document.getElementById('quick-panel');
  const controllerMockup = document.getElementById('controller-mockup');
  const pillPlus = document.getElementById('pill-plus');
  const btnOpenWorkspace = document.getElementById('btn-open-workspace');

  function toggleQuickPanel(open) {
    const shouldOpen = typeof open === 'boolean' ? open : quickPanel.hidden;
    quickPanel.hidden = !shouldOpen;
    pillBtn?.setAttribute('aria-expanded', String(shouldOpen));
    if (pillPlus) pillPlus.textContent = shouldOpen ? '−' : '+';
  }

  pillBtn?.addEventListener('click', () => toggleQuickPanel());

  document.addEventListener('pointerdown', (e) => {
    if (quickPanel && !quickPanel.hidden && !controllerMockup?.contains(e.target)) {
      toggleQuickPanel(false);
    }
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && quickPanel && !quickPanel.hidden) {
      toggleQuickPanel(false);
      pillBtn?.focus();
    }
  });

  btnOpenWorkspace?.addEventListener('click', () => {
    toggleQuickPanel(false);
    switchView('activity');
  });

  // --- 3. Workspace Tab Views ---
  const modeTabs = document.querySelectorAll('.mode-tab');
  const views = {
    activity: document.getElementById('view-activity'),
    focus: document.getElementById('view-focus'),
    insights: document.getElementById('view-insights'),
  };
  const breadcrumbEl = document.getElementById('active-breadcrumb');

  const breadcrumbTitles = {
    activity: 'Activity History',
    focus: 'Focus Profile',
    insights: 'Insights & Heatmap',
  };

  function switchView(targetViewId) {
    if (!views[targetViewId]) return;

    // Update Tab states
    modeTabs.forEach((tab) => {
      const active = tab.dataset.view === targetViewId;
      tab.classList.toggle('active', active);
      tab.setAttribute('aria-selected', String(active));
    });

    // Update Panels
    Object.keys(views).forEach((key) => {
      if (views[key]) {
        views[key].hidden = key !== targetViewId;
      }
    });

    // Update breadcrumb
    if (breadcrumbEl && breadcrumbTitles[targetViewId]) {
      breadcrumbEl.textContent = breadcrumbTitles[targetViewId];
    }
  }

  modeTabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      const viewId = tab.dataset.view;
      if (viewId) switchView(viewId);
    });

    // Keyboard accessibility for tablist
    tab.addEventListener('keydown', (e) => {
      const tabsArray = Array.from(modeTabs);
      const currentIndex = tabsArray.indexOf(tab);
      let nextIndex = null;

      if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
        nextIndex = (currentIndex + 1) % tabsArray.length;
      } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
        nextIndex = (currentIndex - 1 + tabsArray.length) % tabsArray.length;
      } else if (e.key === 'Home') {
        nextIndex = 0;
      } else if (e.key === 'End') {
        nextIndex = tabsArray.length - 1;
      }

      if (nextIndex !== null) {
        e.preventDefault();
        tabsArray[nextIndex].focus();
        const nextViewId = tabsArray[nextIndex].dataset.view;
        if (nextViewId) switchView(nextViewId);
      }
    });
  });

  // Focus View Demo Buttons
  const demoPauseBtn = document.getElementById('demo-toggle-pause');
  demoPauseBtn?.addEventListener('click', () => {
    if (isTimerRunning) {
      pauseTimer();
      demoPauseBtn.querySelector('span').textContent = 'Resume Session';
    } else {
      startTimer();
      demoPauseBtn.querySelector('span').textContent = 'Pause Session';
    }
  });

  const demoEmergencyBtn = document.getElementById('demo-emergency-bypass');
  demoEmergencyBtn?.addEventListener('click', () => {
    demoEmergencyBtn.textContent = 'Unlocked (30m Lease)';
    demoEmergencyBtn.style.background = '#3c2317';
    demoEmergencyBtn.style.color = '#ffc078';
    setTimeout(() => {
      demoEmergencyBtn.innerHTML = '<svg aria-hidden="true"><use href="icons.svg#shield"/></svg><span>30-min Emergency Unlock</span>';
      demoEmergencyBtn.style.background = '';
      demoEmergencyBtn.style.color = '';
    }, 4000);
  });

  // --- 4. 52-Week Interactive Heatmap Grid ---
  const heatmapGrid = document.getElementById('heatmap-grid');
  const heatmapTooltip = document.getElementById('heatmap-tooltip');
  const tooltipDate = document.getElementById('tooltip-date');
  const tooltipHours = document.getElementById('tooltip-hours');

  if (heatmapGrid) {
    const today = new Date(2026, 10, 9); // Nov 9, 2026
    const totalDays = 52 * 7; // 364 days

    for (let i = totalDays - 1; i >= 0; i--) {
      const dayDate = new Date(today);
      dayDate.setDate(today.getDate() - i);

      const dayOfWeek = dayDate.getDay(); // 0 is Sunday
      const isWeekend = dayOfWeek === 0 || dayOfWeek === 6;

      // Realistic mock focus generation matching demo.ts
      let level = 0;
      let hours = 0;

      if (isWeekend) {
        if ((i * 13) % 5 === 0) {
          level = 1;
          hours = 1.8;
        } else if ((i * 7) % 4 === 0) {
          level = 2;
          hours = 2.4;
        }
      } else {
        const seed = (i * 17 + Math.floor(i / 7) * 3) % 100;
        if (seed > 80) {
          level = 4;
          hours = 6.2;
        } else if (seed > 50) {
          level = 3;
          hours = 4.8;
        } else if (seed > 25) {
          level = 2;
          hours = 3.6;
        } else if (seed > 10) {
          level = 1;
          hours = 2.1;
        }
      }

      const cell = document.createElement('div');
      cell.className = `heat-cell heat-box lvl-${level}`;
      cell.setAttribute('tabindex', '0');
      cell.setAttribute('role', 'gridcell');

      const dateStr = dayDate.toISOString().slice(0, 10);
      cell.dataset.date = dateStr;
      cell.dataset.hours = hours > 0 ? `${hours} hrs deep focus` : 'Rest day / 0 hrs';
      cell.setAttribute('aria-label', `${dateStr}: ${cell.dataset.hours}`);

      cell.addEventListener('mouseenter', (e) => showTooltip(e.target));
      cell.addEventListener('focus', (e) => showTooltip(e.target));
      cell.addEventListener('mouseleave', hideTooltip);
      cell.addEventListener('blur', hideTooltip);

      heatmapGrid.appendChild(cell);
    }
  }

  function showTooltip(target) {
    if (!heatmapTooltip || !target) return;
    if (tooltipDate) tooltipDate.textContent = target.dataset.date || '';
    if (tooltipHours) tooltipHours.textContent = target.dataset.hours || '';

    const rect = target.getBoundingClientRect();
    const parentRect = target.closest('.heatmap-section')?.getBoundingClientRect();

    if (parentRect) {
      heatmapTooltip.style.left = `${rect.left - parentRect.left - 40}px`;
      heatmapTooltip.style.top = `${rect.top - parentRect.top - 46}px`;
      heatmapTooltip.hidden = false;
    }
  }

  function hideTooltip() {
    if (heatmapTooltip) heatmapTooltip.hidden = true;
  }

  // --- 5. Simulated v0.4.4 Notice Bar (3-Second Contract Demo) ---
  const noticeBox = document.getElementById('notice-demo-box');
  const timerBar = document.getElementById('notice-timer-bar');
  const acceptBtn = document.getElementById('notice-accept');
  const changeBtn = document.getElementById('notice-change');

  let noticeTimeout = null;
  let noticeTimeRemaining = 3000;
  let noticeStart = Date.now();
  let isNoticePaused = false;

  const demoServices = [
    { title: 'Obsidian detected', sub: 'Assigned category: Writing', btn: 'Keep Writing' },
    { title: 'Visual Studio Code detected', sub: 'Assigned category: Development', btn: 'Keep Development' },
    { title: 'Figma detected', sub: 'Assigned category: Design', btn: 'Keep Design' },
    { title: 'Spotify detected', sub: 'Assigned category: Audio', btn: 'Keep Audio' },
  ];
  let currentServiceIndex = 0;

  function runNoticeCountdown() {
    if (isNoticePaused || !timerBar) return;
    const elapsed = Date.now() - noticeStart;
    const progress = Math.max(0, 1 - elapsed / 3000);
    timerBar.style.width = `${progress * 100}%`;

    if (elapsed < 3000) {
      requestAnimationFrame(runNoticeCountdown);
    } else {
      // Auto-dismiss cycle
      cycleNotice();
    }
  }

  function resetNoticeCountdown() {
    noticeStart = Date.now();
    isNoticePaused = false;
    if (timerBar) timerBar.style.width = '100%';
    runNoticeCountdown();
  }

  function cycleNotice() {
    currentServiceIndex = (currentServiceIndex + 1) % demoServices.length;
    const svc = demoServices[currentServiceIndex];
    if (noticeBox) {
      const strongEl = noticeBox.querySelector('strong');
      const spanEl = noticeBox.querySelector('span');
      if (strongEl) strongEl.textContent = `Service Suggestion: ${svc.title}`;
      if (spanEl) spanEl.textContent = `${svc.sub} · Click to change`;
      if (acceptBtn) acceptBtn.textContent = svc.btn;
    }
    resetNoticeCountdown();
  }

  noticeBox?.addEventListener('mouseenter', () => {
    isNoticePaused = true;
  });

  noticeBox?.addEventListener('mouseleave', () => {
    noticeStart = Date.now() - (1 - parseFloat(timerBar?.style.width || '100') / 100) * 3000;
    isNoticePaused = false;
    runNoticeCountdown();
  });

  acceptBtn?.addEventListener('click', () => {
    acceptBtn.textContent = 'Saved ✓';
    acceptBtn.style.background = '#275234';
    acceptBtn.style.color = '#99eec0';
    setTimeout(() => {
      acceptBtn.style.background = '';
      acceptBtn.style.color = '';
      cycleNotice();
    }, 800);
  });

  changeBtn?.addEventListener('click', () => {
    cycleNotice();
  });

  // Start notice cycle
  resetNoticeCountdown();

  // --- 6. SHA-256 Checksum One-Click Copy ---
  const copyBtn = document.getElementById('copy-hash-btn');
  const copyText = document.getElementById('copy-text');
  const shaHash = document.getElementById('sha-hash');

  copyBtn?.addEventListener('click', async () => {
    const textToCopy = shaHash?.textContent?.trim() || '';
    try {
      await navigator.clipboard.writeText(textToCopy);
      if (copyText) copyText.textContent = 'Copied!';
      copyBtn.style.borderColor = 'var(--green)';
      copyBtn.style.color = 'var(--green)';
      setTimeout(() => {
        if (copyText) copyText.textContent = 'Copy Hash';
        copyBtn.style.borderColor = '';
        copyBtn.style.color = '';
      }, 2200);
    } catch {
      // Fallback if clipboard API is restricted
      const textarea = document.createElement('textarea');
      textarea.value = textToCopy;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      if (copyText) copyText.textContent = 'Copied!';
      setTimeout(() => {
        if (copyText) copyText.textContent = 'Copy Hash';
      }, 2200);
    }
  });

  // --- 7. Mobile Navigation Drawer ---
  const menuToggle = document.getElementById('menu-toggle');
  const mobileDrawer = document.getElementById('mobile-drawer');
  const drawerLinks = document.querySelectorAll('.drawer-link, .drawer-actions a');

  function toggleMobileMenu(open) {
    const shouldOpen = typeof open === 'boolean' ? open : mobileDrawer.hidden;
    mobileDrawer.hidden = !shouldOpen;
    menuToggle?.setAttribute('aria-expanded', String(shouldOpen));
    document.body.style.overflow = shouldOpen ? 'hidden' : '';
  }

  menuToggle?.addEventListener('click', () => toggleMobileMenu());

  drawerLinks.forEach((link) => {
    link.addEventListener('click', () => toggleMobileMenu(false));
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && mobileDrawer && !mobileDrawer.hidden) {
      toggleMobileMenu(false);
      menuToggle?.focus();
    }
  });

  // --- 8. IntersectionObserver Scroll Reveal ---
  if (matchMedia('(prefers-reduced-motion: no-preference)').matches && 'IntersectionObserver' in window) {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('reveal');
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.12 }
    );

    document
      .querySelectorAll('.walkthrough-card, .proof-card, .tech-card, .comp-col, .install-step, .faq-item')
      .forEach((el) => observer.observe(el));
  }
})();
