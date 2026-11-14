package com.digitaldiscipline.android;

final class ShortDwell {
    private static final long MIN_DWELL_MS = 1_000;
    private static final long RETURN_GRACE_MS = 300_000;
    private String candidate;
    private long since;
    private long previousSample;
    private String lastCounted;
    private long lastSeen;

    boolean observe(String item, long now) {
        if (item == null) {
            candidate = null;
            return false;
        }
        if (!item.equals(candidate) || now - previousSample > 600) {
            candidate = item;
            since = now;
        }
        previousSample = now;
        if (item.equals(lastCounted) && now - lastSeen <= RETURN_GRACE_MS) {
            lastSeen = now;
            return false;
        }
        if (now - since < MIN_DWELL_MS) return false;
        lastCounted = item;
        lastSeen = now;
        return true;
    }
}
