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
import android.view.MotionEvent;
import android.view.ViewConfiguration;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.os.Handler;
import android.os.Looper;
import android.graphics.Rect;
import android.animation.ValueAnimator;
import org.json.JSONArray;
import org.json.JSONObject;
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
    private WindowManager.LayoutParams pillPosition;
    private WindowManager.LayoutParams panelPosition;
    private LinearLayout panel;
    private int page;
    private final Handler ui = new Handler(Looper.getMainLooper());
    private final Runnable tick = this::refreshTimer;
    private TextView countdown;
    private Button timerButton;
    private Button endButton;
    private Button silenceButton;
    private int timerPhase = -1;

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
        button.setOnClickListener(view -> { if (panel == null) showPanel(); else closePanel(); });
        button.setOnTouchListener(new View.OnTouchListener() {
            private float startX, startY;
            private int x, y;
            private boolean dragging;
            @Override public boolean onTouch(View view, MotionEvent event) {
                switch (event.getActionMasked()) {
                    case MotionEvent.ACTION_DOWN:
                        startX = event.getRawX();
                        startY = event.getRawY();
                        x = pillPosition.x;
                        y = pillPosition.y;
                        dragging = false;
                        return true;
                    case MotionEvent.ACTION_MOVE:
                        float dx = event.getRawX() - startX;
                        float dy = event.getRawY() - startY;
                        if (Math.hypot(dx, dy) > ViewConfiguration.get(displayContext).getScaledTouchSlop()) dragging = true;
                        if (dragging) {
                            closePanel();
                            pillPosition.x = x + Math.round(dx);
                            pillPosition.y = y + Math.round(dy);
                            clampPill();
                            windows.updateViewLayout(button, pillPosition);
                        }
                        return true;
                    case MotionEvent.ACTION_UP:
                        if (dragging) {
                            getSharedPreferences(CHANNEL, MODE_PRIVATE).edit()
                                .putInt("x", pillPosition.x).putInt("y", pillPosition.y).apply();
                        } else view.performClick();
                        return true;
                    case MotionEvent.ACTION_CANCEL:
                        return true;
                    default:
                        return false;
                }
            }
        });
        pillPosition = placement();
        windows.addView(button, pillPosition);
        visible = true;
    }

    private WindowManager.LayoutParams placement() {
        Rect safe = bounds();
        android.content.SharedPreferences saved = getSharedPreferences(CHANNEL, MODE_PRIVATE);
        WindowManager.LayoutParams params = overlay(dp(66), dp(46));
        params.setTitle("Digital Discipline floating pill");
        params.x = saved.getInt("x", (safe.width() - params.width) / 2);
        params.y = saved.getInt("y", dp(8));
        params.x = Math.max(0, Math.min(params.x, safe.width() - params.width));
        params.y = Math.max(0, Math.min(params.y, safe.height() - params.height));
        return params;
    }

    private Rect bounds() {
        if (Build.VERSION.SDK_INT >= 30) {
            android.view.WindowMetrics metrics = windows.getCurrentWindowMetrics();
            Rect safe = new Rect(metrics.getBounds());
            android.graphics.Insets inset = metrics.getWindowInsets().getInsetsIgnoringVisibility(
                WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
            safe.left += inset.left;
            safe.top += inset.top;
            safe.right -= inset.right;
            safe.bottom -= inset.bottom;
            return safe;
        }
        android.util.DisplayMetrics metrics = new android.util.DisplayMetrics();
        windows.getDefaultDisplay().getMetrics(metrics);
        return new Rect(0, 0, metrics.widthPixels, metrics.heightPixels);
    }

    private void clampPill() {
        Rect safe = bounds();
        pillPosition.x = Math.max(0, Math.min(pillPosition.x, safe.width() - pillPosition.width));
        pillPosition.y = Math.max(0, Math.min(pillPosition.y, safe.height() - pillPosition.height));
    }

    private WindowManager.LayoutParams overlay(int width, int height) {
        WindowManager.LayoutParams params = new WindowManager.LayoutParams(width, height,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE | WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL
                | WindowManager.LayoutParams.FLAG_SECURE, PixelFormat.TRANSLUCENT);
        params.gravity = Gravity.TOP | Gravity.LEFT;
        params.setTitle("Digital Discipline workspace overlay");
        return params;
    }

    private void showPanel() {
        Rect safe = bounds();
        int width = Math.min(dp(350), safe.width() - dp(16));
        int height = Math.min(dp(320), safe.height() - dp(62));
        if (width < dp(180) || height < dp(120)) return;
        panelPosition = overlay(width, height);
        panelPosition.x = Math.max(dp(8), Math.min(pillPosition.x + (pillPosition.width - width) / 2,
            safe.width() - width - dp(8)));
        int below = pillPosition.y + pillPosition.height + dp(6);
        panelPosition.y = below + height <= safe.height() ? below : Math.max(0, pillPosition.y - height - dp(6));
        panel = new LinearLayout(displayContext);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setPadding(dp(12), dp(8), dp(12), dp(10));
        GradientDrawable bg = new GradientDrawable(GradientDrawable.Orientation.TL_BR, new int[]{0xf52b213a, 0xf51a1525});
        bg.setCornerRadius(dp(21));
        bg.setStroke(dp(1), 0xff77638c);
        panel.setBackground(bg);
        populate();
        try { windows.addView(panel, panelPosition); }
        catch (WindowManager.BadTokenException | SecurityException error) { closePanel(); return; }
        if (ValueAnimator.areAnimatorsEnabled()) {
            panel.setAlpha(0);
            panel.setTranslationY(dp(-7));
            panel.animate().alpha(1).translationY(0).setDuration(180).start();
        }
    }

    private void populate() {
        ui.removeCallbacks(tick);
        countdown = null;
        panel.removeAllViews();
        LinearLayout tabs = new LinearLayout(displayContext);
        panel.addView(tabs);
        String[] names = {"Focus", "To-do", "Notes"};
        for (int n = 0; n < names.length; n++) {
            int index = n;
            Button tab = control(names[n], () -> { page = index; finishEditing(); });
            if (page == n) tab.setTextColor(0xffd7bdff);
            tabs.addView(tab, new LinearLayout.LayoutParams(0, -2, 1));
        }
        Button close = control("×", this::closePanel);
        close.setContentDescription("Close workspace");
        tabs.addView(close, new LinearLayout.LayoutParams(dp(48), -2));
        ScrollView scroll = new ScrollView(displayContext);
        panel.addView(scroll, new LinearLayout.LayoutParams(-1, 0, 1));
        LinearLayout body = new LinearLayout(displayContext);
        body.setOrientation(LinearLayout.VERTICAL);
        scroll.addView(body);
        if (page == 0) {
            timerPhase = -1;
            countdown = label("", 27);
            body.addView(countdown);
            long today = ActivityReport.midnight(java.time.LocalDate.now(), java.time.ZoneId.systemDefault());
            int[] counts = new SavedLinks(this).shortFormCounts(today, System.currentTimeMillis());
            body.addView(label("Saved today  ·  " + counts[0] + " Shorts  ·  " + counts[1] + " Reels", 14));
            silenceButton = control("Stop sound", () -> { FocusAlarm.silence(this); refreshTimer(); });
            body.addView(silenceButton);
            timerButton = control("", this::toggleTimer);
            body.addView(timerButton);
            endButton = control("End session", () -> { new FocusTimer(this).finish(false); refreshTimer(); });
            body.addView(endButton);
            refreshTimer();
        } else {
            showEntries(body, page == 1 ? WorkspaceStore.TASKS : WorkspaceStore.NOTES);
        }
    }

    private void toggleTimer() {
        FocusTimer timer = new FocusTimer(this);
        FocusTimer.Session session = timer.current();
        if (session == null) timer.start(timer.plan(), "");
        else if (session.completed()) {
            if (session.hasNext()) timer.next(session, false); else timer.finish(false);
        } else if (session.paused()) timer.resume();
        else timer.pause();
        refreshTimer();
    }

    private void refreshTimer() {
        ui.removeCallbacks(tick);
        if (countdown == null) return;
        FocusTimer timer = new FocusTimer(this);
        FocusTimer.Session session = timer.current();
        int phase = session == null ? 0 : session.completed() ? 3 : session.paused() ? 2 : 1;
        long seconds = session == null ? timer.plan().minutes() * 60L : (session.remaining() + 999) / 1000;
        String time = phase == 3 ? "Timer finished" : String.format(java.util.Locale.getDefault(), "%02d:%02d", seconds / 60, seconds % 60);
        if (!time.contentEquals(countdown.getText())) countdown.setText(time);
        int previous = timerPhase;
        if (phase != timerPhase) {
            timerPhase = phase;
            timerButton.setText(switch (phase) {
                case 1 -> "Pause";
                case 2 -> "Resume";
                case 3 -> session.hasNext() ? "Continue plan" : "Done";
                default -> "Start focus";
            });
            endButton.setVisibility(phase == 1 || phase == 2 ? View.VISIBLE : View.GONE);
        }
        silenceButton.setVisibility(FocusAlarm.ringing ? View.VISIBLE : View.GONE);
        if (phase == 3 && previous != phase) FocusAlarm.checkCompletion(this);
        ui.postDelayed(tick, 300);
    }

    private void showEntries(LinearLayout body, String list) {
        boolean tasks = WorkspaceStore.TASKS.equals(list);
        WorkspaceStore store = new WorkspaceStore(this);
        body.addView(control(tasks ? "Add task" : "New note", () -> editEntry(body, list, null)));
        JSONArray entries = store.entries(list);
        if (entries.length() == 0) body.addView(label(tasks ? "No tasks yet" : "No notes yet", 15));
        for (int i = 0; i < entries.length(); i++) {
            JSONObject entry = entries.optJSONObject(i);
            if (entry == null) continue;
            if (tasks) {
                android.widget.CheckBox check = new android.widget.CheckBox(displayContext);
                check.setText(entry.optString("title"));
                check.setTextColor(0xfff3ecff);
                check.setMinHeight(dp(46));
                check.setChecked(entry.optBoolean("done"));
                check.setOnCheckedChangeListener((view, checked) -> store.save(list, entry.optString("id"),
                    entry.optString("title"), "", checked));
                check.setOnLongClickListener(view -> { editEntry(body, list, entry); return true; });
                body.addView(check);
            } else {
                body.addView(control(entry.optString("title"), () -> editEntry(body, list, entry)));
            }
        }
    }

    private void editEntry(LinearLayout body, String list, JSONObject entry) {
        boolean tasks = WorkspaceStore.TASKS.equals(list);
        body.removeAllViews();
        panelPosition.flags &= ~WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE;
        panelPosition.softInputMode = WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE;
        windows.updateViewLayout(panel, panelPosition);
        EditText title = input(tasks ? "Task" : "Title", entry == null ? "" : entry.optString("title"), 100);
        body.addView(title);
        EditText details = input("Note", entry == null ? "" : entry.optString("body"), 4000);
        if (!tasks) {
            details.setInputType(android.text.InputType.TYPE_CLASS_TEXT | android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE);
            details.setMinLines(2);
            body.addView(details);
        }
        body.addView(control("Save", () -> {
            String name = title.getText().toString().trim();
            if (name.isEmpty()) { title.setError("Enter a title"); return; }
            new WorkspaceStore(this).save(list, entry == null ? null : entry.optString("id"), name,
                tasks ? "" : details.getText().toString(), entry != null && entry.optBoolean("done"));
            finishEditing();
        }));
        body.addView(control("Cancel", this::finishEditing));
        if (entry != null) body.addView(control("Delete", () -> {
            body.removeAllViews();
            body.addView(label(tasks ? "Delete this task?" : "Delete this note?", 16));
            body.addView(control("Delete permanently", () -> {
                new WorkspaceStore(this).remove(list, entry.optString("id"));
                finishEditing();
            }));
            body.addView(control("Cancel", this::finishEditing));
        }));
    }

    private void finishEditing() {
        if ((panelPosition.flags & WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE) == 0) {
            displayContext.getSystemService(android.view.inputmethod.InputMethodManager.class)
                .hideSoftInputFromWindow(panel.getWindowToken(), 0);
        }
        panelPosition.flags |= WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE;
        windows.updateViewLayout(panel, panelPosition);
        populate();
    }

    private EditText input(String hint, String value, int limit) {
        EditText input = new EditText(displayContext);
        input.setTextColor(0xfff3ecff);
        input.setHintTextColor(0xffb6a9c9);
        input.setHint(hint);
        input.setText(value);
        input.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(limit)});
        return input;
    }

    private Button control(String text, Runnable action) {
        Button control = new Button(displayContext);
        control.setAllCaps(false);
        control.setText(text);
        control.setMinHeight(dp(45));
        control.setMinWidth(0);
        control.setPadding(dp(4), 0, dp(4), 0);
        control.setTextColor(0xfff3ecff);
        control.setOnClickListener(view -> action.run());
        return control;
    }

    private TextView label(String text, int size) {
        TextView label = new TextView(displayContext);
        label.setTextColor(0xfff3ecff);
        label.setTextSize(size);
        label.setText(text);
        label.setPadding(dp(6), dp(6), dp(6), dp(6));
        return label;
    }

    private void closePanel() {
        ui.removeCallbacks(tick);
        countdown = null;
        if (panel != null && panel.isAttachedToWindow()) windows.removeViewImmediate(panel);
        panel = null;
    }

    @Override public void onConfigurationChanged(Configuration change) {
        super.onConfigurationChanged(change);
        closePanel();
        if (button != null && button.isAttachedToWindow()) {
            pillPosition = placement();
            windows.updateViewLayout(button, pillPosition);
        }
    }

    @Override public void onDestroy() {
        closePanel();
        visible = false;
        if (button != null && button.isAttachedToWindow()) windows.removeViewImmediate(button);
        stopForeground(STOP_FOREGROUND_REMOVE);
        super.onDestroy();
    }

    private int dp(float value) {
        return Math.round(value * displayContext.getResources().getDisplayMetrics().density);
    }
}
