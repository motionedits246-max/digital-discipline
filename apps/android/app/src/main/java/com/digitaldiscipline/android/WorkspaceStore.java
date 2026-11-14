package com.digitaldiscipline.android;

import android.content.Context;
import android.content.SharedPreferences;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.util.UUID;

final class WorkspaceStore {
    static final String TASKS = "tasks";
    static final String NOTES = "notes";
    private final SharedPreferences prefs;

    WorkspaceStore(Context context) {
        prefs = context.getSharedPreferences("workspace", Context.MODE_PRIVATE);
    }

    JSONArray entries(String list) {
        try { return new JSONArray(prefs.getString(list, "[]")); }
        catch (JSONException malformed) { return new JSONArray(); }
    }

    void save(String list, String id, String title, String body, boolean done) {
        if (id == null) id = UUID.randomUUID().toString();
        try {
            JSONArray entries = entries(list);
            JSONArray next = new JSONArray();
            next.put(new JSONObject().put("id", id).put("title", title).put("body", body)
                .put("done", done).put("updated", System.currentTimeMillis()));
            for (int index = 0; index < entries.length() && next.length() < 250; index++) {
                JSONObject entry = entries.optJSONObject(index);
                if (entry != null && !id.equals(entry.optString("id"))) next.put(entry);
            }
            prefs.edit().putString(list, next.toString()).apply();
        } catch (JSONException impossible) {
            throw new IllegalStateException(impossible);
        }
    }

    void remove(String list, String id) {
        JSONArray entries = entries(list);
        for (int index = 0; index < entries.length(); index++) {
            JSONObject entry = entries.optJSONObject(index);
            if (entry != null && id.equals(entry.optString("id"))) {
                entries.remove(index);
                prefs.edit().putString(list, entries.toString()).apply();
                return;
            }
        }
    }
}
