# Digital Discipline 0.5.3 for Linux

This branch starts from Windows 0.5.3, including the compact to-do and notes panel. The Linux package targets Ubuntu 24.04 x86-64 (amd64). A desktop session is required. Choose an X11/Xorg session for foreground app tracking, idle detection, the global Ctrl+Shift+D shortcut and selected-app minimization. Availability of tray icons and always-on-top windows depends on the desktop environment.

## Download and install

1. Sign into GitHub and open [Linux builds](https://github.com/motionedits246-max/digital-discipline/actions/workflows/linux-ci.yml?query=branch%3Afeature%2Flinux-desktop).
2. Open the latest successful run and download **Digital-Discipline-Linux-amd64** under **Artifacts**. Extract the ZIP. GitHub keeps these build artifacts for 30 days.
3. In the extracted folder, run:

   ```bash
   sha256sum -c SHA256SUMS-linux.txt
   sudo apt install ./Digital-Discipline-0.5.3-linux-amd64.deb
   ```

4. Open **Digital Discipline** from your app menu or run `digital-discipline-desktop`. It starts its local agent automatically. Settings & Privacy includes optional startup at desktop sign-in.
5. For browser activity, Shorts and Reels tracking, and website restrictions, use **Settings & Privacy → Browser connections**. Load the included unpacked extension in Brave, Chrome, or Edge, approve the connection in Digital Discipline, then enable **Record browser activity** in the extension.

Activity data is stored in `${XDG_DATA_HOME:-$HOME/.local/share}/digital-discipline`. Saved AI keys and encrypted folder sync require an unlocked desktop Secret Service keyring, such as GNOME Keyring. One-time Coach keys can be used without a keyring. For Linux updates, use **Application updates → Open Linux downloads** and install the newer package with APT.

On Wayland, browser telemetry and browser rules still use the paired extension. Compositor-wide foreground app tracking, idle detection, selected-app minimization and the global hotkey require X11 in this build. Export, dashboard, analytics, tasks, notes, focus timers, and backups use the same app and data model on both platforms.

## Build from source

On Ubuntu 24.04, install stable Rust, Node.js 22 with npm, and the Linux Tauri prerequisites.

```bash
sudo apt update
sudo apt install build-essential libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev libdbus-1-dev pkg-config file
git clone --branch feature/linux-desktop https://github.com/motionedits246-max/digital-discipline.git
cd digital-discipline
bash scripts/build-linux.sh
```

The package and checksum are written to `release-output/`. To download the Windows build, use the [Windows 0.5.3 build](https://github.com/motionedits246-max/digital-discipline/actions/runs/43354731670) or the [project site](https://digital-discipline-windows.codecut-official.chatgpt.site).
