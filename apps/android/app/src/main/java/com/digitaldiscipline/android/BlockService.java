package com.digitaldiscipline.android;

import android.accessibilityservice.AccessibilityService;
import android.app.KeyguardManager;
import android.app.usage.UsageEvents;
import android.app.usage.UsageStatsManager;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.PixelFormat;
import android.graphics.drawable.GradientDrawable;
import android.os.Handler;
import android.os.Looper;
import android.os.PowerManager;
import android.os.SystemClock;
import android.view.Gravity;
import android.view.WindowManager;
import android.view.accessibility.AccessibilityEvent;
import android.widget.Button;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.TextView;

public final class BlockService extends AccessibilityService {
    private static BlockService connectedService;
    private final Handler handler = new Handler(Looper.getMainLooper());
    private final Runnable close = this::hideNotice;
    private final Runnable checkForeground = this::checkRecentForeground;
    private final SharedPreferences.OnSharedPreferenceChangeListener changes = (prefs, key) -> {
        if ("blocks".equals(key) || "mode".equals(key)) scheduleForegroundCheck();
    };
    private Rules rules;
    private FrameLayout overlay;
    private long lastBlock;
    private String lastPackage;

    static boolean isConnected() { return connectedService != null; }

    @Override protected void onServiceConnected() {
        if (rules == null) {
            rules = new Rules(this);
            rules.prefs.registerOnSharedPreferenceChangeListener(changes);
        }
        connectedService = this;
        if (Rules.CHILD.equals(rules.mode())) Rules.showChildNotice(this);
        scheduleForegroundCheck();
    }

    @Override public void onAccessibilityEvent(AccessibilityEvent event) {
        if (event.getEventType() != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
            || event.getPackageName() == null) return;
        enforce(event.getPackageName().toString(), false);
        if (!handler.hasCallbacks(checkForeground)) scheduleForegroundCheck();
    }

    private boolean hasRules() {
        if (rules == null || rules.blocks().isEmpty()) return false;
        String mode = rules.mode();
        return Rules.INDIVIDUAL.equals(mode) || Rules.CHILD.equals(mode);
    }

    private void scheduleForegroundCheck() {
        handler.removeCallbacks(checkForeground);
        if (hasRules()) handler.postDelayed(checkForeground, 850);
    }

    // Use only recent package lifecycle events, and only with the separately granted Usage Access.
    private void checkRecentForeground() {
        if (!hasRules()) return;
        if (getSystemService(PowerManager.class).isInteractive() && ActivityReport.allowed(this)
            && !getSystemService(KeyguardManager.class).isKeyguardLocked()) {
            long now = System.currentTimeMillis();
            UsageEvents events = getSystemService(UsageStatsManager.class).queryEvents(now - 2300L, now);
            if (events != null) {
                String foreground = null;
                String activity = null;
                UsageEvents.Event event = new UsageEvents.Event();
                while (events.hasNextEvent()) {
                    events.getNextEvent(event);
                    switch (event.getEventType()) {
                        case UsageEvents.Event.ACTIVITY_RESUMED -> {
                            foreground = event.getPackageName();
                            activity = event.getClassName();
                        }
                        case UsageEvents.Event.ACTIVITY_PAUSED, UsageEvents.Event.ACTIVITY_STOPPED -> {
                            if (java.util.Objects.equals(foreground, event.getPackageName())
                                && java.util.Objects.equals(activity, event.getClassName())) {
                                foreground = null;
                                activity = null;
                            }
                        }
                        case UsageEvents.Event.SCREEN_NON_INTERACTIVE, UsageEvents.Event.KEYGUARD_SHOWN -> {
                            foreground = null;
                            activity = null;
                        }
                        default -> {}
                    }
                }
                if (foreground != null) enforce(foreground, true);
            }
        }
        scheduleForegroundCheck();
    }

    private void enforce(String id, boolean confirmedByUsage) {
        if (!hasRules() || !rules.blocks().contains(id) || !Rules.canBlock(this, id)
            || getSystemService(KeyguardManager.class).isKeyguardLocked()) return;
        long now = SystemClock.elapsedRealtime();
        boolean recentlyBlocked = id.equals(lastPackage) && now - lastBlock < 2300;
        if (recentlyBlocked && now - lastBlock < (confirmedByUsage ? 950 : 300)) return;

        boolean homeRequested = !recentlyBlocked || !confirmedByUsage
            ? performGlobalAction(GLOBAL_ACTION_HOME) : false;
        if (!homeRequested) {
            try {
                startActivity(new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK));
                homeRequested = true;
            } catch (android.content.ActivityNotFoundException | SecurityException ignored) {
                homeRequested = performGlobalAction(GLOBAL_ACTION_HOME);
            }
        }
        if (homeRequested) {
            lastPackage = id;
            lastBlock = now;
            if (overlay == null || !recentlyBlocked) showNotice(id, rules.mode());
        }
    }

    private void showNotice(String id, String mode) {
        hideNotice();
        WindowManager wm = getSystemService(WindowManager.class);
        overlay = new FrameLayout(this);
        LinearLayout card = new LinearLayout(this);
        card.setOrientation(LinearLayout.VERTICAL);
        card.setPadding(dp(22), dp(16), dp(22), dp(12));
        GradientDrawable background = new GradientDrawable();
        background.setColor(0xff241d33);
        background.setCornerRadius(dp(18));
        background.setStroke(dp(1), 0xff605181);
        card.setBackground(background);
        card.setElevation(dp(10));

        TextView title = new TextView(this);
        title.setTextColor(0xfff9f1ff);
        title.setTextSize(20);
        title.setText(R.string.app_blocked);
        card.addView(title);
        TextView body = new TextView(this);
        body.setTextColor(0xffdad0eb);
        body.setTextSize(15);
        String name = Rules.apps(this).getOrDefault(id, id);
        int message = Rules.CHILD.equals(mode) ? R.string.blocked_by_parent : R.string.blocked_by_rule;
        body.setText(getString(message, name));
        card.addView(body);
        Button okay = new Button(this);
        okay.setText(android.R.string.ok);
        okay.setOnClickListener(view -> hideNotice());
        card.addView(okay);

        overlay.addView(card, new FrameLayout.LayoutParams(-1, -2));
        WindowManager.LayoutParams params = new WindowManager.LayoutParams(
            Math.min(getResources().getDisplayMetrics().widthPixels - dp(24), dp(430)), -2,
            WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE | WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL,
            PixelFormat.TRANSLUCENT);
        params.gravity = Gravity.CENTER;
        try {
            wm.addView(overlay, params);
            handler.postDelayed(close, 5000);
        } catch (WindowManager.BadTokenException | SecurityException e) {
            overlay = null;
        }
    }

    private int dp(float n) { return Math.round(n * getResources().getDisplayMetrics().density); }

    private void hideNotice() {
        handler.removeCallbacks(close);
        if (overlay != null) {
            if (overlay.isAttachedToWindow()) getSystemService(WindowManager.class).removeView(overlay);
            overlay = null;
        }
    }

    @Override public void onInterrupt() { hideNotice(); }

    @Override public boolean onUnbind(Intent intent) {
        disconnect();
        return super.onUnbind(intent);
    }

    @Override public void onDestroy() {
        disconnect();
        super.onDestroy();
    }

    private void disconnect() {
        if (connectedService == this) connectedService = null;
        handler.removeCallbacks(checkForeground);
        if (rules != null) {
            rules.prefs.unregisterOnSharedPreferenceChangeListener(changes);
            rules = null;
        }
        hideNotice();
    }
}
