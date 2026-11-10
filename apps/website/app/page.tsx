"use client";

import { useEffect, useState } from "react";
import {
  Activity,
  ArrowDown,
  ArrowRight,
  ArrowUpRight,
  BarChart3,
  CalendarDays,
  Check,
  ChevronRight,
  Clock3,
  Code2,
  Download,
  EyeOff,
  LayoutDashboard,
  LockKeyhole,
  Menu,
  Minus,
  Monitor,
  Pause,
  Play,
  RotateCcw,
  Shield,
  SlidersHorizontal,
  Square,
  Target,
  Video,
  X,
  Plus,
} from "lucide-react";

const DOWNLOAD = "/downloads/Digital-Discipline-0.4.4-x64-setup.exe";
const CHECKSUM =
  "cee81d66fe23008bc8042de4e762210fc696898ef08575dee5f020298497f9d3";
type View = "overview" | "focus" | "insights";

function Mark({ className = "" }: { className?: string }) {
  return (
    <svg
      className={className}
      width="26"
      height="26"
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
    >
      <circle
        cx="16"
        cy="16"
        r="12.5"
        stroke="currentColor"
        strokeWidth="1.7"
      />
      <path d="M16 3.5a12.5 12.5 0 0 0 0 25V3.5Z" fill="currentColor" />
      <path d="M16 3.5v25" stroke="currentColor" strokeWidth="1.7" />
    </svg>
  );
}
function WindowsMark() {
  return (
    <svg
      width="17"
      height="17"
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d="M1 3.2 8.7 2v7H1V3.2ZM9.8 1.8 19 0v9H9.8V1.8ZM1 10.2h7.7v7L1 16v-5.8ZM9.8 10.2H19v9.6L9.8 18v-7.8Z" />
    </svg>
  );
}
function DownloadLink({
  className = "",
  children = "Download for Windows",
}: {
  className?: string;
  children?: React.ReactNode;
}) {
  return (
    <a
      href={DOWNLOAD}
      download="Digital-Discipline-0.4.4-x64-setup.exe"
      className={"download-button " + className}
    >
      <WindowsMark />
      <span>{children}</span>
      <ArrowDown size={16} />
    </a>
  );
}
const heat = Array.from({ length: 91 }, (_, i) => {
  const n = (i * 17 + i * i * 3) % 23;
  return n < 7 ? 0 : n < 12 ? 1 : n < 17 ? 2 : n < 21 ? 3 : 4;
});
function Heatmap({ large = false }: { large?: boolean }) {
  return (
    <div
      className={"heatmap " + (large ? "large" : "")}
      role="img"
      aria-label="Illustrative activity over thirteen weeks"
    >
      {heat.map((level, i) => (
        <span key={i} className={"heat level-" + level} />
      ))}
    </div>
  );
}
const sessions = [
  { start: 30, minutes: 102, type: "work" },
  { start: 132, minutes: 37, type: "leisure" },
  { start: 169, minutes: 26, type: "other" },
  { start: 195, minutes: 25, type: "idle" },
  { start: 220, minutes: 119, type: "work" },
  { start: 339, minutes: 59, type: "other" },
  { start: 398, minutes: 35, type: "leisure" },
  { start: 433, minutes: 20, type: "other" },
];
function Timeline() {
  return (
    <div className="sample-timeline">
      <div className="timeline-hours">
        <span>8 AM</span>
        <span>10 AM</span>
        <span>12 PM</span>
        <span>2 PM</span>
        <span>4 PM</span>
        <span>6 PM</span>
      </div>
      <div className="timeline-lane">
        {sessions.map((s, i) => (
          <span
            key={i}
            className={"timeline-segment " + s.type}
            style={{ left: s.start / 6 + "%", width: s.minutes / 6 + "%" }}
          />
        ))}
      </div>
      <div className="timeline-legend">
        <span>
          <i />
          Development
        </span>
        <span>
          <i />
          Entertainment
        </span>
        <span>
          <i />
          Other
        </span>
        <span>
          <i />
          Idle
        </span>
      </div>
    </div>
  );
}
function Overview() {
  return (
    <>
      <div className="screen-heading">
        <div>
          <span className="screen-eyebrow">YOUR TIME, AT A GLANCE</span>
          <h3>Overview</h3>
          <p>A little clarity for your day.</p>
        </div>
        <span className="date-chip">
          <CalendarDays size={12} />
          Today
          <ChevronRight size={10} />
        </span>
      </div>
      <div className="screen-metrics">
        {[
          {
            title: "Active today",
            value: "6h 38m",
            sub: "Foreground · excluding idle",
            icon: Activity,
          },
          {
            title: "Development",
            value: "3h 41m",
            sub: "Time building things",
            icon: Code2,
          },
          {
            title: "Short-form plays",
            value: "14",
            sub: "9 unique · 5 repeat",
            icon: Video,
          },
          {
            title: "Entertainment",
            value: "1h 12m",
            sub: "Your chosen categories",
            icon: Clock3,
          },
        ].map((m) => (
          <div className="screen-metric" key={m.title}>
            <div>
              {m.title}
              <m.icon size={12} />
            </div>
            <strong>{m.value}</strong>
            <small>{m.sub}</small>
          </div>
        ))}
      </div>
      <div className="screen-panel timeline-panel">
        <div className="screen-panel-title">
          <div>
            <strong>Activity timeline</strong>
            <small>Foreground sessions · Today</small>
          </div>
          <span>Day view</span>
        </div>
        <Timeline />
      </div>
      <div className="screen-bottom">
        <div className="screen-panel recent-panel">
          <div className="screen-panel-title">
            <strong>Most used apps</strong>
            <ArrowUpRight size={12} />
          </div>
          {[
            {
              letter: "C",
              name: "Code",
              category: "Development",
              time: "3h 41m",
            },
            {
              letter: "B",
              name: "Brave",
              category: "Entertainment",
              time: "1h 12m",
            },
            { letter: "F", name: "Figma", category: "Design", time: "1h 45m" },
          ].map((a) => (
            <div className="app-row" key={a.letter}>
              <span className="app-avatar">{a.letter}</span>
              <div>
                <strong>{a.name}</strong>
                <small>{a.category}</small>
              </div>
              <span>{a.time}</span>
            </div>
          ))}
        </div>
        <div className="screen-panel consistency-panel">
          <div className="screen-panel-title">
            <strong>Show up. Build momentum.</strong>
          </div>
          <p>Small steps add up.</p>
          <Heatmap />
          <div className="heatmap-caption">
            <span>Last 13 weeks</span>
            <span>
              Less <i />
              <i />
              <i /> More
            </span>
          </div>
        </div>
      </div>
    </>
  );
}
function FocusScreen() {
  const [running, setRunning] = useState(false),
    [seconds, setSeconds] = useState(25 * 60);
  useEffect(() => {
    if (!running) return;
    const id = setInterval(
      () =>
        setSeconds((n) => {
          if (n <= 1) {
            setRunning(false);
            return 0;
          }
          return n - 1;
        }),
      1000,
    );
    return () => clearInterval(id);
  }, [running]);
  return (
    <>
      <div className="screen-heading">
        <div>
          <span className="screen-eyebrow">ONE THING AT A TIME</span>
          <h3>Focus &amp; Goals</h3>
          <p>A small commitment. A clear next step.</p>
        </div>
        <Target size={18} />
      </div>
      <div className="focus-preview screen-panel">
        <div className="focus-preview-top">
          <span>WORK BLOCK</span>
          <span className="soft-badge">
            {running ? "Running" : seconds === 1500 ? "Ready" : "Paused"}
          </span>
        </div>
        <div className="focus-clock">
          <svg viewBox="0 0 170 170" aria-hidden="true">
            <circle cx="85" cy="85" r="75" />
            <circle
              cx="85"
              cy="85"
              r="75"
              className="clock-progress"
              style={{ strokeDashoffset: 471 * (1 - seconds / 1500) }}
            />
          </svg>
          <div>
            <strong>
              {Math.floor(seconds / 60)
                .toString()
                .padStart(2, "0")}
              :{(seconds % 60).toString().padStart(2, "0")}
            </strong>
            <small>time for what matters</small>
          </div>
        </div>
        <strong className="focus-task">Finish the first draft</strong>
        <p>Work for 25 minutes. Take a 5-minute break.</p>
        <div className="focus-controls">
          <button onClick={() => setRunning((n) => !n)}>
            {running ? <Pause size={13} /> : <Play size={13} />}{" "}
            {running
              ? "Pause preview"
              : seconds === 1500
                ? "Start preview"
                : "Resume preview"}
          </button>
          <button
            className="reset-focus"
            aria-label="Reset sample focus timer"
            onClick={() => {
              setRunning(false);
              setSeconds(1500);
            }}
          >
            <RotateCcw size={13} />
          </button>
        </div>
      </div>
      <div className="sample-scope">
        <Shield size={13} />
        <div>
          <strong>A boundary for this session</strong>
          <span>youtube.com/shorts · restricted</span>
        </div>
        <Check size={13} />
      </div>
      <p className="screen-demo-note">
        A sample timer to explore the interface. No activity is recorded.
      </p>
    </>
  );
}
function InsightsScreen() {
  return (
    <>
      <div className="screen-heading">
        <div>
          <span className="screen-eyebrow">NOTICE YOUR PATTERNS</span>
          <h3>Behavior Insights</h3>
          <p>Understand the hours that make your day.</p>
        </div>
        <BarChart3 size={18} />
      </div>
      <div className="screen-panel insight-panel">
        <div className="screen-panel-title">
          <strong>Your week, in perspective</strong>
          <span>Sample week</span>
        </div>
        <div className="week-chart">
          {[52, 78, 62, 88, 73, 34, 21].map((v, i) => (
            <div className="week-column" key={i}>
              <div className="week-bar">
                <span style={{ height: v + "%" }} />
                <i style={{ height: (100 - v) * 0.35 + "%" }} />
              </div>
              <small>{["M", "T", "W", "T", "F", "S", "S"][i]}</small>
            </div>
          ))}
        </div>
        <div className="chart-key">
          <span>
            <i />
            Development
          </span>
          <span>
            <i />
            Entertainment
          </span>
        </div>
      </div>
      <div className="screen-panel insight-pattern">
        <span className="pattern-icon">
          <Activity size={17} />
        </span>
        <div>
          <strong>A rhythm you can actually see</strong>
          <p>
            Explore activity by hour and category. Choose what you want your
            next week to look like.
          </p>
        </div>
      </div>
      <div className="screen-panel insight-heat">
        <div className="screen-panel-title">
          <strong>Consistency, one day at a time</strong>
        </div>
        <Heatmap large />
      </div>
      <p className="screen-demo-note">
        Illustrative activity. Your desktop reports use your own observations.
      </p>
    </>
  );
}
function AppPreview() {
  const [view, setView] = useState<View>("overview"),
    [privateMode, setPrivateMode] = useState(false);
  return (
    <div className="product-window" id="product-preview">
      <div className="product-titlebar">
        <span>
          <Mark />
          digital discipline
        </span>
        <div className="preview-badge">
          <i />
          Interactive preview
        </div>
        <div className="window-decoration" aria-hidden="true">
          <Minus size={13} />
          <Square size={10} />
          <X size={13} />
        </div>
      </div>
      <div className="product-interior">
        <aside className="product-sidebar">
          <span className="sidebar-label">YOUR WORKSPACE</span>
          <nav aria-label="Interactive product preview">
            {[
              { id: "overview", title: "Overview", icon: LayoutDashboard },
              { id: "focus", title: "Focus & Goals", icon: Target },
              { id: "insights", title: "Behavior Insights", icon: BarChart3 },
            ].map((item) => (
              <button
                key={item.id}
                aria-label={item.title}
                aria-pressed={view === item.id}
                className={view === item.id ? "active" : ""}
                onClick={() => {
                  setView(item.id as View);
                  setPrivateMode(false);
                }}
              >
                <item.icon size={14} />
                <span>{item.title}</span>
                {view === item.id && <i />}
              </button>
            ))}
          </nav>
          <div className="sidebar-bottom">
            <button
              aria-pressed={privateMode}
              onClick={() => setPrivateMode((n) => !n)}
            >
              <EyeOff size={14} />
              Privacy preview
            </button>
            <div>
              <span className="sample-dot" />
              <span>
                Sample workspace<small>No real activity</small>
              </span>
            </div>
          </div>
        </aside>
        <div className="product-screen">
          {privateMode ? (
            <div className="privacy-preview">
              <Shield size={38} />
              <h3>
                Your details.
                <br />
                Your decision.
              </h3>
              <p>
                Hide the sample activity details while you explore. Your privacy
                stays in your hands.
              </p>
              <button onClick={() => setPrivateMode(false)}>
                Show sample workspace
                <ArrowRight size={13} />
              </button>
            </div>
          ) : view === "overview" ? (
            <Overview />
          ) : view === "focus" ? (
            <FocusScreen />
          ) : (
            <InsightsScreen />
          )}
          <span className="sample-watermark">SAMPLE DATA</span>
        </div>
      </div>
    </div>
  );
}
function PillDemo() {
  const [open, setOpen] = useState(false),
    [paused, setPaused] = useState(false);
  return (
    <div className="floating-controller">
      {open && (
        <div className="quick-panel">
          <div className="quick-top">
            <span>
              <Mark />
              Digital Discipline
            </span>
            <button
              aria-label="Close quick panel preview"
              onClick={() => setOpen(false)}
            >
              <X size={14} />
            </button>
          </div>
          <small>CURRENT ACTIVITY · PREVIEW</small>
          <strong>{paused ? "Tracking paused" : "Code"}</strong>
          <p>
            {paused
              ? "You choose when to resume."
              : "Development · local observation"}
          </p>
          <div className="quick-numbers">
            <span>
              Active today<strong>6h 38m</strong>
            </span>
            <span>
              Short plays<strong>14</strong>
            </span>
          </div>
          <button className="quick-pause" onClick={() => setPaused((n) => !n)}>
            {paused ? <Play size={12} /> : <Pause size={12} />}{" "}
            {paused ? "Resume preview" : "Pause preview"}
          </button>
        </div>
      )}
      <button
        className={"floating-pill " + (paused ? "is-paused" : "")}
        aria-expanded={open}
        aria-label={
          open
            ? "Collapse floating controller preview"
            : "Open floating controller preview"
        }
        onClick={() => setOpen((n) => !n)}
      >
        <span className="pill-status-dot" />
        <strong>{paused ? "Paused" : "Code"}</strong>
        <span className="pill-stat">
          <Video size={13} />
          14
        </span>
        <ChevronRight size={15} className={open ? "turned" : ""} />
      </button>
      <div className="pill-annotation">
        <span />
        Small when you want. There when you need.
      </div>
    </div>
  );
}
function BrowserBoundary() {
  const [limited, setLimited] = useState(true);
  return (
    <div className="boundary-preview">
      <div className="boundary-toolbar">
        <span>
          <Shield size={14} />
          Your browser limits
        </span>
        <span className="soft-badge">SAMPLE</span>
      </div>
      <div className="boundary-rule">
        <span className="video-symbol">
          <Video size={19} />
        </span>
        <div>
          <strong>YouTube Shorts</strong>
          <small>youtube.com/shorts</small>
        </div>
        <button
          className={"sample-switch " + (limited ? "enabled" : "")}
          aria-label="Toggle sample Shorts restriction"
          role="switch"
          aria-checked={limited}
          onClick={() => setLimited((n) => !n)}
        >
          <span />
        </button>
      </div>
      <div className="budget-description">
        <span>{limited ? "11 of 15 minutes used" : "Sample limit paused"}</span>
        <strong>{limited ? "4m left" : "Your choice"}</strong>
      </div>
      <div className={"budget-meter " + (!limited ? "inactive" : "")}>
        <span />
      </div>
      <div className="boundary-result">
        <Check size={12} />
        {limited
          ? "Regular videos stay available."
          : "You can change your boundaries anytime."}
      </div>
    </div>
  );
}
const faqs = [
  {
    question: "Does my activity leave my computer?",
    answer:
      "Your activity history is stored locally in your Windows user profile. Core tracking and browser restrictions work without a cloud account. Public lookups and AI features are optional and require your permission.",
  },
  {
    question: "Do I need the browser companion?",
    answer:
      "Windows app tracking works on its own. To observe YouTube Shorts, regular videos and Instagram Reels, or apply website limits, add the included companion to Brave or a supported Chromium browser. Browser recording stays off until you opt in.",
  },
  {
    question: "Can I use it without AI?",
    answer:
      "Yes. Tracking, goals, focus sessions, browser limits and reports work without an AI key. The optional Coach uses your own provider key. Service classification uses saved choices and local rules first; an LLM request needs your approval.",
  },
  {
    question: "What can Digital Discipline restrict?",
    answer:
      "The browser companion can apply domain and route restrictions, schedules and time budgets. Temporary exceptions and emergency recovery remain available. Selected native Windows apps currently receive focus warnings, rather than process blocking.",
  },
  {
    question: "What should I know before installing?",
    answer:
      "Version 0.4.4 is an early-access Windows x64 development build. Its certificate is self-signed, so Windows may show a publisher or SmartScreen warning. Quit an older version from its tray menu before updating. Existing pairing, profiles and recording preferences are retained.",
  },
];
export default function Home() {
  const [menuOpen, setMenuOpen] = useState(false),
    [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!menuOpen) return;
    const handle = (e: KeyboardEvent) => {
      if (e.key === "Escape") setMenuOpen(false);
    };
    window.addEventListener("keydown", handle);
    return () => window.removeEventListener("keydown", handle);
  }, [menuOpen]);
  async function copyChecksum() {
    try {
      await navigator.clipboard.writeText(CHECKSUM);
      setCopied(true);
      setTimeout(() => setCopied(false), 2200);
    } catch {
      setCopied(false);
    }
  }
  return (
    <>
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <header className="site-header">
        <div className="header-inner">
          <a className="wordmark" href="#" aria-label="Digital Discipline home">
            <Mark />
            <span>digital discipline</span>
          </a>
          <nav
            className={"site-nav " + (menuOpen ? "open" : "")}
            aria-label="Main navigation"
          >
            <a href="#features" onClick={() => setMenuOpen(false)}>
              Features
            </a>
            <a href="#privacy" onClick={() => setMenuOpen(false)}>
              Privacy
            </a>
            <a href="#faq" onClick={() => setMenuOpen(false)}>
              FAQs
            </a>
            <a className="mobile-download" href={DOWNLOAD} download>
              Download for Windows
            </a>
          </nav>
          <div className="header-actions">
            <DownloadLink className="header-download">Get the app</DownloadLink>
            <button
              className="menu-toggle"
              aria-expanded={menuOpen}
              aria-label={menuOpen ? "Close navigation" : "Open navigation"}
              onClick={() => setMenuOpen((n) => !n)}
            >
              {menuOpen ? <X size={21} /> : <Menu size={21} />}
            </button>
          </div>
        </div>
      </header>
      <main id="main">
        <section className="hero">
          <div className="hero-copy">
            <a href="#download" className="release-tag">
              <span className="tag-dot" />A LITTLE MORE INTENTION
              <span className="tag-divider" />
              v0.4.4
              <ArrowUpRight size={12} />
            </a>
            <h1>
              Your time.
              <br />
              <span>Back in</span>
              <br />
              your hands<span className="red-stop">.</span>
            </h1>
            <p className="hero-description">
              See where your day goes. Put boundaries around endless feeds. Make
              room for the work—and life—you choose.
            </p>
            <div className="hero-actions">
              <DownloadLink />
              <a href="#features" className="explore-link">
                Take a closer look
                <ArrowRight size={15} />
              </a>
            </div>
            <p className="hero-meta">
              <Monitor size={13} />
              Windows x64
              <span />
              Local first
              <span />
              No account needed
            </p>
          </div>
          <div className="hero-product">
            <div className="product-aura" />
            <AppPreview />
            <PillDemo />
          </div>
          <div className="hero-bottom">
            <span>
              Less autopilot.
              <br />
              <strong>More of your day.</strong>
            </span>
            <a
              href="#features"
              aria-label="Explore Digital Discipline features"
            >
              <ArrowDown size={18} />
            </a>
            <p>
              Made for the moments between
              <br />
              “one more video” and what comes next.
            </p>
          </div>
        </section>
        <section id="features" className="features section-shell">
          <div className="section-heading">
            <div>
              <span className="eyebrow">
                <span />A QUIETER KIND OF CONTROL
              </span>
              <h2>
                Not another thing
                <br />
                to keep track of.
              </h2>
            </div>
            <p>
              A clear picture of your habits.
              <br />A few thoughtful boundaries.
              <br />
              Then, back to your day.
            </p>
          </div>
          <div className="feature-grid">
            <article className="feature-card attention-card">
              <div className="card-index">01 / UNDERSTAND</div>
              <h3>
                Know where
                <br />
                the hours go.
              </h3>
              <p>
                Apps, active time and the little detours. See the patterns that
                are easy to miss while you’re in them.
              </p>
              <div className="activity-graphic">
                <div className="activity-caption">
                  <span>YOUR AFTERNOON</span>
                  <span>Sample activity</span>
                </div>
                <div className="activity-stack">
                  <div>
                    <span className="activity-initial">C</span>
                    <span>
                      <strong>Building something good</strong>
                      <small>Code · Development</small>
                    </span>
                    <span>3h 41m</span>
                  </div>
                  <div>
                    <span className="activity-initial youtube-initial">
                      <Play size={12} fill="currentColor" />
                    </span>
                    <span>
                      <strong>A break, on your terms</strong>
                      <small>Brave · Entertainment</small>
                    </span>
                    <span>1h 12m</span>
                  </div>
                  <div>
                    <span className="activity-initial">F</span>
                    <span>
                      <strong>Making ideas visible</strong>
                      <small>Figma · Design</small>
                    </span>
                    <span>1h 45m</span>
                  </div>
                </div>
                <div className="activity-footer">
                  <i />
                  <span>Recorded locally. Understood by you.</span>
                </div>
              </div>
            </article>
            <article className="feature-card boundary-card">
              <div className="card-index">02 / SET A BOUNDARY</div>
              <h3>
                A little less
                <br />
                “just one more.”
              </h3>
              <p>
                Set website limits and short-feed boundaries. Keep an
                intentional break from becoming the rest of your evening.
              </p>
              <BrowserBoundary />
              <p className="feature-footnote">
                Browser companion required. Try the sample switch.
              </p>
            </article>
          </div>
          <div className="focus-story">
            <div className="focus-story-copy">
              <span className="eyebrow">03 / MAKE SOME SPACE</span>
              <h3>
                One task.
                <br />A little breathing room.
              </h3>
              <p>
                Choose a goal, start a focus session, and take breaks that
                actually feel like breaks. Your desktop keeps track while you
                get on with it.
              </p>
              <a
                href="#product-preview"
                onClick={() =>
                  document
                    .querySelector<HTMLButtonElement>(
                      ".product-sidebar button:nth-child(2)",
                    )
                    ?.click()
                }
              >
                Explore the focus preview
                <ArrowUpRight size={15} />
              </a>
            </div>
            <div
              className="focus-visual"
              aria-label="Illustrative focus session"
            >
              <div className="focus-orbit orbit-one" />
              <div className="focus-orbit orbit-two" />
              <div className="focus-center">
                <Target size={20} />
                <strong>25:00</strong>
                <span>ONE THING AT A TIME</span>
              </div>
              <span className="orbit-label label-work">
                <Code2 size={13} />
                Your next big idea
              </span>
              <span className="orbit-label label-break">
                <Clock3 size={13} />
                Space for a break
              </span>
              <span className="orbit-label label-block">
                <Shield size={13} />
                Fewer interruptions
              </span>
            </div>
          </div>
        </section>
        <section id="privacy" className="privacy-section">
          <div className="privacy-inner section-shell">
            <div>
              <span className="eyebrow">
                <LockKeyhole size={13} />
                PRIVATE BY DESIGN
              </span>
              <h2>
                Your habits.
                <br />
                Your hard drive.
              </h2>
              <p>
                Your activity isn’t a profile to sell.
                <br />
                It’s a little perspective, kept on your PC.
              </p>
              <a className="subtle-link" href="#faq">
                Understand your privacy choices
                <ArrowUpRight size={14} />
              </a>
            </div>
            <div className="privacy-settings">
              <div className="settings-heading">
                <span>
                  <Shield size={17} />
                  Settings &amp; Privacy
                </span>
                <span>YOUR RULES</span>
              </div>
              <div className="privacy-setting">
                <span className="privacy-setting-icon">
                  <Monitor size={18} />
                </span>
                <div>
                  <strong>Activity history</strong>
                  <small>Stored in your Windows user profile</small>
                </div>
                <span className="setting-state">Local</span>
              </div>
              <div className="privacy-setting">
                <span className="privacy-setting-icon">
                  <SlidersHorizontal size={18} />
                </span>
                <div>
                  <strong>Browser recording</strong>
                  <small>Off until you choose to enable it</small>
                </div>
                <span className="setting-state">Your choice</span>
              </div>
              <div className="privacy-setting">
                <span className="privacy-setting-icon">
                  <LockKeyhole size={18} />
                </span>
                <div>
                  <strong>AI features</strong>
                  <small>Optional. Your provider. Your permission.</small>
                </div>
                <span className="setting-state">Opt-in</span>
              </div>
              <div className="privacy-settings-foot">
                <Check size={13} />
                Core features work offline, without an account.
              </div>
            </div>
          </div>
        </section>
        <section id="faq" className="faq-section section-shell">
          <div className="faq-intro">
            <span className="eyebrow">A FEW GOOD QUESTIONS</span>
            <h2>
              A little clarity,
              <br />
              before you begin.
            </h2>
            <p>
              Here’s what to expect from
              <br />
              Digital Discipline today.
            </p>
          </div>
          <div className="faq-list">
            {faqs.map((faq, i) => (
              <details
                key={faq.question}
                id={i === 4 ? "install-notes" : undefined}
              >
                <summary>
                  {faq.question}
                  <Plus size={17} />
                </summary>
                <p>{faq.answer}</p>
              </details>
            ))}
          </div>
        </section>
        <section id="download" className="download-section section-shell">
          <div className="download-overline">
            <Mark />
            <span>THE NEXT HOUR IS YOURS</span>
          </div>
          <h2>
            Make room for
            <br />
            <span>what matters.</span>
          </h2>
          <p>Start with a clearer picture of your day.</p>
          <DownloadLink />
          <div className="download-meta">
            <span>v0.4.4</span>
            <i />
            <span>Windows x64</span>
            <i />
            <span>5.98 MB</span>
          </div>
          <p className="development-note">
            Early access · Development-signed installer.
            <br />
            <a
              href="#install-notes"
              onClick={() => {
                const item = document.getElementById(
                  "install-notes",
                ) as HTMLDetailsElement | null;
                if (item) item.open = true;
              }}
            >
              Read the installation notes
              <ArrowUpRight size={11} />
            </a>
          </p>
          <details className="checksum">
            <summary>
              Verify this download
              <ChevronRight size={12} />
            </summary>
            <div>
              <span>SHA-256</span>
              <code>{CHECKSUM}</code>
              <button onClick={() => void copyChecksum()}>
                {copied ? <Check size={13} /> : <Square size={12} />}{" "}
                {copied ? "Copied" : "Copy checksum"}
              </button>
              <small role="status">
                {copied ? "Checksum copied to clipboard." : ""}
              </small>
            </div>
          </details>
        </section>
      </main>
      <footer className="site-footer section-shell">
        <a
          className="wordmark"
          href="#"
          aria-label="Digital Discipline, back to top"
        >
          <Mark />
          <span>digital discipline</span>
        </a>
        <p>Less noise. More intention.</p>
        <div>
          <a href="#privacy">Privacy</a>
          <a href="#download">Download</a>
          <span>© {new Date().getFullYear()} Digital Discipline</span>
        </div>
      </footer>
    </>
  );
}
