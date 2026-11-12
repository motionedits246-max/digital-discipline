# Digital Discipline for Android

Prototype 0.3.0 supports Android 10 and later. It uses Android Usage Access for recent foreground app activity, and the explicitly enabled Digital Discipline accessibility service for blocking. The service receives window-change events and checks package names. If Usage Access is also enabled, it can check recent app lifecycle events when a window event is missed. It requests no window content. A block returns the phone to Home and shows a temporary notice. The blocking list shows launcher apps for the current Android profile. Android Settings, the home launcher, the current keyboard, the phone and messaging handlers, and system permission or install screens are protected.

## Try Individual mode

Install the signed APK, choose **Individual**, and grant **Usage access** in Settings. Enable **Digital Discipline app blocking** under Android Accessibility after reading the app’s disclosure. Check that the blocking service is marked **Done**. In **Blocking**, search the installed app list, select an eligible app, open it from the launcher, and check for the block notice. Disable the rule and confirm the app opens again. Open two unblocked apps for a minute each and refresh **Activity**. It shows app-open timestamps and estimated foreground time for the last 24 hours, starting at setup if the device was enrolled more recently.

## Try Parent and Child on two phones

1. Choose **Parent phone** on the parent’s device. Open **Settings → Pairing code → Copy code**. Keep the pairing code private. Copy and Paste use the local clipboard on each phone. Transfer the code privately between devices or enter it by hand.
2. Choose **Child phone** on the child’s device. Paste or enter the 43-character pairing code, then set a separate 6–12 digit parent PIN and allow the visible supervision notification. The code is case-sensitive and accepts line breaks introduced by copying. With the child phone in hand, the parent can grant Usage Access and enable blocking under Android Accessibility.
3. While parent settings are unlocked on the child device, go to **Settings → Export child activity** and save the encrypted .dd file. Transfer it using Android Files or Quick Share. On the parent phone choose **Settings → Import child activity**.
4. On the parent phone open **Blocking**, select an eligible app and export a rules file. Transfer it to the child phone. Unlock **Parent access** with the PIN, then choose **Import parent rules**. Open the selected app to see the temporary block notice. The parent can export and import an empty selection to clear all rules.
5. Confirm an incorrect PIN cannot open settings and that leaving Digital Discipline relocks the child dashboard. Check the supervision notification, then restart the child phone and check that it returns. Try importing the same or an older rules file to confirm it cannot roll back the rules.

Files are authenticated with the pairing key. The prototype pairs a single child and exchanges files manually, with no server. Android retains the underlying usage events for a short period. Split-screen and picture-in-picture durations are estimates. The activity timeline shows up to 2,000 recent app opens. Standard Android controls can revoke permissions or uninstall the app. Android may require **Allow restricted settings** in the installed app’s info screen before its accessibility service can be enabled on a sideloaded device. The app does not collect screenshots.

## Build and sign

Use JDK 17, Gradle 8.11.1, Android SDK Platform 35 and Build Tools 35.0.0. From the repository root:

```sh
gradle -p apps/android :app:lintRelease :app:assembleRelease
```

The unsigned APK is in `apps/android/app/build/outputs/apk/release/`. GitHub Actions uploads an aligned unsigned APK and its SHA-256 checksum in one artifact. Sign the APK with your own test key and verify it before installing. Use the same key for updates.

```sh
apksigner sign --ks YOUR_TEST_KEYSTORE --out Digital-Discipline-Android-0.3.0.apk Digital-Discipline-Android-0.3.0-unsigned.apk
apksigner verify --verbose Digital-Discipline-Android-0.3.0.apk
```
