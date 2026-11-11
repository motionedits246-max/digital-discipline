"use client";

import { useEffect, useRef, useState, type CSSProperties } from "react";
import {
  Activity,
  ArrowUpRight,
  Bell,
  Check,
  ChevronRight,
  Clock3,
  Code2,
  HardDrive,
  LayoutGrid,
  LockKeyhole,
  Pause,
  Play,
  RotateCcw,
  Shield,
  Target,
  Video,
} from "lucide-react";

const scenes = [
  {
    id: "track",
    title: "Track your day",
    hint: "Apps become a timeline",
    icon: Activity,
    duration: 8600,
  },
  {
    id: "limit",
    title: "Set a Shorts limit",
    hint: "Keep regular videos available",
    icon: Shield,
    duration: 8800,
  },
  {
    id: "focus",
    title: "Start a focus block",
    hint: "Work with a clear end time",
    icon: Target,
    duration: 9200,
  },
] as const;

const sampleApps = [
  {
    name: "Code",
    category: "Development",
    icon: Code2,
    time: "32 min",
    width: 49,
  },
  {
    name: "Brave",
    category: "Entertainment",
    icon: Video,
    time: "12 min",
    width: 18,
  },
  {
    name: "Figma",
    category: "Design",
    icon: LayoutGrid,
    time: "21 min",
    width: 33,
  },
] as const;

type Playback = {
  index: number;
  elapsed: number;
  tour: boolean;
  finished: boolean;
};
const start: Playback = { index: 0, elapsed: 0, tour: true, finished: false };

function WindowBar({ children }: { children: React.ReactNode }) {
  return (
    <div className="scene-window-bar">
      <span className="scene-window-dots" aria-hidden="true">
        <i />
        <i />
        <i />
      </span>
      {children}
    </div>
  );
}

