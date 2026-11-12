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


def tap(label, prefix=False):
    def click_matching():
        try:
            adb("shell", "uiautomator", "dump", "/data/local/tmp/dd-window.xml")
            tree = ET.fromstring(adb("exec-out", "cat", "/data/local/tmp/dd-window.xml"))
        except (ET.ParseError, subprocess.SubprocessError):
            return False
        for node in tree.iter("node"):
            values = [node.get("text", ""), node.get("content-desc", "")]
            if label in values or prefix and any(value.startswith(label) for value in values):
                bounds = list(map(int, re.findall(r"\d+", node.get("bounds", ""))))
                if len(bounds) != 4:
                    continue
                left, top, right, bottom = bounds
                adb("shell", "input", "tap", str((left + right) // 2), str((top + bottom) // 2))
                return True
        return False

    wait_for(click_matching, label, attempts=5)


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
adb("shell", "appops", "set", PACKAGE, "GET_USAGE_STATS", "allow")
# This emulator is dedicated to this run. Production users grant access in Settings.
adb("shell", "settings", "put", "secure", "enabled_accessibility_services", PACKAGE + "/.BlockService")
adb("shell", "settings", "put", "secure", "accessibility_enabled", "1")
launch(PACKAGE + "/.MainActivity")
tap("Get started")
tap("Individual")
tap("Blocking")
tap("Search installed apps")
adb("shell", "input", "text", fixture_package)
adb("shell", "input", "keyevent", "KEYCODE_BACK")
tap("Block ", prefix=True)

# A blocked launch should return to the launcher, regardless of the transient notice.
home = adb("shell", "cmd", "package", "resolve-activity", "--brief", "-a",
           "android.intent.action.MAIN", "-c", "android.intent.category.HOME").splitlines()[-1].split("/")[0]
launch(component)
wait_for(lambda: home + "/" in top_activity(), "Home after the selected app opens")
time.sleep(1.2)
assert home + "/" in top_activity(), top_activity()
print("Block took effect for " + fixture_package, flush=True)

adb("shell", "input", "keyevent", "KEYCODE_HOME")
time.sleep(5.1)
launch(PACKAGE + "/.MainActivity")
tap("Search installed apps")
adb("shell", "input", "text", fixture_package)
adb("shell", "input", "keyevent", "KEYCODE_BACK")
tap("Block ", prefix=True)
launch(component)
time.sleep(3)
assert fixture_package + "/" in top_activity(), top_activity()
print("The same app stays open after its rule is cleared.", flush=True)
