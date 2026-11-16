# Digital Discipline

**Understand your screen time. Take back your attention.**

Digital Discipline is an open-source, local-first productivity and digital wellbeing project. It helps you see where your time goes, recognize distracting habits, build better routines, and stay focused without turning productivity into another distraction.

The primary application on this branch is built for **Windows**, with an optional browser companion for supported Chromium-based browsers.

[Explore the code](#project-structure) · [Get started](#getting-started) · [Privacy](#privacy-first-by-design) · [Contribute](#contributing)

---

## Why Digital Discipline?

A quick video turns into half an hour. A short break becomes a browsing session. At the end of the day, it's hard to remember what actually took your time.

Most screen-time tools show a total. Digital Discipline aims to give that number context — **what you were doing, when you got distracted, and what you can change**.

The goal isn't to eliminate entertainment or track every second obsessively. It's to make screen time intentional, with insights and controls you choose.

## What you can do

| Area | What it helps with |
| --- | --- |
| **Understand your day** | See foreground app usage, idle periods, activity timelines, categories, and longer-term patterns. |
| **Recognize scrolling habits** | Observe supported YouTube and Instagram activity with the optional, opt-in browser companion. |
| **Reduce distractions** | Set browser site/path restrictions, usage limits, schedules, focus sessions, and reversible exceptions. |
| **Keep work in view** | Use the floating pill and quick panel to check progress without switching to a full dashboard. |
| **Plan the next step** | Keep simple to-dos and notes close at hand; connect tasks with focus routines. |
| **Reflect and improve** | Review goals, activity history, trends, and optional AI-assisted insights. |
| **Stay in control** | Pause tracking, configure exclusions, manage consent, inspect diagnostics, and export or delete local data. |

### A workspace that stays out of the way

Digital Discipline is designed around a small floating **pill** that can expand into a quick panel or a full workspace. Open the detailed dashboard when you need it; keep the lightweight view nearby when you don't.

- **Pill:** glanceable status and progress.
- **Quick panel:** activity overview, to-do checklist, and saved notes.
- **Workspace:** timelines, analytics, focus settings, browser controls, and privacy preferences.
- **Background agent:** continues collecting permitted activity while the main window is closed, until you explicitly stop it.

### Focus without permanent lockouts

Focus tools help you shape your browsing environment, rather than locking you out of your own computer. You can define restrictions and schedules for selected sites or content paths, and use temporary exceptions or emergency recovery when needed.

Browser rules are **not** an OS-wide, tamper-proof app lock. The project is a self-directed productivity tool, not surveillance software.

## Everyday use cases

**Studying:** Start a focus session, keep your study tasks visible, and reduce access to selected distracting pages while you work.

**Coding or creative work:** See time spent across desktop applications, classify activities in a way that makes sense for your workflow, and review interruptions afterward.

**Reducing short-form scrolling:** Opt into browser activity recording to learn when Shorts or Reels become a habit. Observations are based on supported page and visible-playback signals; they are estimates, not an official or exhaustive platform watch history.

**Building consistency:** Review your activity heatmap and trends, set realistic goals, and adjust your routines over time.

## How it works

```text
Windows applications ──> Rust background agent ──> Local activity storage
                                  │
Optional browser companion ───────┤
                                  │
                                  ▼
                         Tauri desktop app
                    Pill • Quick panel • Dashboard
                           Goals • Focus • Insights
```

1. **Observe:** With tracking enabled, the background agent records supported foreground-app and idle-time activity. The browser companion collects only the browser signals you've opted into.
2. **Understand:** Local data powers the timeline, categories, visualizations, usage summaries, and personal progress.
3. **Act:** Choose focus rules, adjust goals, manage tasks, or review insights. Core functionality doesn't require a cloud account.

AI-assisted features are optional and require their own setup and consent. They are not necessary for everyday tracking or focus controls.

## Privacy-first by design

Digital Discipline is built around **local ownership and explicit choices**.

- **Local-first history:** Activity is stored on your Windows device, rather than requiring continuous cloud synchronization.
- **Optional browser recording:** Browser observations have a separate opt-in control.
- **No default content capture:** The core collector does not record keystrokes, clipboard contents, private chats, or continuous screen recordings.
- **User-controlled history:** Manage exclusions, retention, exports, and deletion from the app.
- **Separate AI controls:** External-provider requests are optional. Review what is shared and your provider's policies before enabling them.
- **Clear limits:** Local API authentication is not a defense against malicious software already running as the same Windows user. Local activity storage should not be assumed to be fully encrypted.

For implementation details and known limitations, consult the [project documentation](docs/) and [implementation ledger](docs/IMPLEMENTATION-LEDGER.md).

## Getting started

### Install on Windows

Visit [GitHub Releases](https://github.com/motionedits246-max/digital-discipline/releases) for published downloads, or check the [Windows build workflow](https://github.com/motionedits246-max/digital-discipline/actions/workflows/windows-ci.yml) for available test artifacts.

Download an installer from a trusted successful build and follow its instructions. Development builds may be self-signed rather than signed by a publicly trusted Windows publisher.

### Build from source

You'll need Windows, Rust with the MSVC toolchain, Visual Studio C++ Build Tools/Windows SDK, Node.js, Python, and WebView2.

```powershell
git clone https://github.com/motionedits246-max/digital-discipline.git
cd digital-discipline
.\scripts\build-windows.ps1
```

For development, use `.\scripts\dev-windows.ps1`. The Windows build script handles tests and packaging; successful artifacts are placed in `release-output/`.

### Connect the browser companion (optional)

Open **Settings & Privacy → Browser setup** in the desktop app. Follow the guided instructions to load the unpacked companion in Brave and authorize its connection. Enable browser activity recording only if you want those insights; browser restriction settings are separate.

Keep your existing extension installation when upgrading if you need to preserve its local data and extension identity.

## Technology

- **Rust:** background activity collection, policies, and local services.
- **Tauri + React + TypeScript:** lightweight Windows interface.
- **SQLite:** on-device activity history.
- **Manifest V3 browser extension:** optional browser observations and restrictions.
- **GitHub Actions:** automated Windows tests and packaging.

## Project structure

| Directory | Purpose |
| --- | --- |
| [`agent/`](agent/) | Rust background agent and local services |
| [`apps/desktop/`](apps/desktop/) | Windows desktop interface |
| [`extension/`](extension/) | Browser companion |
| [`scripts/`](scripts/) | Development, testing, and build tools |
| [`tests/`](tests/) | Automated checks |
| [`docs/`](docs/) | Architecture, blueprint, design decisions, and technical documentation |
| [`.github/workflows/`](.github/workflows/) | Continuous integration and Windows packaging |

The **`main` branch represents the Windows desktop project**. Android and Linux experiments live on separate branches; their features and installation steps should not be assumed to apply to this branch.

## Contributing

Contributions are welcome, especially thoughtful bug reports, usability improvements, accessibility fixes, performance work, and browser compatibility testing.

Before opening a pull request, describe the problem, explain the change, and include reproduction or testing notes where possible. Please avoid committing API keys, personal activity records, or other sensitive data.

- [Report an issue or suggest an improvement](https://github.com/motionedits246-max/digital-discipline/issues)
- [Browse the source](https://github.com/motionedits246-max/digital-discipline)
- [Read the original project blueprint](docs/blueprint/README.md)

---

**Your time is yours. Digital Discipline helps you see it, understand it, and use it on purpose.**

If the project is useful to you, consider giving it a **star** on GitHub.
