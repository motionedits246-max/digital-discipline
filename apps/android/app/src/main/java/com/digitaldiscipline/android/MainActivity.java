package com.digitaldiscipline.android;

import android.Manifest;
import android.animation.ValueAnimator;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.res.ColorStateList;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.PersistableBundle;
import android.provider.Settings;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.Gravity;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.text.DateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.concurrent.TimeUnit;

public final class MainActivity extends Activity {
    private static final int BG = 0xff15111e;
    private static final int PANEL = 0xff211a2b;
    private static final int BORDER = 0xff40354f;
    private static final int WHITE = 0xfffff4ff;
    private static final int MUTED = 0xffc4b8d3;
    private static final int ACCENT = 0xffd7bdff;
    private static final int MINT = 0xffa8ebd2;
    private static final int PEACH = 0xffffb6a4;
    private static final int ERROR = 0xffffa299;
    private static final int WRITE_FILE = 40;
    private static final int READ_FILE = 41;
    private static final int MAX_PACKET = 512 * 1024;
    private Rules rules;
    private LinearLayout root;
    private LinearLayout frame;
    private boolean unlocked;
    private boolean choosingMode;
    private int enrollStep;
    private String validatedPairingCode;
    private TextView enrollmentFeedback;
    private String pendingPacket;
    private String pendingReadKind;
    private boolean busy;
    private final Handler ui = new Handler(Looper.getMainLooper());
    static final String OPEN_WORKSPACE = "com.digitaldiscipline.android.OPEN_WORKSPACE";
    private FocusTimer focus;
    private WorkspaceStore workspace;
    private SavedLinks links;
    private AlertDialog workspaceDialog;
    private Runnable workspaceTick;
    private Runnable redrawWorkspace;
    private int workspacePage;
    private final Runnable tickFocus = this::updateFocus;
    private static final int WRITE_SUMMARY = 53;
    private String pendingSummary;
    private TextView focusTime;
    private boolean focusCompleted;
    private android.widget.ProgressBar focusProgress;
    private boolean sample;
    private int range;
    private long selectedDay;
    private static final int TODAY = 0;
    private static final int WEEK = 1;
    private static final int ROLLING = 2;
    private String detailApp;
    private boolean launchPending;
    private FrameLayout stage;
    private BrandMark heroMark;
    private int tab;
    private static final int ACTIVITY = 0;
    private static final int FOCUS = 1;
    private static final int SETTINGS = 2;

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        if (Build.VERSION.SDK_INT >= 30) getWindow().setDecorFitsSystemWindows(false);
        rules = new Rules(this);
        if (rules.prefs.contains("blocks") || rules.prefs.contains("sent_revision")
                || rules.prefs.contains("applied_revision")) {
            rules.prefs.edit().remove("blocks").remove("sent_revision").remove("applied_revision").apply();
        }
        focus = new FocusTimer(this);
        workspace = new WorkspaceStore(this);
        links = new SavedLinks(this);
        launchPending = state == null;
        if (state != null) {
            tab = state.getInt("tab");
            range = state.getInt("range");
            sample = state.getBoolean("sample");
            selectedDay = state.getLong("selectedDay");
            detailApp = state.getString("detailApp");
            choosingMode = state.getBoolean("choosingMode");
        }
    }

    @Override protected void onSaveInstanceState(Bundle out) {
        out.putInt("tab", tab);
        out.putInt("range", range);
        out.putBoolean("sample", sample);
        out.putLong("selectedDay", selectedDay);
        out.putString("detailApp", detailApp);
        out.putBoolean("choosingMode", choosingMode);
        super.onSaveInstanceState(out);
    }

    @Override protected void onResume() {
        super.onResume();
        if (Rules.CHILD.equals(rules.mode())) Rules.showChildNotice(this);
        if (Rules.INDIVIDUAL.equals(rules.mode()) && tab != ACTIVITY) {
            try { ActivityReport.collectRecent(this, rules.prefs.getLong("enrolled", 0)); }
            catch (RuntimeException ignored) { /* Activity can retry when opened. */ }
        }
        if (enrollStep == 0) render();
        if (launchPending) {
            launchPending = false;
            animateLaunch();
        }
        receiveShortcut();
    }

    @Override protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        if (Intent.ACTION_MAIN.equals(intent.getAction()) && intent.hasCategory(Intent.CATEGORY_LAUNCHER)) {
            launchPending = true;
        }
    }

    @Override protected void onStop() {
        super.onStop();
        ui.removeCallbacks(tickFocus);
        if (workspaceDialog != null) workspaceDialog.dismiss();
        if (Rules.CHILD.equals(rules.mode()) && unlocked) {
            unlocked = false;
            detailApp = null;
            tab = SETTINGS;
            render();
        }
    }

    private void render() {
        mount();
        String mode = rules.mode();
        if (Rules.INDIVIDUAL.equals(mode)) addWorkspaceButton();
        if (mode.isEmpty() && !sample) {
            if (enrollStep != 0) renderEnrollment();
            else if (choosingMode) chooseMode();
            else welcome();
            return;
        }
        if (Rules.CHILD.equals(mode) && !unlocked) {
            LinearLayout panel = card("Supervised device");
            panel.setGravity(Gravity.CENTER_HORIZONTAL);
            text(panel, "Your parent can review activity reports shared from this phone.", 16, MUTED, false);
            action(panel, "Parent access", this::askPin);
            return;
        }
        if (detailApp == null || tab != ACTIVITY) dashboardHeader(switch (mode) {
            case Rules.CHILD -> "Child device";
            case Rules.PARENT -> "Parent dashboard";
            default -> "Personal dashboard";
        });
        LinearLayout nav = new LinearLayout(this);
        nav.setOrientation(LinearLayout.HORIZONTAL);
        nav.setPadding(dp(14), dp(7), dp(14), dp(8));
        nav.setBackgroundColor(0xff1e1827);
        frame.addView(nav, new LinearLayout.LayoutParams(-1, -2));
        navButton(nav, "Activity", ACTIVITY);
        if (!Rules.PARENT.equals(mode)) navButton(nav, "Focus", FOCUS);
        if (!sample) navButton(nav, "Settings", SETTINGS);
        if (tab == SETTINGS) {
            settings(mode);
            return;
        }
        if (tab == FOCUS) {
            showFocus();
            return;
        }
        try {
            JSONObject report = sample ? ActivityReport.sample() : Rules.PARENT.equals(mode)
                ? new JSONObject(rules.prefs.getString("report", "{}"))
                : ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), true);
            showActivity(report);
        } catch (Exception e) {
            text(root, "Activity is currently unavailable.", 15, MUTED, false);
        }
    }

    private void mount() {
        ui.removeCallbacks(tickFocus);
        focusTime = null;
        heroMark = null;
        stage = new FrameLayout(this);
        stage.setBackgroundColor(0xff0d0915);
        frame = new LinearLayout(this);
        frame.setOrientation(LinearLayout.VERTICAL);
        if (tab == ACTIVITY && !rules.mode().isEmpty()) frame.setBackgroundColor(0xff100d16);
        else frame.setBackground(new GradientDrawable(GradientDrawable.Orientation.TL_BR,
            new int[]{0xff151020, 0xff0d0915, 0xff151222}));
        ScrollView scroll = new ScrollView(this);
        scroll.setFillViewport(true);
        scroll.setClipToPadding(false);
        root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(dp(22), dp(18), dp(22), dp(24));
        root.setFocusableInTouchMode(true);
        scroll.addView(root, new ScrollView.LayoutParams(-1, -2));
        frame.addView(scroll, new LinearLayout.LayoutParams(-1, 0, 1));
        frame.setOnApplyWindowInsetsListener((view, insets) -> {
            if (Build.VERSION.SDK_INT >= 30) {
                android.graphics.Insets system = insets.getInsets(WindowInsets.Type.systemBars()
                    | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
                view.setPadding(system.left, system.top, system.right, system.bottom);
            }
            return insets;
        });
        stage.addView(frame, new FrameLayout.LayoutParams(-1, -1));
        setContentView(stage);
        frame.requestApplyInsets();
    }

    private void welcome() {
        root.setGravity(Gravity.CENTER_HORIZONTAL);
        heroMark = new BrandMark(this, true);
        root.addView(heroMark, new LinearLayout.LayoutParams(-1, dp(210), 1));
        TextView heading = text(root, "Make time for\nwhat matters.", 39, WHITE, true);
        heading.setGravity(Gravity.CENTER);
        heading.setLetterSpacing(-0.045f);
        TextView description = text(root, "Digital Discipline helps you see where your time goes and set aside time to focus.", 16, MUTED, false);
        description.setGravity(Gravity.CENTER);
        description.setMaxWidth(dp(330));
        action(root, "Preview with sample data", () -> {
            sample = true;
            render();
            enter();
        });
        primary(root, "Get started", () -> {
            choosingMode = true;
            render();
            enter();
        });
    }

    private void chooseMode() {
        backButton(root, () -> {
            choosingMode = false;
            render();
        });
        title("Make it yours", "Choose how you’ll use Digital Discipline.");
        modeRow("Individual", "Track your time and run focus sessions on this phone.", "Activity", () -> {
            rules.prefs.edit().putString("mode", Rules.INDIVIDUAL)
                .putLong("enrolled", System.currentTimeMillis()).apply();
            render();
            enter();
        });
        modeRow("Parent phone", "Review activity reports shared from a child phone.", "Settings", () -> {
            rules.prefs.edit().putString("mode", Rules.PARENT).putString("key", Rules.newKey()).apply();
            tab = SETTINGS;
            render();
            enter();
        });
        modeRow("Child phone", "Connect to your parent’s Digital Discipline app.", "Settings", () -> {
            enrollStep = 1;
            render();
            enter();
        });
    }

    private void modeRow(String label, String description, String icon, Runnable listener) {
        LinearLayout row = new LinearLayout(this);
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setPadding(dp(14), dp(15), dp(13), dp(14));
        row.setBackground(ripple(surface(PANEL)));
        row.setFocusable(true);
        row.setContentDescription(label + ". " + description);
        row.setOnClickListener(view -> listener.run());
        DisciplineArt symbol = new DisciplineArt(this, icon);
        FrameLayout badge = new FrameLayout(this);
        badge.setBackground(surface(0xff34292a));
        FrameLayout.LayoutParams iconSize = new FrameLayout.LayoutParams(dp(23), dp(23), Gravity.CENTER);
        badge.addView(symbol, iconSize);
        row.addView(badge, new LinearLayout.LayoutParams(dp(40), dp(40)));
        LinearLayout copy = new LinearLayout(this);
        copy.setOrientation(LinearLayout.VERTICAL);
        LinearLayout.LayoutParams copyLayout = new LinearLayout.LayoutParams(0, -2, 1);
        copyLayout.setMargins(dp(12), 0, dp(8), 0);
        row.addView(copy, copyLayout);
        text(copy, label, 18, WHITE, true).setPadding(0, 0, 0, dp(3));
        text(copy, description, 14, MUTED, false).setPadding(0, 0, 0, 0);
        TextView arrow = text(row, "›", 26, ACCENT, false);
        arrow.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        arrow.setPadding(0, 0, 0, 0);
        arrow.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(-1, -2);
        layout.topMargin = dp(10);
        root.addView(row, layout);
    }

    private void addWorkspaceButton() {
        if (FloatingWorkspace.visible && Settings.canDrawOverlays(this)) return;
        FrameLayout button = new FrameLayout(this);
        BrandMark mark = new BrandMark(this, false);
        mark.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        button.addView(mark, new FrameLayout.LayoutParams(dp(26), dp(26), Gravity.CENTER));
        button.setBackground(ripple(surface(PANEL)));
        button.setContentDescription("Open Digital Discipline workspace");
        button.setFocusable(true);
        button.setOnClickListener(view -> openWorkspace());
        LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(dp(48), dp(48));
        size.gravity = Gravity.CENTER_HORIZONTAL;
        size.bottomMargin = dp(6);
        root.addView(button, size);
    }

    private void openWorkspace() {
        if (!Rules.INDIVIDUAL.equals(rules.mode()) || isFinishing()) return;
        if (workspaceDialog != null && workspaceDialog.isShowing()) return;
        LinearLayout panel = new LinearLayout(this);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setPadding(dp(16), dp(12), dp(16), dp(12));
        panel.setBackground(glass(0xff2b213a, 0xff191523, 0xff77638c));
        TextView heading = text(panel, "Workspace", 25, WHITE, true);
        heading.setAccessibilityHeading(true);
        LinearLayout tabs = new LinearLayout(this);
        panel.addView(tabs);
        ScrollView scroll = new ScrollView(this);
        LinearLayout body = new LinearLayout(this);
        body.setOrientation(LinearLayout.VERTICAL);
        scroll.addView(body);
        int limit = getResources().getDisplayMetrics().heightPixels / 2;
        panel.addView(scroll, new LinearLayout.LayoutParams(-1, Math.min(dp(330), limit)));
        action(panel, "Close", () -> workspaceDialog.dismiss());
        AlertDialog current = new AlertDialog.Builder(this).setView(panel).create();
        workspaceDialog = current;
        current.setOnDismissListener(dialog -> {
            if (workspaceDialog != current) return;
            if (workspaceTick != null) ui.removeCallbacks(workspaceTick);
            workspaceTick = null;
            workspaceDialog = null;
            redrawWorkspace = null;
        });
        String[] labels = {"Focus", "To-do", "Notes"};
        Runnable draw = () -> {
            if (workspaceTick != null) ui.removeCallbacks(workspaceTick);
            workspaceTick = null;
            body.removeAllViews();
            scroll.scrollTo(0, 0);
            for (int i = 0; i < tabs.getChildCount(); i++) {
                Button button = (Button) tabs.getChildAt(i);
                boolean selected = i == workspacePage;
                button.setSelected(selected);
                button.setTextColor(selected ? BG : WHITE);
                button.setBackground(ripple(surface(selected ? ACCENT : PANEL)));
            }
            if (workspacePage == 0) {
                workspaceFocus(body);
            } else {
                workspaceEntries(body, workspacePage == 1 ? WorkspaceStore.TASKS : WorkspaceStore.NOTES);
            }
        };
        for (int i = 0; i < labels.length; i++) {
            final int page = i;
            Button tabButton = button(labels[i], () -> {
                workspacePage = page;
                draw.run();
            }, false);
            tabButton.setPadding(dp(4), dp(8), dp(4), dp(8));
            tabs.addView(tabButton, new LinearLayout.LayoutParams(0, -2, 1));
        }
        redrawWorkspace = draw;
        current.show();
        draw.run();
        WindowManager.LayoutParams params = workspaceDialog.getWindow().getAttributes();
        workspaceDialog.getWindow().setBackgroundDrawableResource(android.R.color.transparent);
        workspaceDialog.getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        params.gravity = Gravity.TOP | Gravity.CENTER_HORIZONTAL;
        params.y = dp(56);
        params.width = Math.min(getResources().getDisplayMetrics().widthPixels - dp(26), dp(460));
        workspaceDialog.getWindow().setAttributes(params);
        if (ValueAnimator.areAnimatorsEnabled()) {
            panel.setAlpha(0);
            panel.setTranslationY(-dp(12));
            panel.animate().alpha(1).translationY(0).setDuration(240).start();
        }
    }

    private void refreshWorkspace() {
        if (redrawWorkspace != null) redrawWorkspace.run();
    }

    private void workspaceFocus(LinearLayout body) {
        try {
            JSONObject report = ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), false);
            if (report.optLong("todayObserved") > 0) {
                JSONArray apps = report.optJSONArray("apps");
                long total = 0;
                for (int i = 0; apps != null && i < apps.length(); i++) {
                    JSONObject app = apps.optJSONObject(i);
                    if (app != null) total += app.optLong("today");
                }
                sessionLine(body, "Screen time today", sessionDuration(total));
            } else {
                text(body, "No screen time recorded today", 14, MUTED, false);
            }
        } catch (Exception ignored) {
            text(body, "Activity is unavailable", 14, MUTED, false);
        }
        TextView timer = text(body, "", 34, ACCENT, true);
        timer.setPadding(0, dp(12), 0, dp(8));
        workspaceTick = new Runnable() {
            @Override public void run() {
                if (workspaceDialog == null) return;
                FocusTimer.Session session = focus.current();
                long minutes = session == null ? focus.plan().minutes() : (session.remaining() + 59_999) / 60_000;
                String label = session == null ? minutes + " min ready" : session.completed() ? "Session complete"
                    : minutes + " min left" + (session.paused() ? " · Paused" : "");
                if (!label.contentEquals(timer.getText())) timer.setText(label);
                ui.postDelayed(this, 1000);
            }
        };
        workspaceTick.run();
        FocusTimer.Session current = focus.current();
        if (current == null) {
            primary(body, "Start " + focus.plan().minutes() + " min", () -> {
                focus.start(focus.plan(), "");
                refreshWorkspace();
            });
        } else if (!current.completed()) {
            primary(body, current.paused() ? "Resume focus" : "Pause focus", () -> {
                if (current.paused()) focus.resume(); else focus.pause();
                refreshWorkspace();
            });
        }
        action(body, "Open Focus", () -> openFromWorkspace(FOCUS));
        action(body, "Open Activity", () -> openFromWorkspace(ACTIVITY));
        action(body, "Open Settings", () -> openFromWorkspace(SETTINGS));
    }

    private void openFromWorkspace(int page) {
        workspaceDialog.dismiss();
        tab = page;
        detailApp = null;
        render();
        enter();
    }

    private void workspaceEntries(LinearLayout body, String list) {
        boolean tasks = WorkspaceStore.TASKS.equals(list);
        primary(body, tasks ? "Add task" : "New note", () -> editWorkspace(list, null));
        JSONArray entries = workspace.entries(list);
        if (entries.length() == 0) text(body, tasks ? "No tasks yet." : "No notes yet.", 14, MUTED, false);
        for (int index = 0; index < entries.length(); index++) {
            JSONObject entry = entries.optJSONObject(index);
            if (entry == null) continue;
            LinearLayout row = new LinearLayout(this);
            row.setGravity(Gravity.CENTER_VERTICAL);
            if (tasks) {
                android.widget.CheckBox check = new android.widget.CheckBox(this);
                check.setChecked(entry.optBoolean("done"));
                check.setContentDescription("Complete " + entry.optString("title"));
                check.setMinHeight(dp(48));
                check.setMinWidth(dp(48));
                check.setOnCheckedChangeListener((view, checked) -> {
                    workspace.save(list, entry.optString("id"), entry.optString("title"), "", checked);
                    try { entry.put("done", checked); } catch (org.json.JSONException impossible) {
                        throw new IllegalStateException(impossible);
                    }
                });
                row.addView(check);
            }
            Button label = button(entry.optString("title"), () -> editWorkspace(list, entry), false);
            label.setGravity(Gravity.START | Gravity.CENTER_VERTICAL);
            row.addView(label, new LinearLayout.LayoutParams(0, -2, 1));
            body.addView(row, new LinearLayout.LayoutParams(-1, -2));
            divider(body);
        }
    }

    private void editWorkspace(String list, JSONObject entry) {
        boolean tasks = WorkspaceStore.TASKS.equals(list);
        LinearLayout fields = new LinearLayout(this);
        fields.setOrientation(LinearLayout.VERTICAL);
        fields.setPadding(dp(18), dp(6), dp(18), dp(8));
        EditText title = field(fields, tasks ? "Task" : "Title", InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
        title.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(100)});
        EditText notes = tasks ? null : field(fields, "Note", InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE);
        if (!tasks) {
            notes.setSingleLine(false);
            notes.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE | InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
            notes.setMinLines(4);
            notes.setMaxLines(8);
            notes.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(4000)});
        }
        if (entry != null) {
            title.setText(entry.optString("title"));
            if (notes != null) notes.setText(entry.optString("body"));
        }
        AlertDialog.Builder editor = new AlertDialog.Builder(this).setTitle(tasks ? "Task" : "Note")
            .setView(fields).setNegativeButton("Cancel", null).setPositiveButton("Save", null);
        if (entry != null) editor.setNeutralButton("Delete", (dialog, which) ->
            new AlertDialog.Builder(this).setTitle(tasks ? "Delete task?" : "Delete note?")
                .setNegativeButton("Cancel", null).setPositiveButton("Delete", (confirm, action) -> {
                    workspace.remove(list, entry.optString("id"));
                    refreshWorkspace();
                }).show());
        AlertDialog edit = editor.create();
        edit.show();
        edit.getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        edit.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener(view -> {
            String value = title.getText().toString().trim();
            if (value.isEmpty()) {
                title.setError(tasks ? "Enter a task" : "Enter a title");
                return;
            }
            workspace.save(list, entry == null ? null : entry.optString("id"), value,
                tasks ? "" : notes.getText().toString(), entry != null && entry.optBoolean("done"));
            edit.dismiss();
            refreshWorkspace();
        });
    }

    private void floatingSetting() {
        LinearLayout panel = card("Floating workspace");
        text(panel, "Show a small button below the camera area. Android requires display-over-other-apps access and a visible notification. Hide it at any time from the notification.", 14, MUTED, false);
        if (FloatingWorkspace.visible && !Settings.canDrawOverlays(this)) {
            stopService(new Intent(this, FloatingWorkspace.class));
        }
        if (FloatingWorkspace.visible && Settings.canDrawOverlays(this)) {
            action(panel, "Hide floating button", () -> {
                stopService(new Intent(this, FloatingWorkspace.class));
                ui.postDelayed(this::render, 120);
            });
            return;
        }
        action(panel, "Show floating button", () -> {
            if (!Settings.canDrawOverlays(this)) {
                try { startActivity(new Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION,
                    android.net.Uri.parse("package:" + getPackageName()))); }
                catch (android.content.ActivityNotFoundException e) { status("Display-over-other-apps settings are unavailable."); }
                return;
            }
            if (Build.VERSION.SDK_INT >= 33
                    && checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
                requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 51);
                return;
            }
            if (!FloatingWorkspace.notificationsAllowed(this)) {
                startActivity(new Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                    .putExtra(Settings.EXTRA_APP_PACKAGE, getPackageName()));
                return;
            }
            try {
                startForegroundService(new Intent(this, FloatingWorkspace.class));
                ui.postDelayed(this::render, 250);
            } catch (IllegalStateException | SecurityException error) {
                status("The floating workspace could not start.");
            }
        });
    }

    private void receiveShortcut() {
        Intent intent = getIntent();
        if (intent == null) return;
        if (OPEN_WORKSPACE.equals(intent.getAction())) {
            setIntent(new Intent());
            openWorkspace();
            return;
        }
        if ("com.digitaldiscipline.android.OPEN_TODAY".equals(intent.getAction())
                || "com.digitaldiscipline.android.OPEN_FOCUS".equals(intent.getAction())) {
            setIntent(new Intent());
            if (!Rules.INDIVIDUAL.equals(rules.mode())) return;
            tab = "com.digitaldiscipline.android.OPEN_FOCUS".equals(intent.getAction()) ? FOCUS : ACTIVITY;
            range = TODAY;
            selectedDay = 0;
            detailApp = null;
            render();
            enter();
            return;
        }
        if (!Intent.ACTION_SEND.equals(intent.getAction())) return;
        setIntent(new Intent());
        if (!Rules.INDIVIDUAL.equals(rules.mode())) {
            status("Set up Individual mode to save links.");
            return;
        }
        if (!"text/plain".equals(intent.getType())) return;
        CharSequence shared = intent.getCharSequenceExtra(Intent.EXTRA_TEXT);
        SavedLinks.Link link = SavedLinks.parse(shared == null ? null : shared.toString());
        if (link == null) {
            status("Share a web link from the other app.");
            return;
        }
        new AlertDialog.Builder(this).setTitle(link.kind())
            .setMessage(link.url() + "\n\nSave this link to Digital Discipline on this phone?")
            .setNegativeButton("Cancel", null).setPositiveButton("Save link", (dialog, which) -> {
                links.save(link);
                showSavedLinks();
            }).show();
    }

    private void shortFormCard(JSONObject report) {
        long now = System.currentTimeMillis();
        long from = rangeStart(report);
        long end = range == WEEK && selectedDay == 0 ? now : rangeEnd(report, reportZone(report), report.optLong("at", now));
        if (range == WEEK && selectedDay == 0) from = ActivityReport.midnight(java.time.LocalDate.now().minusDays(6),
            java.time.ZoneId.systemDefault());
        int shorts = 0;
        int reels = 0;
        JSONArray items = links.all();
        for (int i = 0; i < items.length(); i++) {
            JSONObject item = items.optJSONObject(i);
            if (item == null || item.optLong("at") < from || item.optLong("at") > end) continue;
            if (SavedLinks.SHORT.equals(item.optString("kind"))) shorts++;
            if (SavedLinks.REEL.equals(item.optString("kind"))) reels++;
        }
        LinearLayout panel = card("Shorts and Reels");
        sessionLine(panel, "YouTube Shorts saved", Integer.toString(shorts));
        sessionLine(panel, "Instagram Reels saved", Integer.toString(reels));
        text(panel, "Use Share in YouTube or Instagram and choose Digital Discipline to count a video here.", 14, MUTED, false);
        action(panel, "Saved links", this::showSavedLinks);
    }

    private void showSavedLinks() {
        JSONArray entries = links.all();
        LinearLayout list = new LinearLayout(this);
        list.setOrientation(LinearLayout.VERTICAL);
        list.setPadding(dp(18), dp(8), dp(18), dp(14));
        ScrollView scroll = new ScrollView(this);
        scroll.addView(list);
        AlertDialog dialog = new AlertDialog.Builder(this).setTitle("Saved links").setView(scroll)
            .setPositiveButton("Done", null).create();
        if (entries.length() == 0) text(list, "Share a link to Digital Discipline to save it here.", 15, MUTED, false);
        for (int i = 0; i < Math.min(entries.length(), 100); i++) {
            JSONObject entry = entries.optJSONObject(i);
            if (entry == null) continue;
            String url = entry.optString("url");
            action(list, entry.optString("kind") + " · " + date(entry.optLong("at")), () -> {
                dialog.dismiss();
                new AlertDialog.Builder(this).setTitle(entry.optString("kind")).setMessage(url)
                    .setNegativeButton("Back", (choice, which) -> showSavedLinks())
                    .setNeutralButton("Remove", (choice, which) -> {
                        links.remove(entry.optLong("at"), url);
                        render();
                        showSavedLinks();
                    })
                    .setPositiveButton("Open link", (choice, which) -> {
                        try { startActivity(new Intent(Intent.ACTION_VIEW, android.net.Uri.parse(url))); }
                        catch (android.content.ActivityNotFoundException error) { status("No app can open this link."); }
                    }).show();
            });
        }
        dialog.show();
        dialog.getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
    }

    private void dashboardHeader(String description) {
        LinearLayout brand = new LinearLayout(this);
        brand.setGravity(Gravity.CENTER_VERTICAL);
        brand.addView(new BrandMark(this, false), new LinearLayout.LayoutParams(dp(27), dp(27)));
        TextView name = text(brand, "Digital Discipline", 16, WHITE, true);
        name.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        name.setPadding(dp(5), 0, 0, 0);
        root.addView(brand, new LinearLayout.LayoutParams(-1, dp(36)));
        if (sample) action(root, "Exit sample preview", this::exitSample);
        if (tab == ACTIVITY) {
            text(root, "Activity", 36, WHITE, true).setLetterSpacing(-0.045f);
            if (!"Personal dashboard".equals(description)) text(root, description, 14, MUTED, false);
            return;
        }
        if (tab == FOCUS) {
            text(root, "Focus", 36, WHITE, true).setLetterSpacing(-0.045f);
            activityArtwork(R.drawable.focus_hourglass, null);
        } else {
            text(root, "Settings", 36, WHITE, true).setLetterSpacing(-0.045f);
        }
    }

    private void enter() {
        if (!ValueAnimator.areAnimatorsEnabled()) return;
        root.setAlpha(0f);
        root.setTranslationY(dp(12));
        root.animate().alpha(1).translationY(0).setDuration(240).start();
    }

    private void settings(String mode) {
        if (Rules.PARENT.equals(mode)) {
            LinearLayout pair = card("Connect a child phone");
            text(pair, "On the child phone, choose Child phone and enter this pairing code. Set a separate parent PIN on that phone.", 15, MUTED, false);
            primary(pair, "Pairing code", this::showKey);
            LinearLayout files = card("Exchange files");
            text(files, "Import an encrypted activity file shared from the child phone.", 15, MUTED, false);
            action(files, "Import child activity", () -> openFile("activity"));
            return;
        }
        LinearLayout usage = card("Usage access");
        boolean usageAllowed = ActivityReport.allowed(this);
        badge(usage, usageAllowed ? "Done" : "Not set up", usageAllowed ? MINT : PEACH);
        text(usage, usageAllowed ? "Digital Discipline can read app-open times and duration."
            : "Allow Android Usage Access to see when and how long you use each app.", 15, MUTED, false);
        if (usageAllowed) action(usage, "Manage Usage Access", () -> launchSettings(Settings.ACTION_USAGE_ACCESS_SETTINGS));
        else primary(usage, "Allow Usage Access", () -> launchSettings(Settings.ACTION_USAGE_ACCESS_SETTINGS));

        if (Rules.INDIVIDUAL.equals(mode)) {
            floatingSetting();
            action(root, "Saved links", this::showSavedLinks);
        }
        showPrivacy();
        if (Rules.CHILD.equals(mode)) {
            LinearLayout exchange = card("Parent file exchange");
            text(exchange, "The encrypted file includes names and totals for apps with recorded activity and available weekly totals.", 15, MUTED, false);
            action(exchange, "Export child activity", this::exportActivity);
            action(root, "Lock parent settings", () -> {
                unlocked = false;
                render();
            });
        }
    }

    private void renderEnrollment() {
        backButton(root, this::backFromEnrollment);
        if (enrollStep == 1) {
            text(root, "STEP 1 OF 2", 12, ACCENT, true).setLetterSpacing(0.14f);
            title("Connect this phone", "On the parent phone, open Settings → Pairing code → Copy code.");
            LinearLayout box = card("Pairing code");
            EditText code = field(box, "Paste the 43-character code", InputType.TYPE_CLASS_TEXT
                | InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
            code.setTypeface(Typeface.MONOSPACE);
            if (validatedPairingCode != null) code.setText(validatedPairingCode);
            enrollmentFeedback = text(box, "The code is case-sensitive.", 14, MUTED, false);
            action(box, "Paste pairing code", () -> pastePairingCode(code));
            primary(root, "Continue", () -> {
                String input = PairingInput.normalize(code.getText().toString());
                if (!PairingInput.isCode(input)) {
                    enrollmentError(input.isEmpty() ? "Paste the pairing code from the parent phone."
                        : input.length() != 43
                            ? "The code needs 43 characters. You entered " + input.length() + "."
                            : "This code contains an invalid character. Copy it again from the parent phone.");
                    return;
                }
                try {
                    Rules.keyBytes(input);
                } catch (IllegalArgumentException e) {
                    enrollmentError("This code could not be decoded. Copy it again from the parent phone.");
                    return;
                }
                validatedPairingCode = input;
                hideKeyboard();
                enrollStep = 2;
                render();
                enter();
            });
        } else {
            text(root, "STEP 2 OF 2", 12, ACCENT, true).setLetterSpacing(0.14f);
            title("Set a parent PIN", "This PIN protects settings on this phone. Choose 6–12 digits.");
            LinearLayout box = card("Parent access");
            int pinType = InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD;
            EditText pin = field(box, "Parent PIN", pinType);
            EditText confirm = field(box, "Confirm parent PIN", pinType);
            enrollmentFeedback = text(box, "", 14, MUTED, false);
            enrollmentFeedback.setVisibility(View.GONE);
            LinearLayout privacy = card("Supervision notification");
            text(privacy, "Your child sees when this phone is supervised. You can share app-open times and usage with the parent phone.", 14, MUTED, false);
            action(privacy, "Notification settings", this::openNotificationSettings);
            primary(root, "Enable supervision", () -> enableSupervision(pin, confirm));
        }
    }

    private void enableSupervision(EditText pin, EditText confirm) {
        String value = pin.getText().toString();
        if (!PairingInput.isPin(value)) {
            enrollmentError("Choose a parent PIN with 6–12 digits.");
            return;
        }
        if (!value.contentEquals(confirm.getText())) {
            enrollmentError("The two PINs don’t match.");
            return;
        }
        if (Build.VERSION.SDK_INT >= 33
            && checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            hideKeyboard();
            requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 44);
            enrollmentError("Allow the notification, then tap Enable supervision again.");
            return;
        }
        if (!Rules.noticeAllowed(this)) {
            enrollmentError("Allow the supervision notification in Android Settings, then try again.");
            openNotificationSettings();
            return;
        }
        try {
            rules.enroll(validatedPairingCode, value);
        } catch (Exception e) {
            enrollmentError("Digital Discipline couldn’t save setup. Please try again.");
            return;
        }
        hideKeyboard();
        Rules.showChildNotice(this);
        validatedPairingCode = null;
        enrollStep = 0;
        unlocked = true;
        tab = SETTINGS;
        render();
        enter();
    }

    private void openNotificationSettings() {
        try {
            startActivity(new Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                .putExtra(Settings.EXTRA_APP_PACKAGE, getPackageName()));
        } catch (android.content.ActivityNotFoundException e) {
            enrollmentError("Open Digital Discipline notification settings in Android Settings.");
        }
    }

    private void enrollmentError(String message) {
        enrollmentFeedback.setText(message);
        enrollmentFeedback.setTextColor(ERROR);
        enrollmentFeedback.setVisibility(View.VISIBLE);
        enrollmentFeedback.announceForAccessibility(message);
    }

    private void pastePairingCode(EditText code) {
        ClipboardManager clipboard = getSystemService(ClipboardManager.class);
        ClipData data = clipboard.getPrimaryClip();
        CharSequence copied = data == null || data.getItemCount() == 0 ? null : data.getItemAt(0).getText();
        if (copied == null || copied.length() > 4096 || PairingInput.normalize(copied.toString()).isEmpty()) {
            enrollmentError("No pairing code found on the clipboard. Copy it on the parent phone or enter it here.");
            return;
        }
        code.setText(PairingInput.normalize(copied.toString()));
        code.setSelection(code.length());
        enrollmentFeedback.setTextColor(MUTED);
        enrollmentFeedback.setText("Check the code, then tap Continue.");
        enrollmentFeedback.setVisibility(View.VISIBLE);
    }

    private void backFromEnrollment() {
        hideKeyboard();
        if (enrollStep == 1) validatedPairingCode = null;
        enrollStep--;
        render();
    }

    private void hideKeyboard() {
        View focus = getCurrentFocus();
        if (focus != null) {
            getSystemService(android.view.inputmethod.InputMethodManager.class)
                .hideSoftInputFromWindow(focus.getWindowToken(), 0);
            focus.clearFocus();
        }
    }

    @Override public void onBackPressed() {
        if (detailApp != null) {
            closeAppDetails();
        } else if (sample) {
            exitSample();
        } else if (selectedDay != 0) {
            selectedDay = 0;
            render();
        } else if (enrollStep > 0) {
            backFromEnrollment();
        } else if (rules.mode().isEmpty() && choosingMode) {
            choosingMode = false;
            render();
        } else {
            super.onBackPressed();
        }
    }

    private void askPin() {
        long now = System.currentTimeMillis();
        long until = rules.prefs.getLong("retry_after", 0);
        if (until > now && until - now <= 30_000) {
            status("Too many attempts. Try again in " + ((until - now + 999) / 1000) + " seconds.");
            return;
        }
        LinearLayout fields = new LinearLayout(this);
        fields.setPadding(dp(20), 0, dp(20), 0);
        fields.setOrientation(LinearLayout.VERTICAL);
        EditText input = field(fields, "Parent PIN", InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD);
        new AlertDialog.Builder(this).setTitle("Parent access").setView(fields)
            .setNegativeButton("Cancel", null).setPositiveButton("Unlock", (dialog, which) -> {
                try {
                    if (rules.verify(input.getText().toString())) {
                        rules.prefs.edit().remove("pin_attempts").remove("retry_after").apply();
                        unlocked = true;
                        tab = SETTINGS;
                        render();
                        return;
                    }
                } catch (Exception ignored) {
                    status("Parent access is unavailable.");
                    return;
                }
                int attempts = rules.prefs.getInt("pin_attempts", 0) + 1;
                if (attempts >= 5) {
                    rules.prefs.edit().putInt("pin_attempts", 0)
                        .putLong("retry_after", System.currentTimeMillis() + 30_000).apply();
                    status("Too many attempts. Try again in 30 seconds.");
                } else {
                    rules.prefs.edit().putInt("pin_attempts", attempts).apply();
                    status("Incorrect parent PIN.");
                }
            }).show();
    }

    private void showKey() {
        String key = rules.prefs.getString("key", "");
        LinearLayout sheet = new LinearLayout(this);
        sheet.setOrientation(LinearLayout.VERTICAL);
        sheet.setPadding(dp(22), dp(22), dp(22), dp(20));
        sheet.setBackground(surface(PANEL));
        text(sheet, "Pairing code", 24, WHITE, true);
        text(sheet, "Enter this code on the child phone. Keep it private.", 15, MUTED, false);
        TextView code = text(sheet, key, 16, ACCENT, false);
        code.setTypeface(Typeface.MONOSPACE);
        code.setTextIsSelectable(true);
        code.setPadding(dp(12), dp(12), dp(12), dp(12));
        code.setBackground(surface(BG));
        AlertDialog dialog = new AlertDialog.Builder(this).setView(sheet).create();
        primary(sheet, "Copy code", () -> {
            ClipboardManager clipboard = getSystemService(ClipboardManager.class);
            ClipData clip = ClipData.newPlainText("Digital Discipline pairing code", key);
            PersistableBundle extras = new PersistableBundle();
            extras.putBoolean("android.content.extra.IS_SENSITIVE", true);
            clip.getDescription().setExtras(extras);
            clipboard.setPrimaryClip(clip);
            if (Build.VERSION.SDK_INT < 33) status("Pairing code copied.");
            dialog.dismiss();
        });
        action(sheet, "Done", dialog::dismiss);
        dialog.show();
        if (dialog.getWindow() != null) {
            dialog.getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
            dialog.getWindow().setBackgroundDrawableResource(android.R.color.transparent);
        }
    }

    private TextView badge(LinearLayout parent, String label, int color) {
        TextView badge = new TextView(this);
        badge.setTextSize(13);
        badge.setTypeface(Typeface.create("sans-serif-medium", Typeface.NORMAL));
        badge.setPadding(dp(11), dp(5), dp(11), dp(5));
        LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(-2, -2);
        size.bottomMargin = dp(9);
        parent.addView(badge, size);
        setBadge(badge, label, color);
        return badge;
    }

    private void setBadge(TextView badge, String label, int color) {
        GradientDrawable fill = new GradientDrawable();
        fill.setCornerRadius(dp(12));
        fill.setColor((color & 0xffffff) | 0x22000000);
        badge.setBackground(fill);
        badge.setTextColor(color);
        badge.setText(label);
    }

    private void showFocus() {
        if (sample) {
            badge(root, "Sample preview", ACCENT);
            LinearLayout demo = card("Focus session");
            TextView clock = text(demo, "25:00", 52, WHITE, true);
            primary(demo, "Preview countdown", () -> {
                long started = android.os.SystemClock.elapsedRealtime();
                Runnable preview = new Runnable() {
                    @Override public void run() {
                        if (!clock.isAttachedToWindow()) return;
                        long seconds = Math.max(0, 25 - (android.os.SystemClock.elapsedRealtime() - started) / 1000);
                        clock.setText(String.format(Locale.getDefault(), "00:%02d", seconds));
                        if (seconds > 0) clock.postDelayed(this, 250);
                    }
                };
                preview.run();
            });
            return;
        }
        FocusTimer.Session session = focus.current();
        LinearLayout panel = card(session == null ? "Start a session"
            : session.completed() ? (session.breakPeriod() ? "Break complete" : "Session complete")
            : session.paused() ? "Paused" : session.breakPeriod() ? "On a break" : "In focus");
        if (session == null) {
            EditText task = field(panel, "Task (optional)", InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_CAP_SENTENCES);
            task.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(80)});
            FocusTimer.Plan last = focus.plan();
            EditText minutes = numberField(panel, "Focus minutes", last.minutes());
            LinearLayout chips = new LinearLayout(this);
            for (int value : new int[]{15, 25, 45, 60}) {
                Button chip = button(value + " min", () -> minutes.setText(Integer.toString(value)), false);
                chip.setPadding(0, dp(5), 0, dp(5));
                LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(0, -2, 1);
                size.setMarginEnd(dp(4));
                chips.addView(chip, size);
            }
            panel.addView(chips);
            primary(panel, "Start focus", () -> startFocus(minutes, task, null, null, ""));
            action(panel, "Work and break cycles", () -> focusPlan(task, minutes));
            if (!focus.unfinishedTask().isEmpty()) {
                action(panel, "Use unfinished task", () -> task.setText(focus.unfinishedTask()));
            }
            JSONArray presets = focus.presets();
            if (presets.length() > 0) {
                LinearLayout saved = card("Presets");
                for (int i = 0; i < presets.length(); i++) {
                    JSONObject entry = presets.optJSONObject(i);
                    if (entry == null) continue;
                    int index = i;
                    FocusTimer.Plan plan = FocusTimer.decode(entry);
                    action(saved, plan.name() + " · " + plan.minutes() + " min", () ->
                        new AlertDialog.Builder(this).setTitle(plan.name())
                            .setMessage(plan.rounds() + " round(s) · " + plan.breakMinutes() + " min break")
                            .setNegativeButton("Cancel", null)
                            .setNeutralButton("Remove preset", (dialog, which) -> {
                                focus.removePreset(index);
                                render();
                            })
                            .setPositiveButton("Start", (dialog, which) -> beginFocus(plan, task.getText().toString())).show());
                }
            }
            showFocusHistory();
            return;
        }
        if (!session.task().isEmpty()) text(panel, session.task(), 20, WHITE, true);
        focusTime = text(panel, "", 52, WHITE, true);
        focusTime.setTypeface(Typeface.create("sans-serif-light", Typeface.NORMAL));
        focusProgress = new android.widget.ProgressBar(this, null, android.R.attr.progressBarStyleHorizontal);
        focusProgress.setMax(1000);
        focusProgress.setProgressTintList(ColorStateList.valueOf(MINT));
        focusProgress.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        panel.addView(focusProgress, new LinearLayout.LayoutParams(-1, dp(14)));
        if (session.plan().rounds() > 1) {
            text(panel, "Round " + session.round() + " of " + session.plan().rounds(), 14, MUTED, false);
        }
        if (session.completed()) {
            android.widget.CheckBox done = new android.widget.CheckBox(this);
            done.setText("Mark task complete");
            if (!session.task().isEmpty() && !session.breakPeriod()) panel.addView(done);
            if (session.hasNext()) {
                primary(panel, session.breakPeriod() || session.plan().breakMinutes() == 0 ? "Start next round" : "Start break", () -> {
                    focus.next(session, done.isChecked());
                    render();
                    enter();
                });
            } else {
                primary(panel, "Repeat session", () -> {
                    focus.finish(done.isChecked());
                    beginFocus(session.plan(), done.isChecked() ? "" : session.task());
                });
            }
            action(panel, "Done", () -> {
                focus.finish(done.isChecked());
                render();
            });
        } else {
            primary(panel, session.paused() ? "Resume" : "Pause", () -> {
                if (session.paused()) focus.resume(); else focus.pause();
                render();
            });
            action(panel, "End session", () -> new AlertDialog.Builder(this).setTitle("End this focus session?")
                .setNegativeButton("Keep session", null).setPositiveButton("End session", (dialog, which) -> {
                    focus.finish(false);
                    render();
                }).show());
        }
        focusCompleted = session.completed();
        updateFocus();
    }

    private EditText numberField(LinearLayout panel, String label, int value) {
        text(panel, label, 14, MUTED, true);
        EditText number = field(panel, label, InputType.TYPE_CLASS_NUMBER);
        number.setContentDescription(label);
        number.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(3)});
        number.setText(Integer.toString(value));
        return number;
    }

    private void focusPlan(EditText task, EditText duration) {
        LinearLayout body = new LinearLayout(this);
        body.setOrientation(LinearLayout.VERTICAL);
        body.setPadding(dp(18), dp(8), dp(18), dp(12));
        ScrollView scroll = new ScrollView(this);
        scroll.addView(body);
        EditText name = field(body, "Preset name (optional)", InputType.TYPE_CLASS_TEXT);
        name.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(32)});
        int initial = FocusTimer.parse(duration.getText().toString(), 1, FocusTimer.MAX_MINUTES);
        EditText minutes = numberField(body, "Focus minutes", initial > 0 ? initial : 25);
        EditText pause = numberField(body, "Break minutes", focus.plan().breakMinutes());
        EditText rounds = numberField(body, "Rounds", Math.max(2, focus.plan().rounds()));
        AlertDialog dialog = new AlertDialog.Builder(this).setTitle("Work and break cycles").setView(scroll)
            .setNegativeButton("Cancel", null).setPositiveButton("Start", null)
            .setNeutralButton("Save preset", null).create();
        dialog.show();
        dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener(view -> {
            if (startFocus(minutes, task, pause, rounds, name.getText().toString().trim())) dialog.dismiss();
        });
        dialog.getButton(AlertDialog.BUTTON_NEUTRAL).setOnClickListener(view -> {
            String title = name.getText().toString().trim();
            if (title.isEmpty()) {
                name.setError("Enter a preset name");
                return;
            }
            FocusTimer.Plan plan = readPlan(minutes, pause, rounds, title);
            if (plan == null) return;
            focus.save(plan);
            dialog.dismiss();
            render();
        });
    }

    private FocusTimer.Plan readPlan(EditText minutes, EditText pause, EditText rounds, String name) {
        int duration = FocusTimer.parse(minutes.getText().toString(), 1, FocusTimer.MAX_MINUTES);
        if (duration < 0) {
            minutes.setError("Enter 1 to 720 minutes");
            minutes.requestFocus();
            return null;
        }
        int rest = pause == null ? 0 : FocusTimer.parse(pause.getText().toString(), 0, 120);
        if (rest < 0) {
            pause.setError("Enter 0 to 120 minutes");
            return null;
        }
        int count = rounds == null ? 1 : FocusTimer.parse(rounds.getText().toString(), 1, 12);
        if (count < 0) {
            rounds.setError("Enter 1 to 12 rounds");
            return null;
        }
        return new FocusTimer.Plan(name, duration, rest, count);
    }

    private boolean startFocus(EditText minutes, EditText task, EditText pause, EditText rounds, String name) {
        FocusTimer.Plan plan = readPlan(minutes, pause, rounds, name);
        if (plan == null) return false;
        beginFocus(plan, task.getText().toString());
        return true;
    }

    private void beginFocus(FocusTimer.Plan plan, String task) {
        focus.start(plan, task.trim());
        hideKeyboard();
        render();
        enter();
    }

    private void showFocusHistory() {
        JSONArray sessions = focus.history();
        if (sessions.length() == 0) return;
        LinearLayout history = card("Recent focus sessions");
        for (int i = 0; i < Math.min(sessions.length(), 10); i++) {
            JSONObject session = sessions.optJSONObject(i);
            if (session == null) continue;
            String task = session.optString("task");
            sessionLine(history, task.isEmpty() ? date(session.optLong("date")) : task
                + (session.optBoolean("taskDone") ? " · Done" : ""), sessionDuration(session.optLong("duration")));
        }
    }

    private void updateFocus() {
        if (focusTime == null) return;
        FocusTimer.Session session = focus.current();
        if (session == null) return;
        if (session.completed() != focusCompleted) {
            render();
            enter();
            return;
        }
        long remaining = (session.remaining() + 999) / 1000;
        String display = session.completed() ? session.minutes() + " min"
            : remaining >= 3600 ? String.format(Locale.getDefault(), "%d:%02d:%02d", remaining / 3600, remaining / 60 % 60, remaining % 60)
            : String.format(Locale.getDefault(), "%02d:%02d", remaining / 60, remaining % 60);
        if (!display.contentEquals(focusTime.getText())) focusTime.setText(display);
        int progress = (int) ((session.duration() - session.remaining()) * 1000 / session.duration());
        focusProgress.setProgress(progress, ValueAnimator.areAnimatorsEnabled());
        if (!session.completed() && !session.paused()) ui.postDelayed(tickFocus, 500);
    }

    private void showPrivacy() {
        LinearLayout panel = card("Activity privacy");
        text(panel, "Activity is saved on this phone. Collection runs when you open or refresh Digital Discipline.", 14, MUTED, false);
        try (UsageHistory history = new UsageHistory(this)) {
            long now = System.currentTimeMillis();
            long pause = history.pausedUntil();
            if (pause > now) {
                badge(panel, "Paused until " + date(pause), PEACH);
                action(panel, "Resume collection", () -> updateHistory(store -> store.resume(System.currentTimeMillis())));
            } else {
                action(panel, "Pause collection", () -> new AlertDialog.Builder(this).setTitle("Pause collection")
                    .setItems(new String[]{"1 hour", "Until tomorrow", "24 hours"}, (dialog, index) -> {
                        long current = System.currentTimeMillis();
                        long length = switch (index) {
                            case 0 -> ActivityReport.HOUR;
                            case 1 -> ActivityReport.midnight(java.time.LocalDate.now().plusDays(1), java.time.ZoneId.systemDefault()) - current;
                            default -> UsageHistory.DAY;
                        };
                        updateHistory(store -> store.pause(current, length));
                    }).show());
            }
            int days = history.retention();
            action(panel, "Keep history for " + days + " days", () -> new AlertDialog.Builder(this)
                .setTitle("History retention")
                .setItems(new String[]{"7 days", "14 days", "30 days"}, (dialog, index) -> {
                    int selected = new int[]{7, 14, 30}[index];
                    updateHistory(store -> store.retention(selected, System.currentTimeMillis()));
                }).show());
        }
        action(panel, "Private apps", this::privateApps);
        action(panel, "Delete history", () -> new AlertDialog.Builder(this).setTitle("Delete activity")
            .setItems(new String[]{"Today", "Last 7 days", "All history"}, (dialog, index) -> {
                new AlertDialog.Builder(this).setTitle("Permanently delete activity?")
                    .setMessage("The selected activity will be removed from this phone.")
                    .setNegativeButton("Cancel", null).setPositiveButton("Delete", (confirm, which) -> {
                        long now = System.currentTimeMillis();
                        updateHistory(store -> {
                            if (index == 2) store.deleteAll(now);
                            else store.deleteSince(ActivityReport.midnight(java.time.LocalDate.now().minusDays(index == 1 ? 6 : 0),
                                java.time.ZoneId.systemDefault()), now);
                        });
                    }).show();
            }).show());
        LinearLayout health = card("Data freshness");
        try (UsageHistory history = new UsageHistory(this)) {
            long now = System.currentTimeMillis();
            long checkpoint = history.checkpoint();
            String label = history.pausedUntil() > now ? "Collection paused"
                : checkpoint <= 0 || checkpoint > now ? "No recent refresh"
                : "Last collection check " + date(checkpoint);
            text(health, label, 14, MUTED, false);
        }
        action(health, "How usage is measured", this::showCoverage);
    }

    private void updateHistory(java.util.function.Consumer<UsageHistory> change) {
        try (UsageHistory store = new UsageHistory(this)) {
            synchronized (ActivityReport.class) {
                change.accept(store);
            }
        } catch (RuntimeException error) {
            status("Could not update saved activity.");
        }
        render();
    }

    private void privateApps() {
        LinearLayout panel = new LinearLayout(this);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setPadding(dp(18), dp(8), dp(18), dp(16));
        EditText search = field(panel, "Search installed apps", InputType.TYPE_CLASS_TEXT);
        ScrollView scroll = new ScrollView(this);
        LinearLayout list = new LinearLayout(this);
        list.setOrientation(LinearLayout.VERTICAL);
        scroll.addView(list);
        panel.addView(scroll, new LinearLayout.LayoutParams(-1, Math.min(dp(360), getResources().getDisplayMetrics().heightPixels / 3)));
        List<Map.Entry<String, String>> apps = new ArrayList<>(AppCatalog.names(this).entrySet());
        try (UsageHistory store = new UsageHistory(this)) {
            for (String id : store.exclusions()) {
                if (apps.stream().noneMatch(entry -> entry.getKey().equals(id))) apps.add(new java.util.AbstractMap.SimpleImmutableEntry<>(id, id));
            }
        }
        apps.sort(Map.Entry.comparingByValue(String.CASE_INSENSITIVE_ORDER));
        AlertDialog dialog = new AlertDialog.Builder(this).setTitle("Private apps").setView(panel)
            .setPositiveButton("Done", null).create();
        Runnable filter = () -> {
            list.removeAllViews();
            String query = search.getText().toString().trim().toLowerCase(Locale.ROOT);
            for (var app : apps) {
                String id = app.getKey();
                String name = app.getValue();
                if (!id.toLowerCase(Locale.ROOT).contains(query) && !name.toLowerCase(Locale.ROOT).contains(query)) continue;
                LinearLayout row = new LinearLayout(this);
                row.setGravity(Gravity.CENTER_VERTICAL);
                row.setMinimumHeight(dp(60));
                list.addView(row, new LinearLayout.LayoutParams(-1, -2));
                row.addView(activityIcon(id), new LinearLayout.LayoutParams(dp(30), dp(30)));
                TextView label = text(row, name + "\n" + id, 14, WHITE, false);
                label.setPadding(dp(10), dp(8), dp(8), dp(8));
                label.setLayoutParams(new LinearLayout.LayoutParams(0, -2, 1));
                row.setOnClickListener(view -> {
                    dialog.dismiss();
                    privateApp(id, name);
                });
                row.setFocusable(true);
                divider(list);
            }
            if (list.getChildCount() == 0) text(list, "No matching apps.", 14, MUTED, false);
        };
        search.addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            @Override public void onTextChanged(CharSequence value, int start, int before, int count) { filter.run(); }
            @Override public void afterTextChanged(Editable value) {}
        });
        filter.run();
        dialog.show();
    }

    private void privateApp(String id, String name) {
        boolean excluded;
        try (UsageHistory store = new UsageHistory(this)) {
            excluded = store.exclusions().contains(id);
        }
        new AlertDialog.Builder(this).setTitle(name)
            .setItems(new String[]{excluded ? "Include future activity and exports" : "Exclude future activity and exports", "Delete saved history"},
                (dialog, index) -> {
                    if (index == 0) {
                        updateHistory(store -> {
                            store.exclude(id, !excluded);
                            store.resetCoverageAt(System.currentTimeMillis());
                        });
                        return;
                    }
                    new AlertDialog.Builder(this).setTitle("Delete " + name + " history?")
                        .setNegativeButton("Cancel", null).setPositiveButton("Delete",
                            (confirm, which) -> updateHistory(store -> store.deleteApp(id, System.currentTimeMillis()))).show();
                }).show();
    }

    private void showActivity(JSONObject report) {
        boolean parent = Rules.PARENT.equals(rules.mode());
        java.time.ZoneId zone = reportZone(report);
        if (sample) badge(root, "Sample data", ACCENT);
        if (detailApp != null) {
            showAppDetails(report, zone);
            return;
        }
        activityArtwork(range == WEEK ? R.drawable.activity_week : R.drawable.activity_today, null);
        LinearLayout ranges = new LinearLayout(this);
        ranges.setBackground(surface(0xff1a1522));
        ranges.setPadding(dp(3), dp(3), dp(3), dp(3));
        LinearLayout.LayoutParams rangeSize = new LinearLayout.LayoutParams(-1, -2);
        rangeSize.topMargin = dp(12);
        root.addView(ranges, rangeSize);
        rangeButton(ranges, "Today", TODAY);
        rangeButton(ranges, "Last 7 Days", WEEK);
        rangeButton(ranges, "Last 24h", ROLLING);
        if (Rules.INDIVIDUAL.equals(rules.mode()) && range == TODAY) {
            action(root, focus.current() == null ? "Start " + focus.plan().minutes() + " min focus" : "Open focus session", () -> {
                if (focus.current() == null) focus.start(focus.plan(), "");
                tab = FOCUS;
                render();
                enter();
            });
        }
        if (!parent && !sample) {
            if (!report.optBoolean("access")) {
                LinearLayout setup = card("Usage Access needed");
                text(setup, "Grant access to refresh your activity. Saved observations are still available.", 14, MUTED, false);
                primary(setup, "Open Usage Access", () -> launchSettings(Settings.ACTION_USAGE_ACCESS_SETTINGS));
            }
            long paused = report.optLong("pausedUntil");
            if (paused > System.currentTimeMillis()) badge(root, "Collection paused until " + date(paused), PEACH);
        }
        if (!report.optBoolean("tracked")) {
            if (parent) primary(card("No activity report"), "Import child activity", () -> openFile("activity"));
            if (Rules.INDIVIDUAL.equals(rules.mode())) shortFormCard(report);
            return;
        }
        long end = report.optLong("at");
        if (parent) text(root, "Report from " + date(end), 13, MUTED, false);
        if (range == WEEK && selectedDay == 0) {
            showWeek(report.optJSONArray("week"));
        } else {
            if (range == WEEK) action(root, "‹ Last 7 Days", () -> {
                selectedDay = 0;
                render();
                enter();
            });
            showToday(report, zone, end);
        }
        if (Rules.INDIVIDUAL.equals(rules.mode())) shortFormCard(report);
        if (!parent && !sample) {
            long refreshed = report.optLong("lastCollected");
            if (refreshed > 0) text(root, "Last collected " + date(refreshed), 13, MUTED, false);
            action(root, "Refresh activity", this::render);
            action(root, "How usage is measured", this::showCoverage);
            if (Rules.INDIVIDUAL.equals(rules.mode())) action(root, "Export activity summary", this::exportSummary);
        }
    }

    private void exitSample() {
        sample = false;
        detailApp = null;
        range = TODAY;
        tab = ACTIVITY;
        render();
        enter();
    }

    private void showCoverage() {
        new AlertDialog.Builder(this).setTitle("Activity coverage")
            .setMessage("Digital Discipline imports recent Android foreground events when you open or refresh the app. Open it regularly to retain more history. A blank chart period means there was no observation. A partial period shows only its recorded time. Split-screen and picture-in-picture are estimates. Paused collection and denied access can leave gaps.")
            .setPositiveButton("Done", null).show();
    }

    private java.time.ZoneId reportZone(JSONObject report) {
        try {
            return java.time.ZoneId.of(report.optString("zone", java.time.ZoneId.systemDefault().getId()));
        } catch (java.time.DateTimeException ignored) {
            return java.time.ZoneId.systemDefault();
        }
    }

    private void activityArtwork(int drawable, String appId) {
        FrameLayout art = new FrameLayout(this);
        art.setBackground(surface(0xff15111c));
        art.setClipToOutline(true);
        root.addView(art, new LinearLayout.LayoutParams(-1, dp(134)));
        ImageView image = new ImageView(this);
        image.setImageResource(drawable);
        image.setScaleType(ImageView.ScaleType.CENTER_CROP);
        image.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        art.addView(image, new FrameLayout.LayoutParams(-1, -1));
        if (ValueAnimator.areAnimatorsEnabled()) {
            image.setScaleX(1.08f);
            image.setScaleY(1.08f);
            image.animate().scaleX(1).scaleY(1).setDuration(520).start();
        }
        if (appId == null) return;
        FrameLayout.LayoutParams center = new FrameLayout.LayoutParams(dp(49), dp(49), Gravity.CENTER);
        center.topMargin = dp(13);
        art.addView(activityIcon(appId), center);
    }

    private void showWeek(JSONArray week) {
        LinearLayout panel = card("Activity heatmap");
        if (week == null || week.length() != 7) {
            text(panel, "Weekly data is unavailable in this report.", 14, MUTED, false);
            return;
        }
        panel.addView(new WeekChart(this, week, this::sessionDuration, day -> {
            selectedDay = day;
            render();
            enter();
        }));
    }

    private long rangeStart(JSONObject report) {
        if (range == WEEK) return selectedDay;
        return range == ROLLING ? report.optLong("from") : report.optLong("todayStart", report.optLong("from"));
    }

    private long rangeEnd(JSONObject report, java.time.ZoneId zone, long end) {
        if (range != WEEK) return end;
        return Math.min(end, java.time.Instant.ofEpochMilli(selectedDay).atZone(zone)
            .toLocalDate().plusDays(1).atStartOfDay(zone).toInstant().toEpochMilli());
    }

    private Map<String, Long> appTotals(JSONObject report, long from, long to) {
        Map<String, Long> totals = new java.util.HashMap<>();
        JSONArray timeline = report.optJSONArray("sessions");
        if (timeline == null) return totals;
        for (int i = 0; i < timeline.length(); i++) {
            JSONObject session = timeline.optJSONObject(i);
            if (session == null) continue;
            long usage = ActivityReport.overlap(from, to, session.optLong("from"), session.optLong("to"));
            if (usage > 0) totals.merge(session.optString("id"), usage, Long::sum);
        }
        return totals;
    }

    private void showToday(JSONObject report, java.time.ZoneId zone, long end) {
        JSONArray apps = report.optJSONArray("apps");
        if (apps == null) return;
        long from = rangeStart(report);
        long to = rangeEnd(report, zone, end);
        boolean modern = report.has("rollingHours");
        String durationField = range == ROLLING || !report.has("todayStart") ? "time" : "today";
        LinearLayout summary = card("Screen time by hour");
        summary.setBackground(glass(0xff272038, 0xff1e2930, 0xff625375));
        TextView interval = text(summary, date(from, zone) + " – " + date(to, zone), 13, MUTED, false);
        if (!modern && range == ROLLING) {
            text(summary, "A rolling 24-hour breakdown is unavailable in this report.", 14, MUTED, false);
            return;
        }
        JSONArray hours = report.optJSONArray(range == ROLLING ? "rollingHours" : "hours");
        long firstHour = report.optLong("hourStart");
        if (range == WEEK) {
            hours = null;
            JSONArray week = report.optJSONArray("week");
            for (int i = 0; week != null && i < week.length(); i++) {
                JSONObject day = week.optJSONObject(i);
                if (day != null && day.optLong("start") == from) hours = day.optJSONArray("bins");
            }
            firstHour = from;
        }
        Map<String, String> labels = new java.util.HashMap<>();
        List<JSONObject> used = new ArrayList<>();
        Map<String, Long> totals = range == WEEK ? appTotals(report, from, to) : java.util.Collections.emptyMap();
        long total = 0;
        if (range == WEEK && report.optBoolean("summaryOnly")) {
            for (int i = 0; hours != null && i < hours.length(); i++) total += hours.optLong(i);
        }
        for (int i = 0; i < apps.length(); i++) {
            JSONObject app = apps.optJSONObject(i);
            if (app == null) continue;
            String id = app.optString("id");
            labels.put(id, app.optString("name", id));
            long amount = range == WEEK ? totals.getOrDefault(id, 0L) : app.optLong(durationField);
            if (amount > 0) {
                total += amount;
                try { used.add(new JSONObject().put("id", id).put("name", labels.get(id)).put("usage", amount)); }
                catch (org.json.JSONException impossible) { throw new IllegalStateException(impossible); }
            }
        }
        long observed = report.optLong(range == ROLLING ? "rollingObserved" : "todayObserved", -1);
        boolean anyBins = false;
        for (int i = 0; hours != null && i < hours.length(); i++) if (!hours.isNull(i)) anyBins = true;
        if (modern && !anyBins) {
            interval.setText(interval.getText() + "\nNo observation for this period");
        } else {
            text(summary, sessionDuration(total), 35, WHITE, true);
            if (modern && range != WEEK && observed >= 0 && observed < to - from) {
                text(summary, "Partial coverage", 13, PEACH, false);
            }
        }
        if (hours != null) {
            HourChart chart = new HourChart(this, hours, firstHour, from, to, zone);
            summary.addView(chart, new LinearLayout.LayoutParams(-1, dp(114)));
            TextView detail = text(summary, "Tap an hour to see usage", 13, MUTED, false);
            chart.setOnHourSelected((label, start, stop, millis) -> {
                if (millis < 0) {
                    detail.setText(label + " · No observation");
                    return;
                }
                List<String> contributions = new ArrayList<>();
                appTotals(report, start, stop).entrySet().stream()
                    .sorted(Map.Entry.<String, Long>comparingByValue().reversed())
                    .forEach(entry -> contributions.add(labels.getOrDefault(entry.getKey(), entry.getKey())
                        + " · " + sessionDuration(entry.getValue())));
                detail.setText(label + " · " + sessionDuration(millis)
                    + (contributions.isEmpty() ? "" : "\n" + String.join("\n", contributions)));
            });
        }
        if (range == WEEK && report.optBoolean("summaryOnly")) return;
        LinearLayout ranking = card("Top apps");
        if (used.isEmpty()) {
            text(ranking, modern && !anyBins ? "Activity will appear after a recorded app visit."
                : "No foreground app use recorded in this period.", 14, MUTED, false);
            return;
        }
        used.sort((a, b) -> Long.compare(b.optLong("usage"), a.optLong("usage")));
        long most = used.get(0).optLong("usage");
        for (JSONObject app : used) addRankedApp(ranking, app, "usage", most);
        if (!report.has("sessions")) showOpenHistory(report.optJSONArray("opens"), labels, from, zone);
    }

    private void addRankedApp(LinearLayout parent, JSONObject app, String field, long most) {
        String id = app.optString("id");
        String name = app.optString("name", id);
        long elapsed = app.optLong(field);
        LinearLayout row = new LinearLayout(this);
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setMinimumHeight(dp(52));
        row.setPadding(dp(4), 0, dp(4), 0);
        row.setBackground(ripple(surface(0xff211a2b)));
        row.setContentDescription(name + ", " + sessionDuration(elapsed) + ", usage details");
        row.setFocusable(true);
        row.setTooltipText(name);
        row.addView(activityIcon(id), new LinearLayout.LayoutParams(dp(30), dp(30)));
        usageBar(row, elapsed, most);
        TextView time = text(row, sessionDuration(elapsed), 14, WHITE, true);
        time.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        time.setPadding(0, 0, dp(4), 0);
        time.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        row.setOnClickListener(view -> {
            detailApp = id;
            render();
            enter();
        });
        LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(-1, -2);
        size.bottomMargin = dp(5);
        parent.addView(row, size);
    }

    private View activityIcon(String id) {
        android.graphics.drawable.Drawable drawable = AppCatalog.icon(this, id);
        if (drawable == null) return new BrandMark(this, false);
        ImageView image = new ImageView(this);
        image.setImageDrawable(drawable);
        image.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        return image;
    }

    private void showAppDetails(JSONObject report, java.time.ZoneId zone) {
        action(root, "‹ Activity", this::closeAppDetails);
        activityArtwork(R.drawable.activity_session, detailApp);
        String name = detailApp;
        long fallbackTotal = 0;
        JSONArray apps = report.optJSONArray("apps");
        if (apps != null) {
            for (int i = 0; i < apps.length(); i++) {
                JSONObject app = apps.optJSONObject(i);
                if (app != null && detailApp.equals(app.optString("id"))) {
                    name = app.optString("name", detailApp);
                    fallbackTotal = app.optLong(range == ROLLING || !report.has("todayStart") ? "time" : "today");
                    break;
                }
            }
        }
        title(name, report.has("todayStart") ? "Usage sessions" : "Recorded usage");
        LinearLayout list = card("Opened · Ended");
        long total = 0;
        int openings = 0;
        long start = rangeStart(report);
        long end = rangeEnd(report, zone, report.optLong("at"));
        JSONArray sessions = report.optBoolean("summaryOnly") ? null : report.optJSONArray("sessions");
        java.time.format.DateTimeFormatter clock = java.time.format.DateTimeFormatter
            .ofLocalizedTime(java.time.format.FormatStyle.SHORT).withZone(zone);
        if (sessions != null) {
            for (int i = 0; i < sessions.length(); i++) {
                JSONObject item = sessions.optJSONObject(i);
                if (item == null || !detailApp.equals(item.optString("id"))) continue;
                long from = Math.max(start, item.optLong("from"));
                long to = Math.min(end, item.optLong("to"));
                if (to <= from) continue;
                long elapsed = to - from;
                total += elapsed;
                if (item.optBoolean("opened") && item.optLong("from") >= start) openings++;
                String finish = item.optBoolean("ongoing") && to == report.optLong("at") ? "In progress" : clock.format(java.time.Instant.ofEpochMilli(to));
                sessionLine(list, clock.format(java.time.Instant.ofEpochMilli(from)) + " – " + finish, sessionDuration(elapsed));
                divider(list);
            }
        } else {
            total = fallbackTotal;
            text(list, "Session times are unavailable in this report.", 14, MUTED, false);
        }
        if (sessions != null && total == 0) text(list, "No sessions recorded for this app.", 14, MUTED, false);
        if (report.has("rollingHours") && sessions != null) sessionLine(list, "Observed openings", Integer.toString(openings));
        sessionLine(list, "Total usage", sessionDuration(total));
    }

    private void sessionLine(LinearLayout list, String label, String value) {
        LinearLayout row = new LinearLayout(this);
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setMinimumHeight(dp(49));
        list.addView(row, new LinearLayout.LayoutParams(-1, -2));
        TextView times = text(row, label, 14, WHITE, false);
        times.setLayoutParams(new LinearLayout.LayoutParams(0, -2, 1));
        times.setPadding(0, 0, dp(6), 0);
        TextView duration = text(row, value, 14, ACCENT, true);
        duration.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        duration.setPadding(0, 0, 0, 0);
    }

    private String sessionDuration(long millis) {
        long seconds = millis / 1000;
        if (millis > 0 && seconds == 0) return "<1s";
        String main = duration(millis);
        return seconds >= 60 && seconds % 60 != 0 ? main + " " + seconds % 60 + "s" : main;
    }

    private void closeAppDetails() {
        detailApp = null;
        render();
        enter();
    }

    private void rangeButton(LinearLayout bar, String title, int period) {
        Button item = button(title, () -> {
            if (range == period && selectedDay == 0) return;
            range = period;
            selectedDay = 0;
            render();
            enter();
        }, range == period);
        item.setSelected(range == period);
        item.setPadding(dp(4), dp(6), dp(4), dp(6));
        if (range == WEEK && period == WEEK) {
            item.setBackground(ripple(surface(0xffb4ead5)));
            item.setTextColor(0xff102920);
        }
        bar.addView(item, new LinearLayout.LayoutParams(0, -2, 1));
    }

    private LinearLayout activityRow(LinearLayout parent, String id, String name, String value) {
        LinearLayout row = new LinearLayout(this);
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setMinimumHeight(dp(48));
        parent.addView(row, new LinearLayout.LayoutParams(-1, -2));
        android.graphics.drawable.Drawable icon = AppCatalog.icon(this, id);
        if (icon != null) {
            ImageView image = new ImageView(this);
            image.setImageDrawable(icon);
            image.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            row.addView(image, new LinearLayout.LayoutParams(dp(32), dp(32)));
        } else {
            row.addView(new BrandMark(this, false), new LinearLayout.LayoutParams(dp(32), dp(32)));
        }
        TextView title = text(row, name, 16, WHITE, true);
        title.setLayoutParams(new LinearLayout.LayoutParams(0, -2, 1));
        title.setPadding(dp(9), 0, dp(8), 0);
        TextView trailing = text(row, value, 14, 0xffdec9fc, false);
        trailing.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        trailing.setPadding(0, 0, 0, 0);
        return row;
    }

    private void showOpenHistory(JSONArray opens, Map<String, String> labels, long from, java.time.ZoneId zone) {
        if (opens == null) return;
        Map<java.time.LocalDate, Map<String, List<Long>>> days = new java.util.LinkedHashMap<>();
        for (int i = opens.length() - 1; i >= 0; i--) {
            JSONObject event = opens.optJSONObject(i);
            if (event == null || event.optLong("time") < from) continue;
            long time = event.optLong("time");
            java.time.LocalDate day = java.time.Instant.ofEpochMilli(time).atZone(zone).toLocalDate();
            days.computeIfAbsent(day, ignored -> new java.util.LinkedHashMap<>())
                .computeIfAbsent(event.optString("id"), ignored -> new ArrayList<>()).add(time);
        }
        if (days.isEmpty()) return;
        LinearLayout history = card("App-open history");
        java.time.format.DateTimeFormatter dayLabel = java.time.format.DateTimeFormatter
            .ofLocalizedDate(java.time.format.FormatStyle.MEDIUM);
        java.time.format.DateTimeFormatter timeLabel = java.time.format.DateTimeFormatter
            .ofLocalizedTime(java.time.format.FormatStyle.SHORT).withZone(zone);
        days.forEach((day, groups) -> {
            text(history, day.equals(java.time.LocalDate.now(zone)) ? "Today" : dayLabel.format(day),
                14, MUTED, true);
            groups.forEach((id, times) -> {
                String count = getResources().getQuantityString(R.plurals.app_opens, times.size(), times.size());
                LinearLayout row = activityRow(history, id, labels.getOrDefault(id, id), count + "  ⌄");
                row.setBackground(ripple(surface(0xff282039)));
                row.setPadding(dp(8), 0, dp(8), 0);
                row.setFocusable(true);
                LinearLayout details = new LinearLayout(this);
                details.setOrientation(LinearLayout.VERTICAL);
                details.setPadding(dp(49), dp(8), 0, 0);
                details.setVisibility(View.GONE);
                history.addView(details, new LinearLayout.LayoutParams(-1, -2));
                row.setOnClickListener(view -> {
                    boolean expand = details.getVisibility() != View.VISIBLE;
                    if (expand && details.getChildCount() == 0) {
                        for (long time : times) {
                            text(details, timeLabel.format(java.time.Instant.ofEpochMilli(time)), 14, WHITE, false);
                        }
                    }
                    if (ValueAnimator.areAnimatorsEnabled()) {
                        android.transition.TransitionManager.beginDelayedTransition(history,
                            new android.transition.AutoTransition().setDuration(180));
                    }
                    details.setVisibility(expand ? View.VISIBLE : View.GONE);
                    ((TextView) row.getChildAt(2)).setText(count + (expand ? "  ⌃" : "  ⌄"));
                });
                View gap = new View(this);
                history.addView(gap, new LinearLayout.LayoutParams(1, dp(6)));
            });
        });
    }

    private GradientDrawable glass(int top, int bottom, int edge) {
        GradientDrawable layer = new GradientDrawable(GradientDrawable.Orientation.TL_BR,
            new int[]{top, bottom});
        layer.setCornerRadius(dp(22));
        layer.setStroke(dp(1), edge);
        return layer;
    }

    private void animateLaunch() {
        if (!ValueAnimator.areAnimatorsEnabled() || stage == null) return;
        FrameLayout host = stage;
        LinearLayout content = frame;
        BrandMark destination = heroMark;
        FrameLayout overlay = new FrameLayout(this);
        overlay.setBackgroundColor(0xff0d0915);
        overlay.setClickable(true);
        host.addView(overlay, new FrameLayout.LayoutParams(-1, -1));
        BrandMark splash = new BrandMark(this, true);
        int size = dp(155);
        overlay.addView(splash, new FrameLayout.LayoutParams(size, size, Gravity.CENTER));
        if (destination != null) destination.setAlpha(0);
        host.getViewTreeObserver().addOnPreDrawListener(new android.view.ViewTreeObserver.OnPreDrawListener() {
            @Override public boolean onPreDraw() {
                host.getViewTreeObserver().removeOnPreDrawListener(this);
                if (host == stage) animateMark(host, content, destination, overlay, splash, size);
                return true;
            }
        });
    }

    private void animateMark(FrameLayout host, LinearLayout content, BrandMark destination,
                             FrameLayout overlay, BrandMark splash, int size) {
        int[] start = new int[2];
        int[] end = new int[2];
        splash.getLocationInWindow(start);
        if (destination != null) destination.getLocationInWindow(end);
        float dx = destination == null ? 0 : end[0] + destination.getWidth() / 2f - start[0] - size / 2f;
        float dy = destination == null ? -dp(25) : end[1] + destination.getHeight() / 2f - start[1] - size / 2f;
        float scale = destination == null ? 0.85f
            : (float) Math.min(destination.getWidth(), destination.getHeight()) / size;
        ValueAnimator fade = ValueAnimator.ofInt(255, 0);
        fade.setStartDelay(65);
        fade.setDuration(355);
        fade.addUpdateListener(step -> overlay.getBackground().setAlpha((int) step.getAnimatedValue()));
        fade.start();
        content.setAlpha(.58f);
        content.animate().alpha(1).setDuration(420).start();
        splash.animate().translationX(dx).translationY(dy).scaleX(scale).scaleY(scale)
            .setDuration(420).withEndAction(() -> {
                if (destination != null) destination.setAlpha(1);
                host.removeView(overlay);
            }).start();
    }

    private void divider(LinearLayout parent) {
        View line = new View(this);
        line.setBackgroundColor(BORDER);
        parent.addView(line, new LinearLayout.LayoutParams(-1, dp(1)));
    }

    private void usageBar(LinearLayout parent, long time, long total) {
        android.widget.ProgressBar bar = new android.widget.ProgressBar(this, null,
            android.R.attr.progressBarStyleHorizontal);
        bar.setMax(1000);
        int progress = (int) Math.max(1, (time * 1000) / total);
        if (ValueAnimator.areAnimatorsEnabled()) bar.post(() -> bar.setProgress(progress, true));
        else bar.setProgress(progress);
        bar.setProgressTintList(ColorStateList.valueOf(ACCENT));
        bar.setProgressBackgroundTintList(ColorStateList.valueOf(BORDER));
        bar.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(0, dp(7), 1);
        layout.setMargins(dp(10), 0, dp(10), 0);
        parent.addView(bar, layout);
    }

    private void exportSummary() {
        new AlertDialog.Builder(this).setTitle("Export activity summary")
            .setMessage("Includes app names and recorded totals for Today and Last 24h, plus hourly totals for the last seven days. Private apps and individual session times are excluded.")
            .setNegativeButton("Cancel", null).setPositiveButton("Choose file", (dialog, which) -> {
                try {
                    JSONObject report = ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), false);
                    pendingSummary = report.toString(2);
                    startActivityForResult(new Intent(Intent.ACTION_CREATE_DOCUMENT)
                        .addCategory(Intent.CATEGORY_OPENABLE).setType("application/json")
                        .putExtra(Intent.EXTRA_TITLE, "digital-discipline-summary.json"), WRITE_SUMMARY);
                } catch (Exception error) {
                    pendingSummary = null;
                    status("Could not export the summary.");
                }
            }).show();
    }

    private void exportActivity() {
        if (busy) return;
        if (!unlocked || !Rules.showChildNotice(this)) {
            status("Allow the supervision notification before exporting.");
            return;
        }
        busy = true;
        new Thread(() -> {
            try {
                JSONObject report = ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), false);
                String packet = rules.pack("activity", report);
                runOnUiThread(() -> {
                    busy = false;
                    if (unlocked && Rules.showChildNotice(this)) saveFile("activity", packet);
                });
            } catch (Exception e) {
                runOnUiThread(() -> {
                    busy = false;
                    status("Could not create the activity file.");
                });
            }
        }, "activity-report").start();
    }

    private void saveFile(String kind, String packet) {
        if (packet.length() > MAX_PACKET) {
            status("The activity file exceeds the prototype size limit.");
            return;
        }
        pendingPacket = packet;
        Intent create = new Intent(Intent.ACTION_CREATE_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE)
            .setType("application/octet-stream").putExtra(Intent.EXTRA_TITLE, "digital-discipline-" + kind + ".dd");
        startActivityForResult(create, WRITE_FILE);
    }

    private void openFile(String kind) {
        pendingReadKind = kind;
        Intent open = new Intent(Intent.ACTION_OPEN_DOCUMENT).addCategory(Intent.CATEGORY_OPENABLE).setType("*/*");
        startActivityForResult(open, READ_FILE);
    }

    @Override protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request == WRITE_SUMMARY) {
            String summary = pendingSummary;
            pendingSummary = null;
            if (result == RESULT_OK && data != null && data.getData() != null && summary != null) {
                try (OutputStream output = getContentResolver().openOutputStream(data.getData(), "wt")) {
                    if (output == null) throw new java.io.IOException();
                    output.write(summary.getBytes(StandardCharsets.UTF_8));
                    status("Activity summary saved.");
                } catch (java.io.IOException | SecurityException error) {
                    status("Could not save the activity summary.");
                }
            }
            return;
        }
        String packet = pendingPacket;
        String readKind = pendingReadKind;
        if (request == WRITE_FILE) pendingPacket = null;
        if (request == READ_FILE) pendingReadKind = null;
        if (result != RESULT_OK || data == null || data.getData() == null) return;
        try {
            if (request == WRITE_FILE && packet != null) {
                try (OutputStream output = getContentResolver().openOutputStream(data.getData(), "wt")) {
                    if (output == null) throw new IllegalStateException();
                    output.write(packet.getBytes(StandardCharsets.UTF_8));
                }
                status("Encrypted file saved.");
            } else if (request == READ_FILE && readKind != null) {
                try (InputStream input = getContentResolver().openInputStream(data.getData())) {
                    if (input == null) throw new IllegalStateException();
                    ByteArrayOutputStream bytes = new ByteArrayOutputStream();
                    byte[] chunk = new byte[8192];
                    int size;
                    while ((size = input.read(chunk)) != -1) {
                        if (bytes.size() + size > MAX_PACKET) throw new IllegalArgumentException();
                        bytes.write(chunk, 0, size);
                    }
                    accept(readKind, rules.unpack(readKind, new String(bytes.toByteArray(), StandardCharsets.UTF_8)));
                }
            }
        } catch (Exception e) {
            status("File unavailable, invalid, or from a different pairing.");
        }
    }

    private void accept(String kind, JSONObject content) throws Exception {
        if ("activity".equals(kind) && Rules.PARENT.equals(rules.mode())) {
            long time = content.getLong("at");
            content.getJSONArray("apps");
            content.getJSONArray("opens");
            if (time <= rules.prefs.getLong("report_time", 0)) {
                status("A newer or identical report is already imported.");
                return;
            }
            rules.prefs.edit().putLong("report_time", time).putString("report", content.toString()).apply();
            tab = ACTIVITY;
            status("Child activity imported.");
        }
    }

    private void launchSettings(String action) {
        try {
            startActivity(new Intent(action));
        } catch (android.content.ActivityNotFoundException e) {
            status("This Android settings screen is unavailable.");
        }
    }

    private String date(long time) {
        return DateFormat.getDateTimeInstance(DateFormat.SHORT, DateFormat.SHORT).format(new Date(time));
    }

    private String date(long time, java.time.ZoneId zone) {
        return java.time.format.DateTimeFormatter.ofLocalizedDateTime(java.time.format.FormatStyle.SHORT)
            .withZone(zone).format(java.time.Instant.ofEpochMilli(time));
    }

    private String duration(long millis) {
        long minutes = TimeUnit.MILLISECONDS.toMinutes(millis);
        long seconds = TimeUnit.MILLISECONDS.toSeconds(millis);
        if (seconds < 60) return seconds + "s";
        return (minutes >= 60 ? minutes / 60 + "h " : "") + minutes % 60 + "m";
    }

    private void title(String title, String description) {
        text(root, title, 35, WHITE, true).setLetterSpacing(-0.035f);
        text(root, description, 16, MUTED, false);
    }

    private LinearLayout card(String heading) {
        LinearLayout panel = new LinearLayout(this);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setPadding(dp(18), dp(17), dp(18), dp(17));
        panel.setBackground(surface(PANEL));
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(-1, -2);
        layout.topMargin = dp(14);
        root.addView(panel, layout);
        text(panel, heading, 20, WHITE, true);
        return panel;
    }

    private void navButton(LinearLayout row, String title, int page) {
        boolean selected = tab == page;
        LinearLayout item = new LinearLayout(this);
        item.setGravity(Gravity.CENTER);
        item.setOrientation(LinearLayout.VERTICAL);
        item.setMinimumHeight(dp(60));
        item.setContentDescription(title);
        item.setSelected(selected);
        item.setFocusable(true);
        item.setBackground(ripple(selected ? surface(0xff453358) : null));
        item.addView(new DisciplineArt(this, title), new LinearLayout.LayoutParams(dp(21), dp(21)));
        TextView label = text(item, title, 12, selected ? ACCENT : MUTED, true);
        label.setGravity(Gravity.CENTER);
        label.setPadding(0, dp(4), 0, 0);
        label.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        item.setOnClickListener(view -> {
            if (tab == page) return;
            tab = page;
            detailApp = null;
            render();
            enter();
        });
        LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(0, -2, 1);
        size.setMargins(dp(3), 0, dp(3), 0);
        row.addView(item, size);
    }

    private void backButton(LinearLayout parent, Runnable listener) {
        Button back = button("‹  Back", listener, false);
        LinearLayout.LayoutParams size = new LinearLayout.LayoutParams(-2, -2);
        size.bottomMargin = dp(14);
        parent.addView(back, size);
    }

    private void action(LinearLayout parent, String title, Runnable listener) {
        addButton(parent, title, listener, false);
    }

    private void primary(LinearLayout parent, String title, Runnable listener) {
        addButton(parent, title, listener, true);
    }

    private void addButton(LinearLayout parent, String title, Runnable listener, boolean primary) {
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(-1, -2);
        layout.topMargin = dp(primary ? 14 : 8);
        parent.addView(button(title, listener, primary), layout);
    }

    private Button button(String label, Runnable action, boolean primary) {
        Button button = new Button(this);
        button.setText(label);
        button.setAllCaps(false);
        button.setTypeface(Typeface.create("sans-serif-medium", Typeface.NORMAL));
        button.setTextSize(15);
        button.setMinHeight(dp(primary ? 54 : 48));
        button.setMinWidth(0);
        button.setTextColor(primary ? 0xff221333 : WHITE);
        button.setPadding(dp(14), dp(9), dp(14), dp(9));
        button.setStateListAnimator(null);
        button.setBackground(ripple(surface(primary ? ACCENT : 0xff32263e)));
        button.setOnClickListener(view -> action.run());
        return button;
    }

    private android.graphics.drawable.RippleDrawable ripple(android.graphics.drawable.Drawable surface) {
        GradientDrawable mask = new GradientDrawable();
        mask.setColor(0xffffffff);
        mask.setCornerRadius(dp(19));
        return new android.graphics.drawable.RippleDrawable(ColorStateList.valueOf(0x26fff4ec), surface, mask);
    }

    private EditText field(LinearLayout parent, String hint, int input) {
        FrameLayout wrapper = new FrameLayout(this);
        wrapper.setBackground(surface(BG));
        EditText text = new EditText(this);
        text.setSingleLine(true);
        text.setInputType(input);
        if (input == (InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD)) {
            text.setTransformationMethod(android.text.method.PasswordTransformationMethod.getInstance());
            text.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(12)});
        }
        text.setTextSize(16);
        text.setHint(hint);
        text.setTextColor(WHITE);
        text.setHintTextColor(MUTED);
        text.setPadding(dp(13), dp(12), dp(46), dp(12));
        text.setMinHeight(dp(56));
        text.setBackground(null);
        text.setSaveEnabled(false);
        text.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_NO);
        wrapper.addView(text, new FrameLayout.LayoutParams(-1, -2));
        DisciplineArt pencil = new DisciplineArt(this, "pencil");
        pencil.setAlpha(0.58f);
        FrameLayout.LayoutParams icon = new FrameLayout.LayoutParams(dp(20), dp(20), Gravity.END | Gravity.CENTER_VERTICAL);
        icon.setMarginEnd(dp(13));
        wrapper.addView(pencil, icon);
        LinearLayout.LayoutParams field = new LinearLayout.LayoutParams(-1, -2);
        field.bottomMargin = dp(10);
        parent.addView(wrapper, field);
        text.setOnFocusChangeListener((view, focused) -> {
            GradientDrawable frame = surface(BG);
            if (focused) frame.setStroke(dp(1), ACCENT);
            wrapper.setBackground(frame);
            pencil.setAlpha(focused ? 1f : 0.58f);
        });
        text.addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            @Override public void onTextChanged(CharSequence value, int start, int before, int count) {
                if (!text.hasFocus() || !ValueAnimator.areAnimatorsEnabled()) return;
                pencil.animate().cancel();
                pencil.setTranslationY(-dp(4));
                pencil.setRotation(-15);
                pencil.animate().translationY(0).rotation(0).setDuration(175).start();
            }
            @Override public void afterTextChanged(Editable value) {}
        });
        return text;
    }

    private TextView text(LinearLayout parent, String value, int size, int color, boolean bold) {
        TextView text = new TextView(this);
        text.setText(value);
        text.setTextSize(size);
        text.setTextColor(color);
        text.setPadding(0, 0, 0, dp(8));
        if (bold) text.setTypeface(Typeface.create("sans-serif-medium", Typeface.NORMAL));
        parent.addView(text, new LinearLayout.LayoutParams(-1, -2));
        return text;
    }

    private GradientDrawable surface(int fill) {
        GradientDrawable shape = new GradientDrawable();
        shape.setColor(fill);
        shape.setStroke(dp(1), BORDER);
        shape.setCornerRadius(dp(19));
        return shape;
    }

    private int dp(float n) { return Math.round(n * getResources().getDisplayMetrics().density); }

    private void status(String message) { Toast.makeText(this, message, Toast.LENGTH_LONG).show(); }
}
