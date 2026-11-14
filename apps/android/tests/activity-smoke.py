"""Run against a disposable Android emulator with a signed test APK installed."""
import re
import subprocess
import time
import xml.etree.ElementTree as ET

PACKAGE = "com.digitaldiscipline.android"


def adb(*args):
    return subprocess.check_output(["adb", *args], text=True, timeout=20).strip()


def top_activity():
    activities = adb("shell", "dumpsys", "activity", "activities")
    return " ".join(line for line in activities.splitlines()
                    if "topResumedActivity" in line or "mResumedActivity" in line)


def wait_for(predicate, description, attempts=25):
    for _ in range(attempts):
        if predicate():
            return
        time.sleep(0.4)
    raise AssertionError(f"Timed out waiting for {description}: {top_activity()}")


def has_label(label):
    try:
        adb("shell", "uiautomator", "dump", "/data/local/tmp/dd-window.xml")
        tree = ET.fromstring(adb("exec-out", "cat", "/data/local/tmp/dd-window.xml"))
        return any(node.get("text") == label for node in tree.iter("node"))
    except (ET.ParseError, subprocess.SubprocessError):
        return False


def tap(label, prefix=False, widget=None):
    last_labels = []

    def click_matching():
        try:
            adb("shell", "uiautomator", "dump", "/data/local/tmp/dd-window.xml")
            tree = ET.fromstring(adb("exec-out", "cat", "/data/local/tmp/dd-window.xml"))
        except (ET.ParseError, subprocess.SubprocessError):
            return False
        last_labels.clear()
        for node in tree.iter("node"):
            values = [node.get("text", ""), node.get("content-desc", "")]
            last_labels.extend(value for value in values if value)
            if widget and not node.get("class", "").endswith(widget):
                continue
            if any(value.casefold() == label.casefold() or prefix and value.casefold().startswith(label.casefold())
                   for value in values):
                bounds = list(map(int, re.findall(r"\d+", node.get("bounds", ""))))
                if len(bounds) != 4 or bounds[0] >= bounds[2] or bounds[1] >= bounds[3]:
                    continue
                left, top, right, bottom = bounds
                adb("shell", "input", "tap", str((left + right) // 2), str((top + bottom) // 2))
                return True
        return False

    try:
        wait_for(click_matching, label, attempts=5)
    except AssertionError as error:
        raise AssertionError(f"{error}. Visible labels: {last_labels}") from error


def launch(component):
    adb("shell", "am", "start", "-n", component)


def app_component(package):
    result = adb("shell", "cmd", "package", "resolve-activity", "--brief",
                 "-a", "android.intent.action.MAIN", "-c", "android.intent.category.LAUNCHER", package)
    return next((line for line in reversed(result.splitlines()) if line.startswith(package + "/")), None)


fixture = next(((pkg, activity) for pkg in ("com.android.deskclock", "com.android.calculator2",
                                            "com.android.chrome")
                if (activity := app_component(pkg))), None)
if fixture is None:
    raise AssertionError("No known launchable test app installed")
fixture_package, component = fixture

adb("shell", "input", "keyevent", "KEYCODE_WAKEUP")
adb("shell", "wm", "dismiss-keyguard")
launch(PACKAGE + "/.MainActivity")
tap("Preview with sample data")
tap("Last 7 Days")
tap("Today")
tap("Exit sample preview")
tap("Get started")
tap("Individual", prefix=True)
tap("Focus", widget="LinearLayout")
tap("Focus minutes", widget="EditText")
adb("shell", "input", "keyevent", "KEYCODE_MOVE_END")
adb("shell", "input", "keyevent", "KEYCODE_DEL", "KEYCODE_DEL")
adb("shell", "input", "text", "37")
adb("shell", "input", "keyevent", "KEYCODE_BACK")
tap("Start focus")
# Android's UI dump waits for an idle view hierarchy. Check persistent
# timer state from the static Today shortcut while the countdown runs.
time.sleep(2)
adb("shell", "am", "force-stop", PACKAGE)
launch(PACKAGE + "/.MainActivity")
wait_for(lambda: has_label("Open focus session"), "restored focus shortcut", attempts=5)

# Only the disposable CI device receives this scripted Usage Access grant.
adb("shell", "appops", "set", PACKAGE, "GET_USAGE_STATS", "allow")
launch(component)
time.sleep(3)
assert fixture_package + "/" in top_activity(), top_activity()
launch(PACKAGE + "/.MainActivity")
tap("Activity")
tap("Last 7 Days")
tap("Last 24h")
tap("Today")
tap("Settings")
wait_for(lambda: has_label("Done"), "Usage Access Done", attempts=5)
tap("Open Digital Discipline workspace")
tap("To-do")
tap("Add task")
tap("Task", widget="EditText")
adb("shell", "input", "text", "Test%sworkspace")
adb("shell", "input", "keyevent", "KEYCODE_BACK")
tap("Save")
tap("Test workspace")
tap("Cancel")
tap("Notes")
tap("New note")
tap("Title", widget="EditText")
adb("shell", "input", "text", "Test%snote")
adb("shell", "input", "keyevent", "KEYCODE_BACK")
tap("Save")
tap("Test note")
tap("Cancel")
tap("Close")
adb("shell", "am", "start", "-n", PACKAGE + "/.MainActivity", "-a", "android.intent.action.SEND",
    "-t", "text/plain", "--es", "android.intent.extra.TEXT", "https://www.youtube.com/shorts/AbCdEfG1234")
tap("Save link")
tap("Done")
print("Activity, custom timer restoration, workspace and Shorts sharing checked.", flush=True)
