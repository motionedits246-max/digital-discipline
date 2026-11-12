package com.digitaldiscipline.android;

import android.accessibilityservice.AccessibilityService;
import android.graphics.PixelFormat;
import android.graphics.drawable.GradientDrawable;
import android.os.Handler;
import android.os.Looper;
import android.os.SystemClock;
import android.view.Gravity;
import android.view.WindowManager;
import android.view.accessibility.AccessibilityEvent;
import android.widget.Button;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.TextView;

public final class BlockService extends AccessibilityService {
    private final Handler handler = new Handler(Looper.getMainLooper());
    private final Runnable close = this::hideNotice;
    private Rules rules;
    private FrameLayout overlay;
    private long lastBlock;
    private String lastPackage;

    @Override protected void onServiceConnected() {
        rules = new Rules(this);
        if (Rules.CHILD.equals(rules.mode())) Rules.showChildNotice(this);
    }

    @Override public void onAccessibilityEvent(AccessibilityEvent event) {
        if (rules == null || event.getEventType() != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
            || event.getPackageName() == null || event.getClassName() == null
            || !event.getClassName().toString().contains(".")) return;
        String mode = rules.mode();
        if (!Rules.INDIVIDUAL.equals(mode) && !Rules.CHILD.equals(mode)) return;
        String id = event.getPackageName().toString();
        if (!rules.blocks().contains(id) || !Rules.canBlock(this, id)
            || !Rules.apps(this).containsKey(id)) return;
        if (getSystemService(android.app.KeyguardManager.class).isKeyguardLocked()) return;

        long now = SystemClock.elapsedRealtime();
        if (id.equals(lastPackage) && now - lastBlock < 350) return;
        lastPackage = id;
        lastBlock = now;
        if (performGlobalAction(GLOBAL_ACTION_HOME) || performGlobalAction(GLOBAL_ACTION_BACK)) {
            showNotice(id, mode);
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
        background.setColor(0xff141416);
        background.setCornerRadius(dp(18));
        background.setStroke(dp(1), 0xff424248);
        card.setBackground(background);
        card.setElevation(dp(10));

        TextView title = new TextView(this);
        title.setTextColor(0xfff2f2f4);
        title.setTextSize(20);
        title.setText(R.string.app_blocked);
        card.addView(title);
        TextView body = new TextView(this);
        body.setTextColor(0xffc6c6cf);
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

    @Override public void onDestroy() {
        hideNotice();
        super.onDestroy();
    }
}
