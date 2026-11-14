#!/usr/bin/env bash
set -euo pipefail
root=$(mktemp -d)
export XDG_DATA_HOME="$root/data" XDG_CONFIG_HOME="$root/config"
export XDG_SESSION_TYPE=x11 GDK_BACKEND=x11
unset WAYLAND_DISPLAY
desktop=
wm=
window=
cleanup() {
  [[ -z "$window" ]] || kill "$window" 2>/dev/null || true
  if [[ -f "$XDG_DATA_HOME/digital-discipline/agent.token" ]]; then
    curl --silent --max-time 3 --request POST --header "Authorization: Bearer $(cat "$XDG_DATA_HOME/digital-discipline/agent.token")" http://127.0.0.1:48761/shutdown >/dev/null || true
  fi
  [[ -z "$desktop" ]] || kill "$desktop" 2>/dev/null || true
  [[ -z "$wm" ]] || kill "$wm" 2>/dev/null || true
  rm -rf "$root"
}
trap cleanup EXIT
openbox >"$root/wm.log" 2>&1 &
wm=$!
digital-discipline-desktop >"$root/desktop.log" 2>&1 &
desktop=$!
api() {
  curl --fail --silent --max-time 3 --header "Authorization: Bearer $(cat "$XDG_DATA_HOME/digital-discipline/agent.token")" "$@"
}
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
if [[ "$ready" != true ]]; then
  cat "$root/desktop.log" >&2
  exit 1
fi
[[ "$(stat -c %a "$XDG_DATA_HOME/digital-discipline")" == 700 ]]
timeout 10 xdotool search --sync --onlyvisible --name '^Digital Discipline$' >/dev/null
timeout 5 digital-discipline-desktop
xterm -title DigitalDisciplineTrackingTest >/dev/null 2>&1 &
window=$!
target=$(timeout 10 xdotool search --sync --onlyvisible --name '^DigitalDisciplineTrackingTest$' | head -n 1)
xdotool windowactivate --sync "$target"
for _ in {1..12}; do
  if api http://127.0.0.1:48761/live | python3 -c 'import json,sys; data=json.load(sys.stdin); assert data["platform_supported"] and any(s["app"] == "xterm" and not s["idle"] for s in data["sessions"])' 2>/dev/null; then
    exit 0
  fi
  sleep 1
done
cat "$root/desktop.log" >&2
api http://127.0.0.1:48761/live >&2
exit 1
