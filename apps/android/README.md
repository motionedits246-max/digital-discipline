# Digital Discipline for Android

Version 0.6.0 is the next Individual test build for Android 10 and later. It uses optional Android Usage Access to reconstruct foreground app sessions. App blocking, its Accessibility service and family rule exchange have been removed. Existing installs can update in place when the APK is signed with the same key.

## What to test

- **Preview** on the welcome screen opens sample Activity, app detail and Focus without granting Usage Access. Sample records stay out of saved history and reports.
- **Activity** has Today, Last 7 Days and a separate rolling Last 24h view. Tap an hourly bar for its app breakdown, a heatmap row for that day, or any part of a top-app row for the session timeline and observed openings.
- **Focus** starts from Today or from a saved duration in Focus. Add an optional task, then pause, resume, end or restart the session. Force-stop and reopen the app to check timer restoration.
- **Settings** shows Done after Android grants Usage Access. Pause or resume collection, pick private apps, change retention or delete activity. Individual tracking, local storage and timers work offline. The app imports recent events when opened or refreshed. Android may discard older source events between visits, and foreground duration is estimated for split-screen and picture-in-picture.
- **Family activity** keeps the visible supervision notice and existing encrypted manual transfer. Pair through the code in Settings, export the child’s activity from PIN-protected settings and import it on the parent phone. New files contain app totals and coarse activity windows. Parent and Child roadmap features that rely on rule delivery remain pending.

The release covers core measurement, privacy and timer work from R1. R2, R3 and Lab features are staged in the roadmap.

The Focus and Settings notebook photographs are by Kelly Sikkema on Unsplash. See [Focus photo](https://unsplash.com/photos/blank-notebook-with-a-pen-on-a-dark-surface-XH0VCNKhEkM) and [Settings photo](https://unsplash.com/photos/an-open-notebook-and-pens-on-a-desk-hBdaqrr5Z3k).

## Build and sign

Use JDK 17, Gradle 8.11.1, Android SDK Platform 35 and Build Tools 35.0.0. From the repository root:

```sh
gradle -p apps/android :app:lintRelease :app:assembleRelease
```

GitHub Actions uploads the aligned unsigned APK and its SHA-256 checksum. Sign and verify with the existing test key before installing an update.