function TrackScene({ elapsed }: { elapsed: number }) {
  const activeApp = elapsed < 2350 ? 0 : elapsed < 4900 ? 1 : 2;
  const savedCount =
    elapsed < 1150 ? 0 : elapsed < 3650 ? 1 : elapsed < 6350 ? 2 : 3;
  const app = sampleApps[activeApp];

  return (
    <div className="scene-art track-art" aria-hidden="true">
      <div className="track-workspace">
        <div className="scene-window track-window">
          <WindowBar>
            <app.icon size={13} />
            {app.name}
          </WindowBar>
          <div className="app-artwork" key={app.name}>
            {activeApp === 0 && (
              <div className="editor-art">
                <div className="editor-sidebar">
                  <i />
                  <i />
                  <i />
                  <i />
                  <i />
                </div>
                <div className="editor-lines">
                  <i />
                  <i />
                  <i />
                  <i />
                  <i />
                  <i />
                  <i />
                </div>
              </div>
            )}
            {activeApp === 1 && (
              <div className="watch-art">
                <div className="watch-player">
                  <Play size={24} fill="currentColor" />
                </div>
                <div className="watch-lines">
                  <i />
                  <i />
                  <i />
                </div>
              </div>
            )}
            {activeApp === 2 && (
              <div className="design-art">
                <i className="design-circle" />
                <i className="design-outline" />
                <i className="design-cursor" />
              </div>
            )}
          </div>
        </div>
        <div className="track-connection">
          <i key={app.name} />
        </div>
        <div className="scene-window observation-card">
          <div className="scene-label">
            <Activity size={13} /> Tracking activity
          </div>
          <div className="observation-app" key={app.name}>
            <span
              className={
                "scene-icon " + (activeApp === 1 ? "scene-icon-red" : "")
              }
            >
              <app.icon size={21} />
            </span>
            <div>
              <strong>{app.name}</strong>
              <small>{app.category}</small>
            </div>
          </div>
          <div className="observation-local">
            <HardDrive size={12} /> Stored on this PC
          </div>
        </div>
      </div>
      <div className="scene-window capture-timeline">
        <div className="capture-header">
          <span>
            <Clock3 size={13} /> Activity timeline
          </span>
          <span>Sample afternoon</span>
        </div>
        <div className="capture-bar">
          {sampleApps.map((item, index) => (
            <div
              className={`capture-segment capture-${index}`}
              key={item.name}
              style={{ flexBasis: `${item.width}%` }}
            >
              <span data-recorded={index < savedCount} />
            </div>
          ))}
        </div>
        <div className="capture-legend">
          {sampleApps.map((item, index) => (
            <span key={item.name} data-recorded={index < savedCount}>
              <i className={`capture-${index}`} />
              {item.name}
              <b>{item.time}</b>
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}

function LimitScene({
  elapsed,
  enabled,
  onToggle,
}: {
  elapsed: number;
  enabled: boolean;
  onToggle: () => void;
}) {
  const used =
    11 + Math.min(4, Math.max(0, Math.floor((elapsed - 1000) / 1050)));
  const blocked = enabled && used === 15;
  const reel = Math.min(2, Math.floor(elapsed / 1800));

  return (
    <div className="scene-art limit-art" data-blocked={blocked}>
      <div className="scene-window shorts-window" aria-hidden="true">
        <WindowBar>
          <LockKeyhole size={10} />
          <span>youtube.com/shorts</span>
        </WindowBar>
        <div className="shorts-player">
          <div className="shorts-reel" data-reel={reel}>
            {[0, 1, 2].map((index) => (
              <div key={index} className={`shorts-frame shorts-frame-${index}`}>
                <span className="shorts-shape" />
                <span className="shorts-shape" />
                <span className="shorts-shape" />
              </div>
            ))}
          </div>
          <div className="shorts-hud">
            <Play size={19} fill="currentColor" />
            <div>
              <i />
              <i />
            </div>
          </div>
          <div className="shorts-paused">
            <span className="scene-icon scene-icon-red">
              <LockKeyhole size={22} />
            </span>
            <strong>Shorts paused</strong>
            <small>Daily limit reached</small>
          </div>
        </div>
      </div>
      <div className="limit-panels">
        <div className="scene-window limit-settings">
          <div className="scene-label">
            <Shield size={14} /> Browser limits
          </div>
          <div className="limit-rule">
            <span className="scene-icon scene-icon-red">
              <Video size={17} />
            </span>
            <div>
              <strong>YouTube Shorts</strong>
              <small>15 minutes a day</small>
            </div>
            <button
              type="button"
              className="demo-switch"
              role="switch"
              aria-checked={enabled}
              aria-label="Toggle sample Shorts restriction"
              onClick={onToggle}
            >
              <span />
            </button>
          </div>
          <div className="limit-meter-label">
            <span>
              {enabled ? `${used} of 15 min used` : "Sample limit off"}
            </span>
            <strong>
              {blocked
                ? "Limit reached"
                : enabled
                  ? `${15 - used}m left`
                  : "Off"}
            </strong>
          </div>
          <div className="limit-meter" aria-hidden="true">
            <span style={{ transform: `scaleX(${enabled ? used / 15 : 0})` }} />
          </div>
        </div>
        <div className="scene-window regular-videos" aria-hidden="true">
          <span className="scene-icon">
            <Play size={15} />
          </span>
          <div>
            <strong>Regular videos</strong>
            <small>Still available</small>
          </div>
          <Check size={15} />
        </div>
      </div>
    </div>
  );
}

function FocusScene({ elapsed }: { elapsed: number }) {
  const progress = Math.min(1, Math.max(0, (elapsed - 1400) / 5450));
  const remaining = Math.ceil(25 * 60 * (1 - progress));
  const complete = progress >= 1;
  const timer = `${Math.floor(remaining / 60)
    .toString()
    .padStart(2, "0")}:${(remaining % 60).toString().padStart(2, "0")}`;

  return (
    <div
      className="scene-art focus-art"
      data-complete={complete}
      aria-hidden="true"
    >
      <div className="focus-dial-area">
        <div className="focus-dial-orbit" />
        <div
          className="focus-dial-ring"
          style={{ "--focus-progress": `${progress * 100}%` } as CSSProperties}
        >
          <div className="focus-dial-face">
            {complete ? <Check size={23} /> : <Target size={20} />}
            <strong>{complete ? "25 min" : timer}</strong>
            <span>{complete ? "Block complete" : "Sample countdown"}</span>
          </div>
        </div>
      </div>
      <div className="focus-panels">
        <div className="scene-window focus-task-card">
          <span className="scene-icon">
            <Target size={19} />
          </span>
          <div>
            <small>Current goal</small>
            <strong>Finish the first draft</strong>
          </div>
        </div>
        <div className="scene-window focus-rule-card">
          <span className="scene-icon scene-icon-red">
            <Shield size={17} />
          </span>
          <div>
            <strong>Shorts on hold</strong>
            <small>During this focus block</small>
          </div>
        </div>
        <div className="scene-window focus-log-card">
          <span className="scene-icon">
            <Bell size={17} />
          </span>
          <div>
            <strong>Focus timebox ended</strong>
            <small>25-minute session saved locally</small>
          </div>
        </div>
      </div>
    </div>
  );
}

function getCaption(index: number, elapsed: number, enabled: boolean) {
  if (index === 0) {
    if (elapsed >= 6350)
      return "Each app becomes a labeled block on your activity timeline.";
    if (elapsed >= 2350)
      return "Switch apps and the floating tracker follows what’s in front.";
    return "Digital Discipline records time in the app you’re using.";
  }
  if (index === 1) {
    if (!enabled)
      return "The sample limit is off. Turn it on to try the restriction.";
    if (elapsed >= 5200)
      return "The budget runs out. Shorts pauses and regular videos stay available.";
    return "Give YouTube Shorts a 15-minute daily budget with the browser companion.";
  }
  if (elapsed >= 6850)
    return "A local notification marks the end of the focus session.";
  return "Set a focus goal and apply browser rules for the session. The sample timer runs in fast-forward.";
}

export function MotionTour({ onOpenFocus }: { onOpenFocus: () => void }) {
  const stage = useRef<HTMLDivElement>(null);
  const [playback, setPlayback] = useState(start);
  const [paused, setPaused] = useState(false);
  const [limitEnabled, setLimitEnabled] = useState(true);
  const [inView, setInView] = useState(false);
  const [pageVisible, setPageVisible] = useState(true);
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    const syncMotion = () => setReducedMotion(preference.matches);
    const syncVisibility = () => setPageVisible(!document.hidden);
    syncMotion();
    syncVisibility();
    preference.addEventListener("change", syncMotion);
    document.addEventListener("visibilitychange", syncVisibility);

    const observer = new IntersectionObserver(
      ([entry]) => setInView(entry.intersectionRatio >= 0.35),
      { threshold: [0, 0.35] },
    );
    if (stage.current) observer.observe(stage.current);
    return () => {
      observer.disconnect();
      preference.removeEventListener("change", syncMotion);
      document.removeEventListener("visibilitychange", syncVisibility);
    };
  }, []);

  const playing =
    inView && pageVisible && !reducedMotion && !paused && !playback.finished;

  useEffect(() => {
    if (!playing) return;
    let previous = performance.now();
    const interval = window.setInterval(() => {
      const now = performance.now();
      const delta = Math.min(now - previous, 240);
      previous = now;
      setPlayback((current) => {
        if (current.finished) return current;
        const elapsed = current.elapsed + delta;
        const duration = scenes[current.index].duration;
        if (elapsed < duration) return { ...current, elapsed };
        if (current.tour && current.index < scenes.length - 1)
          return { ...current, index: current.index + 1, elapsed: 0 };
        return { ...current, elapsed: duration, finished: true };
      });
    }, 100);
    return () => window.clearInterval(interval);
  }, [playing]);

  const scene = scenes[playback.index];
  const elapsed = reducedMotion ? scene.duration : playback.elapsed;

  function selectScene(index: number) {
    setPlayback({ index, elapsed: 0, tour: false, finished: false });
    setPaused(false);
  }

  function replayTour() {
    setPlayback(start);
    setPaused(false);
    setLimitEnabled(true);
  }

  function togglePlayback() {
    if (playback.finished) {
      if (playback.tour) replayTour();
      else selectScene(playback.index);
    } else {
      setPaused((value) => !value);
    }
  }

  const playbackLabel = playback.finished
    ? "Replay"
    : paused
      ? "Play"
      : "Pause";
  const PlaybackIcon = playback.finished ? RotateCcw : paused ? Play : Pause;

  return (
    <section
      id="features"
      className="features section-shell"
      aria-labelledby="demo-heading"
    >
      <div className="section-heading tour-heading">
        <h2 id="demo-heading">See it in action.</h2>
        <p>Follow a sample session or pick a scene.</p>
      </div>
      <div className="motion-tour">
        <div className="tour-rail">
          <div
            className="tour-steps"
            role="group"
            aria-label="Product demo scenes"
          >
            {scenes.map((item, index) => (
              <button
                type="button"
                key={item.id}
                className="tour-step"
                aria-pressed={playback.index === index}
                aria-controls="tour-stage"
                onClick={() => selectScene(index)}
              >
                <span className="tour-step-icon">
                  <item.icon size={18} />
                </span>
                <span className="tour-step-copy">
                  <strong>{item.title}</strong>
                  <small>{item.hint}</small>
                </span>
                <ChevronRight className="tour-step-arrow" size={15} />
              </button>
            ))}
          </div>
          <p className="tour-sample-note">
            <HardDrive size={15} /> Sample activity. This demo doesn’t track
            your device.
          </p>
        </div>
        <div
          className="tour-stage"
          id="tour-stage"
          ref={stage}
          data-playing={playing}
          aria-label={`${scene.title} preview`}
        >
          <div className="tour-toolbar">
            <span className="tour-workspace-label">
              <i /> Product demo
            </span>
            <div className="tour-controls">
              <button
                type="button"
                className="tour-playback"
                onClick={togglePlayback}
                aria-label={`${playbackLabel} product demo`}
              >
                <PlaybackIcon size={14} />
                {playbackLabel}
              </button>
              <button
                type="button"
                className="tour-restart"
                onClick={replayTour}
                aria-label="Restart all demo scenes"
                title="Restart all scenes"
              >
                <RotateCcw size={14} />
              </button>
            </div>
          </div>
          <div
            className="tour-canvas"
            data-scene={scene.id}
            onFocusCapture={() =>
              setPlayback((current) =>
                current.tour ? { ...current, tour: false } : current,
              )
            }
          >
            <div className="scene-enter" key={scene.id}>
              {scene.id === "track" && <TrackScene elapsed={elapsed} />}
              {scene.id === "limit" && (
                <LimitScene
                  elapsed={elapsed}
                  enabled={limitEnabled}
                  onToggle={() => setLimitEnabled((value) => !value)}
                />
              )}
              {scene.id === "focus" && <FocusScene elapsed={elapsed} />}
            </div>
          </div>
          <div className="tour-caption">
            <span className="tour-caption-icon">
              <scene.icon size={15} />
            </span>
            <p>{getCaption(playback.index, elapsed, limitEnabled)}</p>
            {scene.id === "focus" && (
              <a
                className="tour-focus-link"
                href="#product-preview"
                onClick={onOpenFocus}
              >
                Try the timer <ArrowUpRight size={13} />
              </a>
            )}
          </div>
          <div className="tour-progress" aria-hidden="true">
            <span
              style={{ transform: `scaleX(${elapsed / scene.duration})` }}
            />
          </div>
        </div>
      </div>
    </section>
  );
}
