# Digital Discipline for Android

Prototype 0.1.0 runs on Android 10 and later. The GitHub Actions artifact contains a zip-aligned unsigned APK. Sign the same APK with the same key for both test phones and keep that key for future updates.

## Individual

1. Choose **Individual** and select **Use on my phone**.
2. In **Device access**, enable the Digital Discipline app blocker in Android Accessibility. Enable activity tracking and grant Usage access if you want an activity report.
3. Block an app in **App controls** and launch it. The block screen should appear. Return Home, unblock it, open it for a short time and then refresh the Digital Discipline report.

## Parent and child

1. Install the APK on both phones and connect them to the same Wi-Fi or hotspot. Some networks isolate connected devices.
2. On the child phone, choose **Parent & child**, then **Child phone**. Set a six-digit parent PIN. Enable the Accessibility service, grant Usage access for reporting and allow family status notifications. Re-enter the parent PIN after returning from Android Settings.
3. On the child phone, choose **Show pairing code**. On the parent phone, choose **Parent & child**, then **Parent phone**. Scan the QR shown on the child phone or enter its code manually.
4. Block an app on the parent phone. Open it on the child phone and confirm that the block screen appears. Reopen Digital Discipline on the child phone and verify that its management screen requires the PIN. Unblock the app from the parent phone and verify that it opens.
5. Refresh child data on the parent phone. **Last 24 hours** shows estimated foreground duration per app. **Recent app opens** shows up to 120 sessions with start timestamps. To revoke the connection, open child settings with the PIN and tap **Disconnect parent phone**. This preserves the child's app rules.

The family link uses an encrypted connection on the local network and a private pairing code. The child phone displays a family status notification while it accepts connections. The code contains the current child IP address, so open the code again and re-pair on the parent phone if the address changes. There is no cloud service or screen capture in this prototype.

Blocking uses an Android Accessibility overlay after launch. Android Settings, launchers and dialers are exempt. This is an app-level control, so disabling Accessibility, force-stopping, clearing app data or uninstalling can disable it. Some sideloaded installs require enabling **Allow restricted settings** in the Digital Discipline Android App info menu before Android permits the Accessibility service.

## Build locally

Use Java 17, Gradle 8.9 and Android SDK Platform 35. From this directory run:

```sh
gradle :app:assembleRelease :app:lintRelease
```
