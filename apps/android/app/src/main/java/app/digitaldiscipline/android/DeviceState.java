package app.digitaldiscipline.android;

import android.app.AppOpsManager;
import android.app.usage.UsageEvents;
import android.app.usage.UsageStatsManager;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.ResolveInfo;
import android.net.Uri;
import android.os.Process;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.security.GeneralSecurityException;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.PBEKeySpec;

final class DeviceState {
    static final String INDIVIDUAL = "individual";
    static final String PARENT = "parent";
    static final String CHILD = "child";
    private static final long DAY = 24 * 60 * 60 * 1000L;
    final Context context;
    final SharedPreferences prefs;

    DeviceState(Context context) {
        this.context = context.getApplicationContext();
        prefs = this.context.getSharedPreferences("discipline", Context.MODE_PRIVATE);
    }

    String role() { return prefs.getString("role", ""); }
    String childKey() { return prefs.getString("childKey", ""); }
    String parentCode() { return prefs.getString("parentCode", ""); }

    void choose(String role) {
        prefs.edit().clear().putString("role", role).apply();
    }

    void createChild(String pin) throws GeneralSecurityException {
        byte[] salt = FamilyLink.random(16);
        prefs.edit().clear().putString("role", CHILD)
            .putString("salt", FamilyLink.encode(salt))
            .putString("pin", FamilyLink.encode(hash(pin, salt))).apply();
    }

    String checkPin(String pin) throws GeneralSecurityException {
        long now = System.currentTimeMillis();
        if (now < prefs.getLong("retryAfter", 0)) return "Wait 30 seconds before trying again.";
        if (pin.matches("[0-9]{6}") && MessageDigest.isEqual(
            FamilyLink.decode(prefs.getString("pin", "")),
            hash(pin, FamilyLink.decode(prefs.getString("salt", ""))))) {
            prefs.edit().remove("failures").remove("retryAfter").apply();
            return "";
        }
        int failures = prefs.getInt("failures", 0) + 1;
        if (failures >= 5) {
            prefs.edit().remove("failures").putLong("retryAfter", now + 30_000).apply();
            return "Too many attempts. Try again in 30 seconds.";
        }
        prefs.edit().putInt("failures", failures).apply();
        return "Incorrect parent PIN.";
    }

    private static byte[] hash(String pin, byte[] salt) throws GeneralSecurityException {
        PBEKeySpec spec = new PBEKeySpec(pin.toCharArray(), salt, 210_000, 256);
        try {
            return SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(spec).getEncoded();
        } finally {
            spec.clearPassword();
        }
    }

    boolean usageAllowed() {
        AppOpsManager ops = context.getSystemService(AppOpsManager.class);
        return ops.unsafeCheckOpNoThrow(AppOpsManager.OPSTR_GET_USAGE_STATS, Process.myUid(),
            context.getPackageName()) == AppOpsManager.MODE_ALLOWED;
    }

    long trackingSince() { return prefs.getLong("trackingSince", 0); }

