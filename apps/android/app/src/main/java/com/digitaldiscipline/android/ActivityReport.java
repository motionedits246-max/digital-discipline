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

import java.time.Instant;
import java.time.LocalDate;
import java.time.ZoneId;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.TreeMap;

final class ActivityReport {
    static final long HOUR = 3_600_000L;
    private static final long DAY = UsageHistory.DAY;

    static boolean allowed(Context context) {
        AppOpsManager ops = context.getSystemService(AppOpsManager.class);
        UserManager users = context.getSystemService(UserManager.class);
        return users.isUserUnlocked() && ops.unsafeCheckOpNoThrow(AppOpsManager.OPSTR_GET_USAGE_STATS,
            Process.myUid(), context.getPackageName()) == AppOpsManager.MODE_ALLOWED;
    }

    static synchronized JSONObject read(Context context, long enrolled, boolean includeSessions) throws JSONException {
        long now = System.currentTimeMillis();
        ZoneId zone = ZoneId.systemDefault();
        Map<String, String> apps = new TreeMap<>(AppCatalog.names(context));
        try (UsageHistory history = new UsageHistory(context)) {
            boolean access = collectRecent(context, history, apps.keySet(), enrolled, now);
            long oldest = now - history.retention() * DAY;
            List<UsageHistory.Session> sessions = history.sessions(oldest, now);
            if (!includeSessions) {
                Set<String> excluded = history.exclusions();
                sessions.removeIf(session -> excluded.contains(session.id()));
            }
            return summarize(now, zone, apps, sessions, history.coverage(oldest, now),
                access, history.pausedUntil(), includeSessions);
        }
    }

    static synchronized void collectRecent(Context context, long enrolled) {
        try (UsageHistory history = new UsageHistory(context)) {
            collectRecent(context, history, AppCatalog.names(context).keySet(), enrolled, System.currentTimeMillis());
        }
    }

    private static boolean collectRecent(Context context, UsageHistory history, Set<String> apps,
                                         long enrolled, long now) {
        boolean access = allowed(context);
        if (!access) history.markAccessLost();
        if (access && history.accessWasLost()) history.restoreAccess(now);
        long checkpoint = history.checkpoint();
        if (access && now >= history.pausedUntil() && (checkpoint < now || checkpoint - now > DAY)) {
            long from = Math.max(Math.max(enrolled, now - 7 * DAY), checkpoint > now ? now : checkpoint);
            collect(context, history, apps, from, now);
        }
        history.prune(now);
        return access;
    }

    private static void collect(Context context, UsageHistory history, Set<String> apps, long from, long end) {
        if (from >= end) return;
        UsageStatsManager manager = context.getSystemService(UsageStatsManager.class);
        UsageEvents events = manager.queryEvents(Math.max(0, from - DAY), end);
        if (events == null) return;
        ArrayList<UsageHistory.Session> result = new ArrayList<>();
        Set<String> excluded = history.exclusions();
        UsageEvents.Event event = new UsageEvents.Event();
        long first = Long.MAX_VALUE;
        long began = 0;
        String active = null;
        String activity = null;
        boolean interactive = true;
        boolean unlocked = true;
        while (events.getNextEvent(event)) {
            long at = event.getTimeStamp();
            first = Math.min(first, at);
            String id = event.getPackageName();
            switch (event.getEventType()) {
                case UsageEvents.Event.SCREEN_INTERACTIVE -> { interactive = true; continue; }
                case UsageEvents.Event.KEYGUARD_HIDDEN -> { unlocked = true; continue; }
                case UsageEvents.Event.SCREEN_NON_INTERACTIVE -> interactive = false;
                case UsageEvents.Event.KEYGUARD_SHOWN -> unlocked = false;
                case UsageEvents.Event.DEVICE_SHUTDOWN, UsageEvents.Event.DEVICE_STARTUP -> {
                    interactive = true;
                    unlocked = true;
                }
                case UsageEvents.Event.ACTIVITY_RESUMED -> {
                    if (!interactive || !unlocked) continue;
                    if (id != null && id.equals(active)) {
                        activity = event.getClassName();
                        continue;
                    }
                    capture(result, apps, excluded, active, began, at, from);
                    active = id;
                    activity = event.getClassName();
                    began = at;
                    continue;
                }
                case UsageEvents.Event.ACTIVITY_PAUSED, UsageEvents.Event.ACTIVITY_STOPPED -> {
                    if (!Objects.equals(active, id) || !Objects.equals(activity, event.getClassName())) continue;
                }
                default -> { continue; }
            }
            capture(result, apps, excluded, active, began, at, from);
            active = null;
            activity = null;
        }
        if (first == Long.MAX_VALUE) return;
        capture(result, apps, excluded, active, began, end, from);
        history.save(Math.max(first, from), end, result);
    }

    private static void capture(List<UsageHistory.Session> result, Set<String> apps, Set<String> excluded,
                                String id, long began, long end, long from) {
        long start = Math.max(began, from);
        if (id != null && end > start && apps.contains(id) && !excluded.contains(id)) {
            result.add(new UsageHistory.Session(id, start, end, began >= from));
        }
    }

