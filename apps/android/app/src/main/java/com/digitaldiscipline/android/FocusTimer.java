package com.digitaldiscipline.android;

import android.content.Context;
import android.content.SharedPreferences;
import android.os.SystemClock;
import android.provider.Settings;

final class FocusTimer {
    record Session(String task, int minutes, long remaining, boolean completed, boolean paused) {
        long duration() { return minutes * 60_000L; }
    }

    private final SharedPreferences preferences;
    private final int boot;

    FocusTimer(Context context) {
        preferences = context.getSharedPreferences("focus", Context.MODE_PRIVATE);
        boot = Settings.Global.getInt(context.getContentResolver(), Settings.Global.BOOT_COUNT, -1);
    }

    int minutes() { return preferences.getInt("minutes", 25); }

    Session current() {
        if (!preferences.getBoolean("active", false)) return null;
        long duration = minutes() * 60_000L;
        boolean paused = preferences.contains("paused");
        long remaining = paused ? preferences.getLong("paused", 0)
            : boot >= 0 && preferences.getInt("boot", -1) == boot
                ? preferences.getLong("elapsedEnd", 0) - SystemClock.elapsedRealtime()
                : preferences.getLong("wallEnd", 0) - System.currentTimeMillis();
        remaining = Math.max(0, Math.min(duration, remaining));
        return new Session(preferences.getString("task", ""), minutes(), remaining, remaining == 0, paused);
    }

    void start(int minutes, String task) {
        schedule(preferences.edit().putInt("minutes", minutes).putString("task", task)
            .putBoolean("active", true), minutes * 60_000L);
    }

    void pause() {
        Session current = current();
        if (current != null && !current.completed() && !current.paused()) {
            preferences.edit().putLong("paused", current.remaining()).apply();
        }
    }

    void resume() {
        Session current = current();
        if (current != null && current.paused()) schedule(preferences.edit(), current.remaining());
    }

    void finish() { preferences.edit().remove("active").remove("task").remove("paused").apply(); }

    private void schedule(SharedPreferences.Editor edit, long duration) {
        edit.remove("paused").putInt("boot", boot)
            .putLong("elapsedEnd", SystemClock.elapsedRealtime() + duration)
            .putLong("wallEnd", System.currentTimeMillis() + duration).apply();
    }
}
