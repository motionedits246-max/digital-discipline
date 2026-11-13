package com.digitaldiscipline.android;

import android.content.ContentValues;
import android.content.Context;
import android.content.SharedPreferences;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.database.sqlite.SQLiteOpenHelper;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import java.util.concurrent.TimeUnit;

final class UsageHistory extends SQLiteOpenHelper {
    record Session(String id, long from, long to, boolean open) {}
    record Window(long from, long to) {}

    static final long DAY = TimeUnit.DAYS.toMillis(1);
    private static final String DB = "usage-history.db";
    private static final Set<String> NONE = java.util.Collections.emptySet();
    private final SharedPreferences prefs;

    UsageHistory(Context context) {
        super(context, DB, null, 1);
        prefs = context.getSharedPreferences("usage-privacy", Context.MODE_PRIVATE);
    }

    @Override public void onCreate(SQLiteDatabase db) {
        db.execSQL("CREATE TABLE sessions (app TEXT NOT NULL, start INTEGER NOT NULL, end INTEGER NOT NULL,"
            + " opened INTEGER NOT NULL, PRIMARY KEY(app,start))");
        db.execSQL("CREATE INDEX sessions_end ON sessions(end)");
        db.execSQL("CREATE TABLE coverage (start INTEGER PRIMARY KEY, end INTEGER NOT NULL)");
    }

    @Override public void onUpgrade(SQLiteDatabase db, int before, int after) {
        throw new IllegalStateException("Unsupported usage database version");
    }

    long pausedUntil() { return prefs.getLong("pausedUntil", 0); }
    int retention() { return prefs.getInt("retention", 30); }
    Set<String> exclusions() { return new java.util.HashSet<>(prefs.getStringSet("excluded", NONE)); }
    long checkpoint() { return prefs.getLong("checkpoint", 0); }

    boolean accessWasLost() { return prefs.getBoolean("accessLost", false); }

    void markAccessLost() {
        if (checkpoint() > 0 && !accessWasLost()) {
            prefs.edit().putBoolean("accessLost", true).apply();
        }
    }

    void restoreAccess(long now) {
        resetCoverageAt(now);
        prefs.edit().remove("accessLost").apply();
    }

    void pause(long now, long duration) {
        long until = now + duration;
        prefs.edit().putLong("pausedUntil", until).putLong("checkpoint", until).apply();
    }

    void resume(long now) {
        prefs.edit().remove("pausedUntil").putLong("checkpoint", now).apply();
    }

    void resetCoverageAt(long now) {
        if (pausedUntil() <= now) prefs.edit().putLong("checkpoint", now).apply();
    }

    void exclude(String id, boolean excluded) {
        Set<String> next = new java.util.HashSet<>(exclusions());
        if (excluded) next.add(id); else next.remove(id);
        prefs.edit().putStringSet("excluded", next).apply();
    }

    void retention(int days, long now) {
        prefs.edit().putInt("retention", days).apply();
        prune(now);
    }

    void prune(long now) { deleteBefore(now - retention() * DAY); }

    private void deleteBefore(long from) {
        SQLiteDatabase db = getWritableDatabase();
        String[] bound = {Long.toString(from)};
        db.beginTransaction();
        try {
            db.delete("sessions", "end <= ?", bound);
            db.delete("coverage", "end <= ?", bound);
            db.setTransactionSuccessful();
        } finally {
            db.endTransaction();
        }
    }

    void deleteAll(long now) {
        SQLiteDatabase db = getWritableDatabase();
        db.beginTransaction();
        try {
            db.delete("sessions", null, null);
            db.delete("coverage", null, null);
            db.setTransactionSuccessful();
        } finally {
            db.endTransaction();
        }
        resetCoverageAt(now);
    }

    void deleteApp(String id, long now) {
        getWritableDatabase().delete("sessions", "app = ?", new String[]{id});
        resetCoverageAt(now);
    }

    void deleteSince(long from, long now) {
        SQLiteDatabase db = getWritableDatabase();
        String[] start = {Long.toString(from)};
        db.beginTransaction();
        try {
            db.delete("sessions", "start >= ?", start);
            db.execSQL("UPDATE sessions SET end = ? WHERE end > ?", new Object[]{from, from});
            db.delete("coverage", "start >= ?", start);
            db.execSQL("UPDATE coverage SET end = ? WHERE end > ?", new Object[]{from, from});
            db.setTransactionSuccessful();
        } finally {
            db.endTransaction();
        }
        resetCoverageAt(now);
    }

    void save(long from, long to, List<Session> sessions) {
        if (to <= from) return;
        SQLiteDatabase db = getWritableDatabase();
        db.beginTransaction();
        try {
            for (Session session : sessions) {
                // An ongoing session can be extended during a later read, even when its
                // beginning predates the last checkpoint.
                String[] previous = {session.id(), Long.toString(session.from())};
                if (!session.open() && session.from() == from) {
                    int extended = db.update("sessions", end(session.to()), "app = ? AND end = ?", previous);
                    if (extended != 0) continue;
                }
                ContentValues values = end(session.to());
                values.put("app", session.id());
                values.put("start", session.from());
                values.put("opened", session.open() ? 1 : 0);
                db.insertWithOnConflict("sessions", null, values, SQLiteDatabase.CONFLICT_REPLACE);
            }
            try (Cursor previous = db.rawQuery("SELECT start FROM coverage WHERE end = ? LIMIT 1",
                    new String[]{Long.toString(from)})) {
                if (previous.moveToFirst()) {
                    db.update("coverage", end(to), "start = ?", new String[]{previous.getString(0)});
                } else {
                    ContentValues values = end(to);
                    values.put("start", from);
                    db.insertWithOnConflict("coverage", null, values, SQLiteDatabase.CONFLICT_REPLACE);
                }
            }
            db.setTransactionSuccessful();
        } finally {
            db.endTransaction();
        }
        prefs.edit().putLong("checkpoint", to).apply();
    }

    List<Session> sessions(long from, long to) {
        ArrayList<Session> result = new ArrayList<>();
        try (Cursor cursor = getReadableDatabase().rawQuery(
                "SELECT app,start,end,opened FROM sessions WHERE end > ? AND start < ? ORDER BY start",
                new String[]{Long.toString(from), Long.toString(to)})) {
            while (cursor.moveToNext()) result.add(new Session(cursor.getString(0), cursor.getLong(1),
                cursor.getLong(2), cursor.getInt(3) != 0));
        }
        return result;
    }

    List<Window> coverage(long from, long to) {
        ArrayList<Window> result = new ArrayList<>();
        try (Cursor cursor = getReadableDatabase().rawQuery(
                "SELECT start,end FROM coverage WHERE end > ? AND start < ? ORDER BY start",
                new String[]{Long.toString(from), Long.toString(to)})) {
            while (cursor.moveToNext()) result.add(new Window(cursor.getLong(0), cursor.getLong(1)));
        }
        return result;
    }

    private static ContentValues end(long time) {
        ContentValues values = new ContentValues();
        values.put("end", time);
        return values;
    }
}
