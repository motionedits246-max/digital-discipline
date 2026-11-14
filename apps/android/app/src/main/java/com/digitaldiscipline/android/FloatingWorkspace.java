package com.digitaldiscipline.android;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Context;
import android.content.Intent;
import android.content.res.Configuration;
import android.graphics.PixelFormat;
import android.graphics.drawable.GradientDrawable;
import android.os.Build;
import android.os.IBinder;
import android.provider.Settings;
import android.view.Gravity;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowManager;
import android.widget.FrameLayout;

public final class FloatingWorkspace extends Service {
    private static final String CHANNEL = "floating-workspace";
    private static final String STOP = "stop";
    private WindowManager windows;
    private Context displayContext;
    private FrameLayout button;

    @Override public IBinder onBind(Intent intent) { return null; }

    @Override public int onStartCommand(Intent intent, int flags, int id) {
        if (intent == null || STOP.equals(intent.getAction()) || !Settings.canDrawOverlays(this)
                || !Rules.INDIVIDUAL.equals(new Rules(this).mode()) || !notificationsAllowed(this)) {
            stopSelf();
            return START_NOT_STICKY;
        }
        PendingIntent stop = PendingIntent.getService(this, 71, new Intent(this, FloatingWorkspace.class)
            .setAction(STOP), PendingIntent.FLAG_IMMUTABLE);
        PendingIntent open = PendingIntent.getActivity(this, 72, workspaceIntent(this), PendingIntent.FLAG_IMMUTABLE);
        startForeground(71, new Notification.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_discipline).setContentTitle("Digital Discipline workspace")
            .setContentText("Tap to open Focus, To-do and Notes")
            .setContentIntent(open).setOngoing(true)
            .addAction(new Notification.Action.Builder(null, "Hide button", stop).build()).build());
        if (button == null) {
            displayContext = Build.VERSION.SDK_INT >= 31
                ? createWindowContext(getSystemService(android.hardware.display.DisplayManager.class).getDisplay(0),
                    WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY, null)
                : this;
            windows = displayContext.getSystemService(WindowManager.class);
            try { addButton(); }
            catch (WindowManager.BadTokenException | SecurityException error) { stopSelf(); }
        }
        return START_NOT_STICKY;
    }

    static Intent workspaceIntent(Context context) {
        return new Intent(context, MainActivity.class).setAction(MainActivity.OPEN_WORKSPACE)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP | Intent.FLAG_ACTIVITY_SINGLE_TOP);
    }

    static boolean notificationsAllowed(Context context) {
        NotificationManager manager = context.getSystemService(NotificationManager.class);
        manager.createNotificationChannel(new NotificationChannel(CHANNEL, "Floating workspace", NotificationManager.IMPORTANCE_LOW));
        return manager.areNotificationsEnabled() && manager.getNotificationChannel(CHANNEL).getImportance() != NotificationManager.IMPORTANCE_NONE;
    }

    static boolean visible;

    private void addButton() {
        button = new FrameLayout(displayContext);
        GradientDrawable surface = new GradientDrawable();
        surface.setColor(0xff251b33);
        surface.setCornerRadius(dp(28));
        surface.setStroke(dp(1), 0xff8875a4);
        button.setBackground(surface);
        button.setContentDescription("Open Digital Discipline workspace");
        button.setFocusable(true);
        BrandMark mark = new BrandMark(displayContext, false);
        mark.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        button.addView(mark, new FrameLayout.LayoutParams(dp(25), dp(25), Gravity.CENTER));
        button.setOnClickListener(view -> startActivity(workspaceIntent(this)));
        windows.addView(button, placement());
        visible = true;
    }

    private WindowManager.LayoutParams placement() {
        WindowManager.LayoutParams params = new WindowManager.LayoutParams(dp(48), dp(48),
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE | WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL
                | WindowManager.LayoutParams.FLAG_SECURE, PixelFormat.TRANSLUCENT);
        params.gravity = Gravity.TOP | Gravity.CENTER_HORIZONTAL;
        int safeTop = 0;
        if (Build.VERSION.SDK_INT >= 30) safeTop = windows.getCurrentWindowMetrics().getWindowInsets()
            .getInsetsIgnoringVisibility(WindowInsets.Type.statusBars() | WindowInsets.Type.displayCutout()).top;
        // The overlay is already laid out inside Android's system-bar safe frame.
        params.y = dp(safeTop > 0 ? 9 : 24);
        return params;
    }

    @Override public void onConfigurationChanged(Configuration change) {
        super.onConfigurationChanged(change);
        if (button != null && button.isAttachedToWindow()) windows.updateViewLayout(button, placement());
    }

    @Override public void onDestroy() {
        visible = false;
        if (button != null && button.isAttachedToWindow()) windows.removeViewImmediate(button);
        stopForeground(STOP_FOREGROUND_REMOVE);
        super.onDestroy();
    }

    private int dp(float value) {
        return Math.round(value * displayContext.getResources().getDisplayMetrics().density);
    }
}
