package com.digitaldiscipline.android;

import android.content.Context;
import android.content.SharedPreferences;
import android.os.SystemClock;
import android.provider.Settings;

import org.json.JSONArray;
import org.json.JSONObject;

final class FocusTimer {
    static final int MAX_MINUTES = 720;
    record Plan(String name, int minutes, int breakMinutes, int rounds) {}
    record Session(String task, Plan plan, int round, boolean breakPeriod, long remaining, boolean paused) {
        boolean completed() { return remaining == 0; }
        int minutes() { return breakPeriod ? plan.breakMinutes() : plan.minutes(); }
        long duration() { return minutes() * 60_000L; }
        boolean hasNext() { return breakPeriod || round < plan.rounds(); }
    }

    private final SharedPreferences preferences;
    private final int boot;

    FocusTimer(Context context) {
        preferences = context.getSharedPreferences("focus", Context.MODE_PRIVATE);
        boot = Settings.Global.getInt(context.getContentResolver(), Settings.Global.BOOT_COUNT, -1);
    }

    static int parse(String input, int minimum, int maximum) {
        try {
            int value = Integer.parseInt(input.trim());
            return value >= minimum && value <= maximum ? value : -1;
        } catch (NumberFormatException error) {
            return -1;
        }
    }

    Plan plan() {
        return new Plan(preferences.getString("planName", ""), preferences.getInt("minutes", 25),
            preferences.getInt("breakMinutes", 5), preferences.getInt("rounds", 1));
    }

    Session current() {
        if (!preferences.getBoolean("active", false)) return null;
        Plan plan = plan();
        boolean breakPeriod = preferences.getBoolean("breakPeriod", false);
        long duration = (breakPeriod ? plan.breakMinutes() : plan.minutes()) * 60_000L;
        boolean paused = preferences.contains("paused");
        long remaining = paused ? preferences.getLong("paused", 0)
            : boot >= 0 && preferences.getInt("boot", -1) == boot
                ? preferences.getLong("elapsedEnd", 0) - SystemClock.elapsedRealtime()
                : preferences.getLong("wallEnd", 0) - System.currentTimeMillis();
        remaining = Math.max(0, Math.min(duration, remaining));
        return new Session(preferences.getString("task", ""), plan,
            preferences.getInt("round", 1), breakPeriod, remaining, paused);
    }

    void start(Plan plan, String task) {
        schedule(preferences.edit().putInt("minutes", plan.minutes()).putInt("breakMinutes", plan.breakMinutes())
            .putInt("rounds", plan.rounds()).putString("planName", plan.name())
            .putString("task", task).putBoolean("active", true)
            .putBoolean("breakPeriod", false).putInt("round", 1), plan.minutes() * 60_000L);
    }

    void next(Session session, boolean taskDone) {
        if (!session.completed() || !session.hasNext()) return;
        boolean takeBreak = !session.breakPeriod() && session.plan().breakMinutes() > 0;
        int nextRound = takeBreak ? session.round() : session.round() + 1;
        long duration = (takeBreak ? session.plan().breakMinutes() : session.plan().minutes()) * 60_000L;
        SharedPreferences.Editor edit = preferences.edit().putBoolean("breakPeriod", takeBreak)
            .putInt("round", nextRound);
        if (!session.breakPeriod()) record(edit, session, taskDone);
        schedule(edit, duration);
    }

    void pause() {
        Session session = current();
        if (session != null && !session.completed() && !session.paused()) {
            preferences.edit().putLong("paused", session.remaining()).apply();
        }
    }

    void resume() {
        Session session = current();
        if (session != null && session.paused() && !session.completed()) schedule(preferences.edit(), session.remaining());
    }

    void finish(boolean taskDone) {
        Session session = current();
        SharedPreferences.Editor edit = preferences.edit().remove("active").remove("task").remove("paused");
        if (session != null && !session.breakPeriod()) record(edit, session, taskDone);
        if (session != null && !session.task().isEmpty() && !taskDone) edit.putString("unfinishedTask", session.task());
        else edit.remove("unfinishedTask");
        edit.apply();
    }

    String unfinishedTask() { return preferences.getString("unfinishedTask", ""); }
    JSONArray history() { return array("history"); }
    JSONArray presets() { return array("presets"); }

    void save(Plan plan) {
        JSONArray presets = presets();
        JSONArray next = new JSONArray();
        next.put(json(plan));
        for (int index = 0; index < presets.length() && next.length() < 20; index++) {
            JSONObject value = presets.optJSONObject(index);
            if (value != null && !plan.name().equalsIgnoreCase(value.optString("name"))) next.put(value);
        }
        preferences.edit().putString("presets", next.toString()).apply();
    }

    void removePreset(int index) {
        JSONArray presets = presets();
        presets.remove(index);
        preferences.edit().putString("presets", presets.toString()).apply();
    }

    static Plan decode(JSONObject value) {
        return new Plan(value.optString("name"), value.optInt("minutes", 25),
            value.optInt("breakMinutes", 5), value.optInt("rounds", 1));
    }

    private static JSONObject json(Plan plan) {
        try {
            return new JSONObject().put("name", plan.name()).put("minutes", plan.minutes())
                .put("breakMinutes", plan.breakMinutes()).put("rounds", plan.rounds());
        } catch (org.json.JSONException impossible) {
            throw new IllegalStateException(impossible);
        }
    }

    private JSONArray array(String key) {
        try { return new JSONArray(preferences.getString(key, "[]")); }
        catch (org.json.JSONException malformed) { return new JSONArray(); }
    }

    private void record(SharedPreferences.Editor edit, Session session, boolean taskDone) {
        long elapsed = session.duration() - session.remaining();
        if (elapsed < 1000) return;
        try {
            JSONArray previous = history();
            JSONArray history = new JSONArray();
            history.put(new JSONObject().put("date", System.currentTimeMillis()).put("duration", elapsed)
                .put("task", session.task()).put("taskDone", taskDone).put("completed", session.completed()));
            for (int i = 0; i < Math.min(99, previous.length()); i++) history.put(previous.get(i));
            edit.putString("history", history.toString());
        } catch (org.json.JSONException impossible) {
            throw new IllegalStateException(impossible);
        }
    }

    private void schedule(SharedPreferences.Editor edit, long duration) {
        edit.remove("paused").putInt("boot", boot)
            .putLong("elapsedEnd", SystemClock.elapsedRealtime() + duration)
            .putLong("wallEnd", System.currentTimeMillis() + duration).apply();
    }
}