    static JSONObject sample() throws JSONException {
        long now = System.currentTimeMillis();
        ZoneId zone = ZoneId.systemDefault();
        long today = Instant.ofEpochMilli(now).atZone(zone).toLocalDate().atStartOfDay(zone).toInstant().toEpochMilli();
        // Anchor demo times to the afternoon so preview content is available at any hour.
        long end = today + 18 * HOUR;
        long hour = today + 9 * HOUR;
        List<UsageHistory.Session> sessions = java.util.Arrays.asList(
            new UsageHistory.Session("com.android.chrome", hour, hour + 24 * 60_000L, true),
            new UsageHistory.Session("com.google.android.youtube", hour + HOUR, hour + HOUR + 18 * 60_000L, true),
            new UsageHistory.Session("com.android.chrome", hour + 2 * HOUR, hour + 2 * HOUR + 37 * 60_000L, true));
        Map<String, String> apps = new TreeMap<>();
        apps.put("com.android.chrome", "Chrome");
        apps.put("com.google.android.youtube", "YouTube");
        return summarize(end, zone, apps, sessions,
            java.util.Collections.singletonList(new UsageHistory.Window(today, end)), true, 0, true).put("sample", true);
    }

    static JSONObject summarize(long end, ZoneId zone, Map<String, String> apps,
                                List<UsageHistory.Session> stored, List<UsageHistory.Window> coverage,
                                boolean access, long paused, boolean includeSessions) throws JSONException {
        LocalDate today = Instant.ofEpochMilli(end).atZone(zone).toLocalDate();
        long dayStart = midnight(today, zone);
        long rolling = end - DAY;
        long firstHour = Instant.ofEpochMilli(Math.min(dayStart, rolling)).atZone(zone)
            .truncatedTo(ChronoUnit.HOURS).toInstant().toEpochMilli();
        int hours = (int) ((end - firstHour) / HOUR) + 1;
        JSONArray hourly = new JSONArray();
        JSONArray rollingHours = new JSONArray();
        for (int slot = 0; slot < hours; slot++) {
            long start = firstHour + slot * HOUR;
            addBin(hourly, stored, coverage, Math.max(start, dayStart), Math.min(start + HOUR, end));
            addBin(rollingHours, stored, coverage, Math.max(start, rolling), Math.min(start + HOUR, end));
        }

        JSONArray week = new JSONArray();
        for (int index = 6; index >= 0; index--) {
            LocalDate day = today.minusDays(index);
            long start = midnight(day, zone);
            long stop = Math.min(end, midnight(day.plusDays(1), zone));
            JSONArray bins = new JSONArray();
            for (long at = start; at < midnight(day.plusDays(1), zone); at += HOUR) {
                addBin(bins, stored, coverage, at, Math.min(at + HOUR, stop));
            }
            week.put(new JSONObject().put("day", day.toString()).put("start", start).put("end", stop)
                .put("bins", bins));
        }
        Map<String, JSONObject> rows = new TreeMap<>();
        JSONArray timeline = new JSONArray();
        JSONArray opens = new JSONArray();
        long start = midnight(today.minusDays(6), zone);
        for (UsageHistory.Session session : stored) {
            if (session.to() <= start) continue;
            String id = session.id();
            JSONObject row = rows.get(id);
            if (row == null) {
                row = new JSONObject().put("id", id).put("name", apps.getOrDefault(id, id));
                rows.put(id, row);
            }
            row.put("time", row.optLong("time") + overlap(rolling, end, session.from(), session.to()));
            row.put("today", row.optLong("today") + overlap(dayStart, end, session.from(), session.to()));
            if (includeSessions) {
                timeline.put(new JSONObject().put("id", id).put("from", session.from())
                    .put("to", session.to()).put("ongoing", access && paused <= end && session.to() == end)
                    .put("opened", session.open()));
                if (session.open() && session.from() >= rolling && opens.length() < 2000) {
                    opens.put(new JSONObject().put("id", id).put("time", session.from()));
                }
            }
        }
        long last = coverage.isEmpty() ? 0 : coverage.get(coverage.size() - 1).to();
        return new JSONObject().put("from", rolling).put("at", end).put("summaryOnly", !includeSessions)
            .put("tracked", access || last > 0).put("access", access).put("lastCollected", last)
            .put("pausedUntil", paused).put("zone", zone.getId()).put("todayStart", dayStart)
            .put("hourStart", firstHour).put("hours", hourly).put("rollingHours", rollingHours)
            .put("week", week).put("apps", new JSONArray(rows.values())).put("opens", opens)
            .put("sessions", timeline)
            .put("todayObserved", observed(coverage, dayStart, end))
            .put("rollingObserved", observed(coverage, rolling, end));
    }

    private static long observed(List<UsageHistory.Window> coverage, long from, long to) {
        long total = 0;
        for (UsageHistory.Window window : coverage) total += overlap(from, to, window.from(), window.to());
        return total;
    }

    private static void addBin(JSONArray bins, List<UsageHistory.Session> sessions,
                               List<UsageHistory.Window> coverage, long from, long to) {
        if (to <= from || observed(coverage, from, to) == 0) {
            bins.put(JSONObject.NULL);
            return;
        }
        long total = 0;
        for (UsageHistory.Session session : sessions) total += overlap(from, to, session.from(), session.to());
        bins.put(total);
    }

    static long overlap(long a, long b, long from, long to) {
        return Math.max(0, Math.min(b, to) - Math.max(a, from));
    }

    static long midnight(LocalDate day, ZoneId zone) {
        return day.atStartOfDay(zone).toInstant().toEpochMilli();
    }

    private ActivityReport() {}
}
