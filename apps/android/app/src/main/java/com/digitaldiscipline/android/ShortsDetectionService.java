package com.digitaldiscipline.android;

import android.accessibilityservice.AccessibilityService;
import android.app.KeyguardManager;
import android.content.ComponentName;
import android.content.Context;
import android.graphics.Rect;
import android.os.Handler;
import android.os.Looper;
import android.os.PowerManager;
import android.os.SystemClock;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityManager;
import android.view.accessibility.AccessibilityNodeInfo;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.nio.charset.StandardCharsets;
import java.util.List;

public final class ShortsDetectionService extends AccessibilityService {
    static final String YOUTUBE = "com.google.android.youtube";
    private static final String RESOURCE = YOUTUBE + ":id/";
    private static final String[] PAGE_IDS = {"reel_player_page_container", "reel_recycler"};
    private static final String[] CAPTION_IDS = {"reel_title", "reel_player_title", "reel_title_text"};
    private static final String[] AD_IDS = {"ad_badge", "reel_ad_badge", "ad_info_button"};
    private static final int SAMPLE_MS = 250;

    private final Handler handler = new Handler(Looper.getMainLooper());
    private final Runnable sample = this::inspect;
    private ShortDwell dwell = new ShortDwell();
    private ShortsStats stats;
    private byte[] salt;


    static boolean active(Context context) {
        AccessibilityManager manager = context.getSystemService(AccessibilityManager.class);
        ComponentName wanted = new ComponentName(context, ShortsDetectionService.class);
        return manager.getEnabledAccessibilityServiceList(android.accessibilityservice.AccessibilityServiceInfo.FEEDBACK_ALL_MASK)
            .stream().anyMatch(info -> {
                android.content.pm.ServiceInfo service = info.getResolveInfo().serviceInfo;
                return wanted.equals(new ComponentName(service.packageName, service.name));
            });
    }

    @Override protected void onServiceConnected() {
        stats = new ShortsStats(this);
        salt = new byte[16];
        new SecureRandom().nextBytes(salt);
        if (permitted()) handler.post(sample);
    }

    @Override public void onAccessibilityEvent(AccessibilityEvent event) {
        if (!YOUTUBE.contentEquals(event.getPackageName() == null ? "" : event.getPackageName())) return;
        if (permitted()) {
            if (!handler.hasCallbacks(sample)) handler.post(sample);
        } else {
            stopSampling();
        }
    }

    private boolean permitted() {
        return stats != null && stats.enabled() && Rules.INDIVIDUAL.equals(new Rules(this).mode());
    }

    private void inspect() {
        if (!permitted()) {
            stopSampling();
            return;
        }
        String item = null;
        AccessibilityNodeInfo root = getRootInActiveWindow();
        if (root != null) {
            try {
                if (getSystemService(PowerManager.class).isInteractive()
                        && !getSystemService(KeyguardManager.class).isKeyguardLocked()
                        && YOUTUBE.contentEquals(root.getPackageName() == null ? "" : root.getPackageName())) {
                    item = identify(root);
                }
            } finally {
                root.recycle();
            }
        }
        if (dwell.observe(item, SystemClock.elapsedRealtime())) stats.add(System.currentTimeMillis());
        if (item != null) handler.postDelayed(sample, SAMPLE_MS);
    }

    private String identify(AccessibilityNodeInfo root) {
        Rect screen = new Rect();
        root.getBoundsInScreen(screen);
        if (screen.isEmpty() || hasAd(root)) return null;
        for (String id : PAGE_IDS) {
            List<AccessibilityNodeInfo> pages = root.findAccessibilityNodeInfosByViewId(RESOURCE + id);
            String match = null;
            try {
                for (AccessibilityNodeInfo page : pages) {
                    Rect frame = new Rect();
                    page.getBoundsInScreen(frame);
                    if (!page.isVisibleToUser() || !frame.intersect(screen)
                            || frame.width() * 10L < screen.width() * 8L
                            || frame.height() * 10L < screen.height() * 7L) continue;
                    String key = captionKey(page);
                    if (key == null) continue;
                    if (match != null && !match.equals(key)) return null;
                    match = key;
                }
            } finally {
                pages.forEach(AccessibilityNodeInfo::recycle);
            }
            if (match != null) return match;
        }
        return null;
    }

    private boolean hasAd(AccessibilityNodeInfo root) {
        for (String id : AD_IDS) {
            List<AccessibilityNodeInfo> nodes = root.findAccessibilityNodeInfosByViewId(RESOURCE + id);
            try {
                for (AccessibilityNodeInfo node : nodes) if (node.isVisibleToUser()) return true;
            } finally {
                nodes.forEach(AccessibilityNodeInfo::recycle);
            }
        }
        return false;
    }

    private String captionKey(AccessibilityNodeInfo page) {
        for (String id : CAPTION_IDS) {
            List<AccessibilityNodeInfo> captions = page.findAccessibilityNodeInfosByViewId(RESOURCE + id);
            try {
                for (AccessibilityNodeInfo caption : captions) {
                    if (!caption.isVisibleToUser()) continue;
                    CharSequence value = caption.getText();
                    if (value == null || value.length() == 0) value = caption.getContentDescription();
                    if (value == null || value.length() == 0) continue;
                    String name = value.subSequence(0, Math.min(value.length(), 256)).toString().trim();
                    if (!name.isEmpty()) return fingerprint("caption:" + name);
                }
            } finally {
                captions.forEach(AccessibilityNodeInfo::recycle);
            }
        }
        return null;
    }

    private String fingerprint(String input) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            digest.update(salt);
            return android.util.Base64.encodeToString(digest.digest(input.getBytes(StandardCharsets.UTF_8)),
                android.util.Base64.NO_WRAP);
        } catch (NoSuchAlgorithmException impossible) {
            throw new AssertionError(impossible);
        }
    }

    private void stopSampling() {
        handler.removeCallbacks(sample);
        dwell = new ShortDwell();
    }

    @Override public void onInterrupt() { stopSampling(); }

    @Override public void onDestroy() {
        stopSampling();
        super.onDestroy();
    }
}