    LinkedHashMap<String, String> apps() {
        Intent launcher = new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER);
        ArrayList<ResolveInfo> matches = new ArrayList<>(context.getPackageManager().queryIntentActivities(launcher, 0));
        matches.sort(new ResolveInfo.DisplayNameComparator(context.getPackageManager()));
        LinkedHashMap<String, String> result = new LinkedHashMap<>();
        for (ResolveInfo info : matches) {
            result.putIfAbsent(info.activityInfo.packageName, info.loadLabel(context.getPackageManager()).toString());
        }
        return result;
    }

    Set<String> protectedApps() {
        Set<String> result = new HashSet<>();
        Collections.addAll(result, context.getPackageName(), "android", "com.android.settings",
            "com.android.systemui", "com.android.phone", "com.android.emergency");
        Intent home = new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME);
        Intent dial = new Intent(Intent.ACTION_DIAL, Uri.parse("tel:"));
        for (Intent intent : new Intent[]{home, dial}) {
            for (ResolveInfo info : context.getPackageManager().queryIntentActivities(intent, 0)) {
                result.add(info.activityInfo.packageName);
            }
        }
        return result;
    }

    Set<String> blocks() { return new HashSet<>(prefs.getStringSet("blocks", Collections.emptySet())); }

    static final Object RULE_LOCK = new Object();

    boolean changeBlock(String pkg, boolean blocked) {
        synchronized (RULE_LOCK) {
            if (blocked && (!apps().containsKey(pkg) || protectedApps().contains(pkg))) return false;
            Set<String> rules = blocks();
            if (blocked) rules.add(pkg);
            else rules.remove(pkg);
            prefs.edit().putStringSet("blocks", rules).apply();
            return true;
        }
    }

    JSONObject snapshot() throws JSONException {
        Map<String, String> apps = apps();
        Set<String> protectedApps = protectedApps();
        Set<String> blocks = blocks();
        boolean tracking = usageAllowed() && trackingSince() != 0;
        UsageReport report = tracking ? usage(apps) : new UsageReport();
        JSONArray entries = new JSONArray();
        for (Map.Entry<String, String> app : apps.entrySet()) {
            String pkg = app.getKey();
            if (protectedApps.contains(pkg)) continue;
            entries.put(new JSONObject().put("pkg", pkg).put("name", app.getValue())
                .put("blocked", blocks.contains(pkg)).put("ms", report.totals.getOrDefault(pkg, 0L)));
        }
        JSONArray history = new JSONArray();
        report.sessions.sort(Comparator.comparingLong((Session item) -> item.start).reversed());
        for (Session session : report.sessions) {
            String label = apps.get(session.pkg);
            if (label == null || protectedApps.contains(session.pkg) || session.end <= session.start) continue;
            history.put(new JSONObject().put("pkg", session.pkg).put("name", label)
                .put("start", session.start).put("ms", session.end - session.start));
            if (history.length() >= 120) break;
        }
        return new JSONObject().put("apps", entries).put("history", history)
            .put("tracking", tracking).put("blocker", BlockService.running);
    }

    private UsageReport usage(Map<String, String> apps) {
        long now = System.currentTimeMillis();
        long optIn = trackingSince();
        long start = Math.max(optIn, now - DAY);
        UsageStatsManager stats = context.getSystemService(UsageStatsManager.class);
        UsageReport report = new UsageReport();
        UsageEvents events = stats.queryEvents(Math.max(optIn, start - DAY), now);
        if (events == null) return report;
        Map<String, Map<Integer, Long>> active = new HashMap<>();
        Map<String, Long> opened = new HashMap<>();
        UsageEvents.Event event = new UsageEvents.Event();
        while (events.hasNextEvent()) {
            events.getNextEvent(event);
            int type = event.getEventType();
            String pkg = event.getPackageName();
            long at = event.getTimeStamp();
            if (type == UsageEvents.Event.SCREEN_NON_INTERACTIVE || type == UsageEvents.Event.KEYGUARD_SHOWN
                || type == UsageEvents.Event.DEVICE_SHUTDOWN || type == UsageEvents.Event.DEVICE_STARTUP) {
                for (Map.Entry<String, Long> entry : opened.entrySet()) {
                    report.record(entry.getKey(), entry.getValue(), at, start);
                }
                active.clear();
                opened.clear();
            } else if (apps.containsKey(pkg)) {
                int instance = java.util.Objects.hashCode(event.getClassName());
                if (type == UsageEvents.Event.ACTIVITY_RESUMED) {
                    active.computeIfAbsent(pkg, unused -> new HashMap<>()).put(instance, at);
                    opened.putIfAbsent(pkg, at);
                } else if (type == UsageEvents.Event.ACTIVITY_PAUSED || type == UsageEvents.Event.ACTIVITY_STOPPED) {
                    Map<Integer, Long> instances = active.get(pkg);
                    if (instances != null && instances.remove(instance) != null && instances.isEmpty()) {
                        report.record(pkg, opened.remove(pkg), at, start);
                        active.remove(pkg);
                    }
                }
            }
        }
        for (Map.Entry<String, Long> entry : opened.entrySet()) {
            report.record(entry.getKey(), entry.getValue(), now, start);
        }
        return report;
    }

    private static final class UsageReport {
        final Map<String, Long> totals = new HashMap<>();
        final ArrayList<Session> sessions = new ArrayList<>();

        void record(String pkg, long opened, long closed, long since) {
            long duration = closed - Math.max(opened, since);
            if (duration <= 0) return;
            totals.merge(pkg, duration, Long::sum);
            if (opened >= since) sessions.add(new Session(pkg, opened, closed));
        }
    }

    private static final class Session {
        final String pkg;
        final long start;
        final long end;

        Session(String pkg, long start, long end) {
            this.pkg = pkg;
            this.start = start;
            this.end = end;
        }
    }
}
