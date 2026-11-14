package com.digitaldiscipline.android;

import android.content.Context;
import android.content.SharedPreferences;

import org.json.JSONArray;
import org.json.JSONException;

final class ShortsStats {
    private static final String PREFS = "shorts-experiment";
    private static final long HISTORY_MS = 8L * 86_400_000;
    private final SharedPreferences prefs;

    ShortsStats(Context context) {
        prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
    }

    boolean enabled() { return prefs.getBoolean("enabled", false); }
    void setEnabled(boolean enabled) { prefs.edit().putBoolean("enabled", enabled).apply(); }

    private JSONArray timestamps() {
        try { return new JSONArray(prefs.getString("timestamps", "[]")); }
        catch (JSONException malformed) { return new JSONArray(); }
    }

    void add(long now) {
        JSONArray old = timestamps();
        JSONArray next = new JSONArray();
        next.put(now);
        for (int i = 0; i < old.length() && next.length() < 24_000; i++) {
            long at = old.optLong(i, -1);
            if (at >= now - HISTORY_MS && at <= now) next.put(at);
        }
        prefs.edit().putString("timestamps", next.toString()).apply();
    }

    int count(long from, long to) {
        JSONArray list = timestamps();
        int count = 0;
        for (int i = 0; i < list.length(); i++) {
            long at = list.optLong(i, -1);
            if (at >= from && at <= to) count++;
        }
        return count;
    }
}
