#!/usr/bin/env bash
set -euo pipefail
root=$(mktemp -d)
export XDG_DATA_HOME="$root/data" XDG_CONFIG_HOME="$root/config"
export XDG_SESSION_TYPE=x11 GDK_BACKEND=x11
unset WAYLAND_DISPLAY
desktop=
wm=
window=

api() {
  curl --fail --silent --max-time 3 --header "Authorization: Bearer $(cat "$XDG_DATA_HOME/digital-discipline/agent.token")" "$@"
}

cleanup() {
  local result=$?
  if (( result != 0 )); then
    echo "X11 smoke test failed (exit $result)" >&2
    for log in wm desktop xterm; do
      if [[ -f "$root/$log.log" ]]; then
        echo "--- $log ---" >&2
        cat "$root/$log.log" >&2
      fi
    done
    timeout 3 xwininfo -root -tree >&2 || true
    timeout 3 xprop -root _NET_ACTIVE_WINDOW _NET_CLIENT_LIST >&2 || true
  fi
  [[ -z "$window" ]] || kill "$window" 2>/dev/null || true
  if [[ -f "$XDG_DATA_HOME/digital-discipline/agent.token" ]]; then
    api --request POST http://127.0.0.1:48761/shutdown >/dev/null || true
  fi
  [[ -z "$desktop" ]] || kill "$desktop" 2>/dev/null || true
  [[ -z "$wm" ]] || kill "$wm" 2>/dev/null || true
  rm -rf "$root"
}
trap cleanup EXIT

wait_for_window() {
  local pid=$1
  local ids
  for _ in {1..25}; do
    if ! kill -0 "$pid" 2>/dev/null; then return 1; fi
    if ids=$(xdotool search --onlyvisible --pid "$pid") && [[ -n "$ids" ]]; then
      echo "${ids%%$'\n'*}"
      return 0
    fi
    sleep 1
  done
  return 1
}

openbox >"$root/wm.log" 2>&1 &
wm=$!
for _ in {1..10}; do
  if xprop -root _NET_SUPPORTING_WM_CHECK | grep -q 'window id # 0x'; then
    break
  fi
  sleep 1
done
xprop -root _NET_SUPPORTING_WM_CHECK | grep -q 'window id # 0x'

echo "Launching the installed desktop and waiting for its agent"
digital-discipline-desktop >"$root/desktop.log" 2>&1 &
desktop=$!
ready=false
for _ in {1..45}; do
  if [[ -f "$XDG_DATA_HOME/digital-discipline/agent.token" ]] &&
     api http://127.0.0.1:48761/health | python3 -c 'import json,sys; assert json.load(sys.stdin)["ok"]' 2>/dev/null; then
    ready=true
    break
  fi
  if ! kill -0 "$desktop" 2>/dev/null; then break; fi
  sleep 1
done
[[ "$ready" == true ]]
[[ "$(stat -c %a "$XDG_DATA_HOME/digital-discipline")" == 700 ]]

echo "Waiting for the desktop window"
main_window=$(wait_for_window "$desktop")
xdotool getwindowgeometry "$main_window"
echo "Checking second-launch activation"
timeout 10 digital-discipline-desktop

echo "Launching a real foreground window"
xterm -title DigitalDisciplineTrackingTest -e sleep 60 >"$root/xterm.log" 2>&1 &
window=$!
target=$(wait_for_window "$window")
timeout 10 xdotool windowactivate --sync "$target"
xprop -id "$target" _NET_WM_PID
echo "Checking that the agent records the X11 app"
for _ in {1..12}; do
  if api http://127.0.0.1:48761/live | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["platform_supported"] and any(s["app"] == "xterm" and not s["idle"] for s in data["sessions"])' 2>/dev/null; then
    echo "Installed desktop, agent and X11 foreground tracking verified"
    exit 0
  fi
  sleep 1
done
api http://127.0.0.1:48761/live >&2
exit 1
