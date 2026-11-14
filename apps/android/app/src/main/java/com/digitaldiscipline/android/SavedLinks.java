package com.digitaldiscipline.android;

import android.content.Context;
import android.content.SharedPreferences;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.Locale;
import java.util.UUID;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

final class SavedLinks {
    static final String SHORT = "YouTube Short";
    static final String REEL = "Instagram Reel";
    static final String STUDY = "Study link";
    private static final Pattern URL = Pattern.compile("https?://[^\\s<>\\[\\]\\\"]+", Pattern.CASE_INSENSITIVE);
    private static final Pattern SHORT_PATH = Pattern.compile("/shorts/([A-Za-z0-9_-]{11})/?");
    private static final Pattern REEL_PATH = Pattern.compile("/(?:[A-Za-z0-9_.]+/)?reels?/([A-Za-z0-9_-]{3,40})/?");

    record Link(String kind, String url) {}
    private final SharedPreferences prefs;

    SavedLinks(Context context) {
        prefs = context.getSharedPreferences("saved-links", Context.MODE_PRIVATE);
    }

    static Link parse(String text) {
        if (text == null || text.length() > 16_384) return null;
        Matcher urls = URL.matcher(text);
        Link fallback = null;
        while (urls.find()) {
            try {
                URI uri = new URI(urls.group().replaceFirst("[).,;!]+$", ""));
                if (uri.getHost() == null || uri.getUserInfo() != null) continue;
                String host = uri.getHost().toLowerCase(Locale.ROOT);
                String path = uri.getPath() == null ? "" : uri.getPath();
                Matcher shortId = SHORT_PATH.matcher(path);
                Matcher reelId = REEL_PATH.matcher(path);
                if (isHost(host, "youtube.com") && shortId.matches()) {
                    return new Link(SHORT, "https://www.youtube.com/shorts/" + shortId.group(1));
                }
                if (isHost(host, "instagram.com") && reelId.matches()) {
                    return new Link(REEL, "https://www.instagram.com/reel/" + reelId.group(1) + "/");
                }
                if (fallback == null && uri.toASCIIString().length() <= 2048) {
                    fallback = new Link(STUDY, uri.toASCIIString());
                }
            } catch (URISyntaxException ignored) {
                // Some apps include surrounding text or malformed links.
            }
        }
        return fallback;
    }

    private static boolean isHost(String host, String root) {
        return host.equals(root) || host.equals("www." + root) || host.equals("m." + root);
    }

    JSONArray all() {
        try { return new JSONArray(prefs.getString("links", "[]")); }
        catch (JSONException malformed) { return new JSONArray(); }
    }

    int[] shortFormCounts(long from, long to) {
        int[] counts = new int[2];
        JSONArray items = all();
        for (int i = 0; i < items.length(); i++) {
            JSONObject item = items.optJSONObject(i);
            if (item == null || item.optLong("at") < from || item.optLong("at") > to) continue;
            if (SHORT.equals(item.optString("kind"))) counts[0]++;
            if (REEL.equals(item.optString("kind"))) counts[1]++;
        }
        return counts;
    }

    void remove(long id, String url) {
        JSONArray entries = all();
        for (int i = 0; i < entries.length(); i++) {
            JSONObject entry = entries.optJSONObject(i);
            if (entry != null && id == entry.optLong("at") && url.equals(entry.optString("url"))) {
                entries.remove(i);
                prefs.edit().putString("links", entries.toString()).apply();
                return;
            }
        }
    }

    void log(String kind) {
        if (SHORT.equals(kind) || REEL.equals(kind)) append(kind, null);
    }

    void save(Link link) { append(link.kind(), link.url()); }

    private void append(String kind, String url) {
        JSONArray previous = all();
        JSONArray next = new JSONArray();
        try {
            JSONObject entry = new JSONObject().put("kind", kind)
                .put("at", System.currentTimeMillis()).put("id", UUID.randomUUID().toString());
            if (url != null) entry.put("url", url);
            next.put(entry);
            for (int i = 0; i < Math.min(999, previous.length()); i++) next.put(previous.get(i));
            prefs.edit().putString("links", next.toString()).apply();
        } catch (JSONException impossible) {
            throw new IllegalStateException(impossible);
        }
    }
}
