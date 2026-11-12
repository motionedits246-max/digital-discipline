package app.digitaldiscipline.android;

import android.accessibilityservice.AccessibilityService;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.graphics.Color;
import android.graphics.PixelFormat;
import android.os.Handler;
import android.os.Looper;
import android.view.Gravity;
import android.view.View;
import android.view.WindowManager;
import android.view.accessibility.AccessibilityEvent;
import android.widget.FrameLayout;
import android.widget.LinearLayout;

import java.io.IOException;
import java.net.InetAddress;

public final class BlockService extends AccessibilityService {
    static volatile boolean running;
    static volatile String pairingCode = "";
    static final String CHANNEL = "family";
    private static final int NOTICE = 40;
    private final Handler handler = new Handler(Looper.getMainLooper());
    private DeviceState state;
    private FamilyLink link;
    private View overlay;
    private String foreground = "";
    private String shown = "";
    private final SharedPreferences.OnSharedPreferenceChangeListener preferences =
        (prefs, key) -> {
            if (key == null || key.equals("blocks") || key.equals("role")) updateOverlay();
            if (key == null || key.equals("role") || key.equals("childKey")) {
                closeLink();
                updateLink();
            }
        };
    private final Runnable poll = new Runnable() {
        @Override public void run() {
            updateLink();
            handler.postDelayed(this, 2500);
        }
    };

    static boolean mayShare(DeviceState state) {
        if (!DeviceState.CHILD.equals(state.role())) return false;
        NotificationManager manager = state.context.getSystemService(NotificationManager.class);
        NotificationChannel channel = manager.getNotificationChannel(CHANNEL);
        return manager.areNotificationsEnabled() && channel != null &&
            channel.getImportance() != NotificationManager.IMPORTANCE_NONE;
    }

    @Override protected void onServiceConnected() {
        if (state == null) {
            state = new DeviceState(this);
            state.prefs.registerOnSharedPreferenceChangeListener(preferences);
        }
        getSystemService(NotificationManager.class).createNotificationChannel(
            new NotificationChannel(CHANNEL, "Family connection", NotificationManager.IMPORTANCE_LOW));
        running = true;
        handler.removeCallbacks(poll);
        handler.post(poll);
    }

    private void updateLink() {
        if (state == null) return;
        InetAddress ip = mayShare(state) && !state.childKey().isEmpty() ? FamilyLink.localAddress() : null;
        if (ip == null || (link != null && !link.onAddress(ip))) closeLink();
        if (ip == null || link != null) return;
        NotificationManager manager = getSystemService(NotificationManager.class);
        PendingIntent open = PendingIntent.getActivity(this, 0,
            new Intent(this, MainActivity.class), PendingIntent.FLAG_IMMUTABLE);
        manager.notify(NOTICE, new Notification.Builder(this, CHANNEL)
            .setSmallIcon(R.drawable.ic_discipline).setContentTitle("Digital Discipline family mode")
            .setContentText("A paired parent can manage blocks and review app usage over Wi-Fi.")
            .setContentIntent(open).setOngoing(true).setOnlyAlertOnce(true).build());
        try {
            link = new FamilyLink(state, ip);
            pairingCode = link.code();
        } catch (IOException e) {
            manager.cancel(NOTICE);
        }
    }

    private void closeLink() {
        pairingCode = "";
        if (link == null) return;
        link.close();
        link = null;
        getSystemService(NotificationManager.class).cancel(NOTICE);
    }

    @Override public void onAccessibilityEvent(AccessibilityEvent event) {
        if (state == null || event.getPackageName() == null) return;
        String pkg = event.getPackageName().toString();
        if (pkg.equals(getPackageName()) && overlay != null &&
            "Digital Discipline block screen".contentEquals(event.getClassName() == null ? "" : event.getClassName())) return;
        if (pkg.equals("com.android.systemui")) {
            hideOverlay();
            return;
        }
        foreground = pkg;
        updateOverlay();
    }

    private void updateOverlay() {
        if (state == null) return;
        String role = state.role();
        if (!(role.equals(DeviceState.CHILD) || role.equals(DeviceState.INDIVIDUAL)) ||
            !state.blocks().contains(foreground) || state.protectedApps().contains(foreground)) {
            hideOverlay();
            return;
        }
        if (overlay != null && shown.equals(foreground)) return;
        hideOverlay();
        showOverlay(state.apps().getOrDefault(foreground, "This app"));
    }

    private void showOverlay(String label) {
        WindowManager windows = getSystemService(WindowManager.class);
        FrameLayout shade = new FrameLayout(this);
        shade.setBackgroundColor(Color.rgb(11, 11, 13));
        LinearLayout card = Ui.card(this);
        card.addView(Ui.text(this, "App blocked", 26, Ui.WHITE, true));
        Ui.body(card, label + " is blocked by Digital Discipline.");
        Ui.action(card, "Back to Home", true, () -> {
            if (performGlobalAction(GLOBAL_ACTION_HOME)) {
                foreground = "";
                hideOverlay();
            }
        });
        FrameLayout.LayoutParams position = new FrameLayout.LayoutParams(
            WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.WRAP_CONTENT, Gravity.CENTER);
        position.setMargins(Ui.dp(this, 24), 0, Ui.dp(this, 24), 0);
        shade.addView(card, position);
        WindowManager.LayoutParams params = new WindowManager.LayoutParams(
            WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.MATCH_PARENT,
            WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE, PixelFormat.TRANSLUCENT);
        params.setTitle("Digital Discipline block screen");
        try {
            windows.addView(shade, params);
            overlay = shade;
            shown = foreground;
        } catch (WindowManager.BadTokenException ignored) {
            // Android disconnected the service before the window was attached.
        }
    }

    private void hideOverlay() {
        if (overlay == null) return;
        getSystemService(WindowManager.class).removeViewImmediate(overlay);
        overlay = null;
        shown = "";
    }

    @Override public void onInterrupt() { hideOverlay(); }

    @Override public boolean onUnbind(Intent intent) {
        stop();
        return super.onUnbind(intent);
    }

    @Override public void onDestroy() {
        stop();
        super.onDestroy();
    }

    private void stop() {
        running = false;
        handler.removeCallbacks(poll);
        if (state != null) state.prefs.unregisterOnSharedPreferenceChangeListener(preferences);
        state = null;
        closeLink();
        hideOverlay();
    }
}
