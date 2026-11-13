package com.digitaldiscipline.android;

import android.app.AppOpsManager;
import android.app.usage.UsageEvents;
import android.app.usage.UsageStatsManager;
import android.content.Context;
import android.os.Process;
import android.os.UserManager;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.TimeUnit;

final class ActivityReport {
    private static final long HOUR = TimeUnit.HOURS.toMillis(1);
    private static final long DAY = TimeUnit.HOURS.toMillis(24);

    static boolean allowed(Context context) {
        AppOpsManager ops = context.getSystemService(AppOpsManager.class);
        UserManager users = context.getSystemService(UserManager.class);
        return users.isUserUnlocked() && ops.unsafeCheckOpNoThrow(AppOpsManager.OPSTR_GET_USAGE_STATS,
            Process.myUid(), context.getPackageName()) == AppOpsManager.MODE_ALLOWED;
    }

    static JSONObject read(Context context, long enrolled, boolean includeActivity) throws JSONException {
        long end = System.currentTimeMillis();
        java.time.ZonedDateTime reportTime = java.time.Instant.ofEpochMilli(end).atZone(java.time.ZoneId.systemDefault());
        long today = reportTime.toLocalDate().atStartOfDay(reportTime.getZone()).toInstant().toEpochMilli();
        long lastHour = reportTime.withMinute(0).withSecond(0).withNano(0).toInstant().toEpochMilli();
        long start = Math.max(end - DAY, enrolled);
        Map<String, String> apps = Rules.apps(context);
        Map<String, Long> totals = new HashMap<>();
        Map<String, Long> todayTotals = new HashMap<>();
        long[] hourly = new long[25];
        long firstHour = lastHour - 24 * HOUR;
        ArrayDeque<JSONObject> opens = new ArrayDeque<>();
        boolean tracking = includeActivity && allowed(context);
        if (tracking) {
            UsageStatsManager manager = context.getSystemService(UsageStatsManager.class);
            UsageEvents events = manager.queryEvents(Math.max(0L, start - DAY), end);
            tracking = events != null;
            if (tracking) {
                UsageEvents.Event event = new UsageEvents.Event();
                String active = null;
                String activity = null;
                long activeSince = start;
                boolean screenOn = true;
                boolean unlocked = true;
                while (events.hasNextEvent()) {
                    events.getNextEvent(event);
                    String id = event.getPackageName();
                    long time = event.getTimeStamp();
                    switch (event.getEventType()) {
                        case UsageEvents.Event.SCREEN_NON_INTERACTIVE:
                            screenOn = false;
                            break;
                        case UsageEvents.Event.SCREEN_INTERACTIVE:
                            screenOn = true;
                            continue;
                        case UsageEvents.Event.KEYGUARD_SHOWN:
                            unlocked = false;
                            break;
                        case UsageEvents.Event.KEYGUARD_HIDDEN:
                            unlocked = true;
                            continue;
                        case UsageEvents.Event.DEVICE_SHUTDOWN:
                        case UsageEvents.Event.DEVICE_STARTUP:
                            break;
                        case UsageEvents.Event.ACTIVITY_RESUMED:
                            if (!screenOn || !unlocked) continue;
                            if (id != null && id.equals(active)) {
                                activity = event.getClassName();
                                continue;
                            }
                            add(totals, todayTotals, hourly, firstHour, today, active, activeSince, time, start, end, apps);
                            active = id;
                            activity = event.getClassName();
                            activeSince = time;
                            if (time >= start && apps.containsKey(id)) {
                                if (opens.size() == 2000) opens.removeFirst();
                                opens.add(new JSONObject().put("id", id).put("time", time));
                            }
                            continue;
                        case UsageEvents.Event.ACTIVITY_PAUSED:
                        case UsageEvents.Event.ACTIVITY_STOPPED:
                            if (!java.util.Objects.equals(active, id)
                                || !java.util.Objects.equals(activity, event.getClassName())) continue;
                            break;
                        default:
                            continue;
                    }
                    add(totals, todayTotals, hourly, firstHour, today, active, activeSince, time, start, end, apps);
                    active = null;
                    activity = null;
                }
                add(totals, todayTotals, hourly, firstHour, today, active, activeSince, end, start, end, apps);
            }
        }

        JSONArray rows = new JSONArray();
        ArrayList<String> ids = new ArrayList<>(apps.keySet());
        ids.sort((left, right) -> apps.get(left).compareToIgnoreCase(apps.get(right)));
        java.util.Set<String> protectedApps = AppCatalog.protectedPackages(context);
        for (String id : ids) {
            rows.put(new JSONObject().put("id", id).put("name", apps.get(id))
                .put("time", totals.getOrDefault(id, 0L)).put("today", todayTotals.getOrDefault(id, 0L)).put("blockable", !protectedApps.contains(id)));
        }
        JSONArray hours = new JSONArray();
        for (long duration : hourly) hours.put(duration);
        return new JSONObject().put("from", start).put("at", end).put("tracked", tracking)
            .put("todayStart", today).put("hourStart", firstHour).put("hours", hours)
            .put("zone", reportTime.getZone().getId())
            .put("apps", rows).put("opens", new JSONArray(opens));
    }

    private static void add(Map<String, Long> totals, Map<String, Long> todayTotals,
                            long[] hourly, long firstHour, long today, String id,
                            long began, long finished, long start, long end, Map<String, String> apps) {
        if (id == null || !apps.containsKey(id)) return;
        long from = Math.max(start, began);
        long to = Math.min(end, finished);
        if (to <= from) return;
        totals.merge(id, to - from, Long::sum);
        long todayTime = to - Math.max(today, from);
        if (todayTime > 0) todayTotals.merge(id, todayTime, Long::sum);
        for (long cursor = Math.max(firstHour, from); cursor < to;) {
            int slot = (int) ((cursor - firstHour) / HOUR);
            if (slot >= hourly.length) break;
            long next = Math.min(to, firstHour + (slot + 1L) * HOUR);
            hourly[slot] += next - cursor;
            cursor = next;
        }
    }
}
