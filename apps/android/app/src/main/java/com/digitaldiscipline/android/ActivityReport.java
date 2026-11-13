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
import java.time.LocalDate;
import java.time.ZoneId;
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
        long start = Math.max(Math.min(end - DAY, today), enrolled);
        ZoneId zone = reportTime.getZone();
        LocalDate firstDay = reportTime.toLocalDate().minusDays(6);
        long weekStart = firstDay.atStartOfDay(zone).toInstant().toEpochMilli();
        long queryStart = Math.max(0L, Math.min(start, Math.max(weekStart, enrolled)) - DAY);
        long[] boundaries = new long[43];
        for (int i = 0; i < boundaries.length; i++) {
            boundaries[i] = firstDay.plusDays(i / 6).atTime((i % 6) * 4, 0).atZone(zone).toInstant().toEpochMilli();
        }
        long[] weekBins = new long[42];
        long firstObservation = Long.MAX_VALUE;
        boolean[] observedBins = new boolean[42];
        JSONArray sessions = new JSONArray();
        Map<String, String> apps = Rules.apps(context);
        Map<String, Long> totals = new HashMap<>();
        Map<String, Long> todayTotals = new HashMap<>();
        long[] hourly = new long[25];
        long firstHour = lastHour - 24 * HOUR;
        ArrayDeque<JSONObject> opens = new ArrayDeque<>();
        boolean tracking = includeActivity && allowed(context);
        if (tracking) {
            UsageStatsManager manager = context.getSystemService(UsageStatsManager.class);
            UsageEvents events = manager.queryEvents(queryStart, end);
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
                    if (time >= Math.max(weekStart, enrolled)) {
                        firstObservation = Math.min(firstObservation, time);
                        int bin = bucket(boundaries, time);
                        if (bin >= 0) observedBins[bin] = true;
                    }
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
                            capture(sessions, weekBins, boundaries, observedBins, active, activeSince,
                                time, enrolled, today, end, apps);
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
                    capture(sessions, weekBins, boundaries, observedBins, active, activeSince,
                        time, enrolled, today, end, apps);
                    add(totals, todayTotals, hourly, firstHour, today, active, activeSince, time, start, end, apps);
                    active = null;
                    activity = null;
                }
                capture(sessions, weekBins, boundaries, observedBins, active, activeSince,
                    end, enrolled, today, end, apps);
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
        JSONObject report = new JSONObject().put("from", start).put("at", end).put("tracked", tracking)
            .put("todayStart", today).put("hourStart", firstHour).put("hours", hours)
            .put("zone", reportTime.getZone().getId())
            .put("apps", rows).put("opens", new JSONArray(opens));
        if (tracking) {
            report.put("sessions", sessions).put("week", week(context, firstDay, zone, boundaries,
                weekBins, observedBins, firstObservation, end));
        }
        return report;
    }

    private static int bucket(long[] boundaries, long time) {
        int found = java.util.Arrays.binarySearch(boundaries, time);
        int slot = found >= 0 ? found : -found - 2;
        return slot < 0 || slot >= boundaries.length - 1 ? -1 : slot;
    }

    private static void capture(JSONArray sessions, long[] bins, long[] boundaries, boolean[] observed,
                                String id, long began, long finished, long enrolled, long today,
                                long end, Map<String, String> apps) throws JSONException {
        if (id == null || !apps.containsKey(id)) return;
        long from = Math.max(Math.max(enrolled, boundaries[0]), began);
        long to = Math.min(end, finished);
        if (from >= to) return;
        for (int slot = bucket(boundaries, from); slot >= 0 && slot < bins.length
                && boundaries[slot] < to; slot++) {
            long amount = Math.min(to, boundaries[slot + 1]) - Math.max(from, boundaries[slot]);
            if (amount > 0) {
                bins[slot] += amount;
                observed[slot] = true;
            }
        }
        long todayFrom = Math.max(from, today);
        if (todayFrom < to) sessions.put(new JSONObject().put("id", id)
            .put("from", todayFrom).put("to", to).put("ongoing", to == end));
    }

    private static synchronized JSONArray week(Context context, LocalDate firstDay, ZoneId zone,
                                    long[] boundaries, long[] bins, boolean[] observed,
                                    long firstObservation, long end) throws JSONException {
        android.content.SharedPreferences prefs = context.getSharedPreferences("activity-week", Context.MODE_PRIVATE);
        JSONObject saved;
        try {
            saved = new JSONObject(prefs.getString(zone.getId(), "{}"));
        } catch (JSONException malformed) {
            saved = new JSONObject();
        }
        JSONObject snapshot = new JSONObject();
        JSONArray days = new JSONArray();
        for (int day = 0; day < 7; day++) {
            String key = firstDay.plusDays(day).toString();
            JSONArray previous = saved.optJSONArray(key);
            JSONArray values = new JSONArray();
            JSONArray archive = new JSONArray();
            for (int bin = 0; bin < 6; bin++) {
                int slot = day * 6 + bin;
                boolean past = boundaries[slot] < end;
                boolean complete = boundaries[slot + 1] <= end && firstObservation <= boundaries[slot];
                Object old = previous == null || previous.isNull(bin) ? JSONObject.NULL : previous.opt(bin);
                Object value = JSONObject.NULL;
                if (past) {
                    if (complete) value = bins[slot];
                    else if (old instanceof Number prior) value = Math.max(prior.longValue(), bins[slot]);
                    else if (observed[slot]) value = bins[slot];
                }
                values.put(value);
                archive.put(boundaries[slot + 1] <= end ? value : old);
            }
            snapshot.put(key, archive);
            days.put(new JSONObject().put("day", key).put("bins", values));
        }
        prefs.edit().clear().putString(zone.getId(), snapshot.toString()).apply();
        return days;
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
