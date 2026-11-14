# Digital Discipline for Android

Version 0.7.2 is the next Individual test build for Android 10 and later. It uses optional Android Usage Access to reconstruct foreground app sessions. App blocking, its Accessibility service and family rule exchange have been removed. Existing installs can update in place when the APK is signed with the same key.

## What to test

- **Preview** on the welcome screen opens sample Activity, app detail and Focus without granting Usage Access. Sample records stay out of saved history and reports.
- **Activity** has Today, Last 7 Days and a separate rolling Last 24h view. Tap an hourly bar for its app breakdown, a heatmap row for that day, or any part of a top-app row for the session timeline and observed openings.
- **Focus** starts from Today or from a saved duration in Focus. Add an optional task, then pause, resume, end or restart the session. Force-stop and reopen the app to check timer restoration.
- **Settings** shows Done after Android grants Usage Access. Pause or resume collection, pick private apps, change retention or delete activity. Individual tracking, local storage and timers work offline. The app imports recent events when opened or refreshed. Android may discard older source events between visits, and foreground duration is estimated for split-screen and picture-in-picture.
- **Family activity** keeps the visible supervision notice and existing encrypted manual transfer. Pair through the code in Settings, export the child’s activity from PIN-protected settings and import it on the parent phone. New files contain app totals and coarse activity windows. Parent and Child roadmap features that rely on rule delivery remain pending.

The release covers core measurement, privacy and timer work from R1. R2, R3 and Lab features are staged in the roadmap.


## New in 0.7.2

- Drag the floating workspace pill to a comfortable position. Tapping it opens the compact Focus, To-do and Notes panel over the current app.
- Choose and preview a completion sound in Settings. The timer sound loops until stopped from the workspace, Focus, or the notification. Enable Alarms & reminders for on-time background alarms and notifications to use the notification Stop action.
- Share or paste a Shorts or Reels permalink and confirm Save link. Activity refreshes immediately. The counts represent saved links.

## New in 0.7.0

- Enter 1–720 minutes in Focus. Work and break cycles support up to 12 rounds, named presets and an explicit start for each next round. Focus history is stored locally.
- Tap the small mark near the top to open Workspace with Focus, To-do and Notes. In Individual Settings, enable the optional floating button after granting Android display-over-other-apps access and notifications. Hide it with the notification or Settings. Android may hide overlays on system or security-sensitive screens.
- Share a YouTube Shorts or Instagram Reels permalink from its app and explicitly save it. Activity counts saved shares by period. Usage Access separately reports app foreground time and does not supply per-video watch events. Other URLs can be saved as study links.
- Export a JSON activity summary through Android's file picker. Long-press the launcher icon for Today, Focus and Workspace. Add Focus from the system Quick Settings tile editor.

Focus hourglass photograph by Aron Visuals on Unsplash.

## Build and sign

Use JDK 17, Gradle 8.11.1, Android SDK Platform 35 and Build Tools 35.0.0. From the repository root:

```sh
gradle -p apps/android :app:lintRelease :app:assembleRelease
```

GitHub Actions uploads the aligned unsigned APK and its SHA-256 checksum. Sign and verify with the existing test key before installing an update.
