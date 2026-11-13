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
import android.widget.Switch;
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
import java.util.HashSet;
import java.util.Set;
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
    private TextView serviceStatus;
    private final Handler ui = new Handler(Looper.getMainLooper());
    private final Runnable refreshServiceStatus = new Runnable() {
        @Override public void run() {
            if (serviceStatus == null) return;
            boolean enabled = Rules.serviceEnabled(MainActivity.this);
            String label = BlockService.isConnected() && enabled ? "Done"
                : enabled ? "Waiting for service" : "Off";
            if (!label.contentEquals(serviceStatus.getText())) {
                setBadge(serviceStatus, label, "Done".equals(label) ? MINT : PEACH);
            }
            ui.postDelayed(this, 850);
        }
    };
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
    private boolean weekly;
    private String detailApp;
    private boolean launchPending;
    private FrameLayout stage;
    private BrandMark heroMark;
    private int tab;
    private static final int ACTIVITY = 0;
    private static final int BLOCKING = 1;
    private static final int SETTINGS = 2;

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        if (Build.VERSION.SDK_INT >= 30) getWindow().setDecorFitsSystemWindows(false);
        rules = new Rules(this);
        launchPending = state == null;
        if (state != null) {
            tab = state.getInt("tab");
            weekly = state.getBoolean("weekly");
            detailApp = state.getString("detailApp");
            choosingMode = state.getBoolean("choosingMode");
        }
    }

    @Override protected void onSaveInstanceState(Bundle out) {
        out.putInt("tab", tab);
        out.putBoolean("weekly", weekly);
        out.putString("detailApp", detailApp);
        out.putBoolean("choosingMode", choosingMode);
        super.onSaveInstanceState(out);
    }

    @Override protected void onResume() {
        super.onResume();
        if (Rules.CHILD.equals(rules.mode())) Rules.showChildNotice(this);
        if (enrollStep == 0) render();
        if (launchPending) {
            launchPending = false;
            animateLaunch();
        }
    }

    @Override protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        if (Intent.ACTION_MAIN.equals(intent.getAction()) && intent.hasCategory(Intent.CATEGORY_LAUNCHER)) {
            launchPending = true;
        }
    }

    @Override protected void onPause() {
        ui.removeCallbacks(refreshServiceStatus);
        super.onPause();
    }

    @Override protected void onStop() {
        super.onStop();
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
        if (mode.isEmpty()) {
            if (enrollStep != 0) renderEnrollment();
            else if (choosingMode) chooseMode();
            else welcome();
            return;
        }
        if (Rules.CHILD.equals(mode) && !unlocked) {
            LinearLayout panel = card("Supervised device");
            panel.setGravity(Gravity.CENTER_HORIZONTAL);
            text(panel, "Your parent manages app rules on this phone.", 16, MUTED, false);
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
        navButton(nav, "Blocking", BLOCKING);
        navButton(nav, "Settings", SETTINGS);
        if (tab == SETTINGS) {
            settings(mode);
            return;
        }
        if (Rules.PARENT.equals(mode)) {
            try {
                JSONObject report = new JSONObject(rules.prefs.getString("report", "{}"));
                if (tab == ACTIVITY) showActivity(report);
                else {
                    showBlocks(report.optJSONArray("apps"), true);
                    if (report.has("apps")) action(root, "Export rules file", this::exportRules);
                }
            } catch (Exception e) {
                text(root, "The saved report could not be opened.", 15, MUTED, false);
            }
        } else if (tab == BLOCKING) {
            if (Rules.CHILD.equals(mode)) {
                LinearLayout box = card("Applied rules");
                Set<String> blocks = rules.blocks();
                text(box, blocks.size() + " blocked apps", 16, WHITE, false);
                action(box, "Import parent rules", () -> openFile("rules"));
            } else {
                showBlocks(localAppList(), false);
            }
        } else {
            try {
                showActivity(ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), true));
            } catch (Exception e) {
                text(root, "Android usage data is currently unavailable.", 15, MUTED, false);
            }
        }
    }

    private void mount() {
        ui.removeCallbacks(refreshServiceStatus);
        serviceStatus = null;
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
        TextView description = text(root, "Digital Discipline helps you see where your time goes and put limits on distractions.", 16, MUTED, false);
        description.setGravity(Gravity.CENTER);
        description.setMaxWidth(dp(330));
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
        modeRow("Individual", "Track your time and manage your own app rules.", "Activity", () -> {
            rules.prefs.edit().putString("mode", Rules.INDIVIDUAL)
                .putLong("enrolled", System.currentTimeMillis()).apply();
            render();
            enter();
        });
        modeRow("Parent phone", "Review activity and set rules for a child device.", "Settings", () -> {
            rules.prefs.edit().putString("mode", Rules.PARENT).putString("key", Rules.newKey()).apply();
            tab = SETTINGS;
            render();
            enter();
        });
        modeRow("Child phone", "Connect to your parent’s Digital Discipline app.", "Blocking", () -> {
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

    private void dashboardHeader(String description) {
        LinearLayout brand = new LinearLayout(this);
        brand.setGravity(Gravity.CENTER_VERTICAL);
        brand.addView(new BrandMark(this, false), new LinearLayout.LayoutParams(dp(27), dp(27)));
        TextView name = text(brand, "Digital Discipline", 16, WHITE, true);
        name.setLayoutParams(new LinearLayout.LayoutParams(-2, -2));
        name.setPadding(dp(5), 0, 0, 0);
        root.addView(brand, new LinearLayout.LayoutParams(-1, dp(36)));
        if (tab == ACTIVITY) {
            text(root, "Activity", 36, WHITE, true).setLetterSpacing(-0.045f);
            if (!"Personal dashboard".equals(description)) text(root, description, 14, MUTED, false);
            return;
        }
        String heading = switch (tab) {
            case BLOCKING -> "App blocking";
            case SETTINGS -> "Settings";
            default -> "Your activity";
        };
        String scene = switch (tab) {
            case BLOCKING -> "Blocking";
            case SETTINGS -> "Settings";
            default -> "Activity";
        };
        int[] colors = switch (tab) {
            case BLOCKING -> new int[]{0xff50293d, 0xff2c203a};
            case SETTINGS -> new int[]{0xff24423e, 0xff252239};
            default -> new int[]{0xff443362, 0xff2c2146};
        };
        LinearLayout hero = new LinearLayout(this);
        hero.setGravity(Gravity.CENTER_VERTICAL);
        hero.setPadding(dp(18), dp(13), dp(7), dp(13));
        GradientDrawable tint = new GradientDrawable(GradientDrawable.Orientation.TL_BR, colors);
        tint.setCornerRadius(dp(24));
        tint.setStroke(dp(1), 0xff665679);
        hero.setBackground(tint);
        LinearLayout words = new LinearLayout(this);
        words.setOrientation(LinearLayout.VERTICAL);
        hero.addView(words, new LinearLayout.LayoutParams(0, -2, 1));
        TextView eyebrow = text(words, description, 13, tab == SETTINGS ? MINT : tab == BLOCKING ? PEACH : ACCENT, true);
        eyebrow.setPadding(0, 0, 0, dp(5));
        text(words, heading, 30, WHITE, true).setLetterSpacing(-0.045f);
        String subtitle = switch (tab) {
            case BLOCKING -> "Choose what to put out of reach.";
            case SETTINGS -> "Manage access and your device.";
            default -> "See where your time goes.";
        };
        text(words, subtitle, 14, MUTED, false).setPadding(0, 0, 0, 0);
        hero.addView(new DisciplineArt(this, "scene-" + scene),
            new LinearLayout.LayoutParams(dp(104), dp(110)));
        root.addView(hero, new LinearLayout.LayoutParams(-1, -2));
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
            text(files, "Import the child’s activity file here. In Blocking, select apps and export a rules file for the child phone.", 15, MUTED, false);
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

        LinearLayout blocking = card("Blocking service");
        serviceBadge(blocking);
        text(blocking, "The Accessibility Service applies your selected app blocks.", 15, MUTED, false);
        if (Rules.serviceEnabled(this)) {
            action(blocking, "Manage Accessibility", () -> launchSettings(Settings.ACTION_ACCESSIBILITY_SETTINGS));
        } else {
            primary(blocking, "Set up app blocking", this::explainBlocking);
        }
        if (Rules.CHILD.equals(mode)) {
            LinearLayout exchange = card("Parent file exchange");
            text(exchange, "The activity file includes installed apps, today’s usage sessions, and available weekly activity totals.", 15, MUTED, false);
            action(exchange, "Export child activity", this::exportActivity);
            action(exchange, "Import parent rules", () -> openFile("rules"));
            action(root, "Lock parent settings", () -> {
                unlocked = false;
                render();
            });
        }
    }

    private void explainBlocking() {
        new AlertDialog.Builder(this).setTitle("Enable app blocking")
            .setMessage("Digital Discipline checks the package name of the active app. If you selected that app for blocking, it returns to Home and shows a short notice. The service does not read screen content. You can turn it off in Android Accessibility settings.")
            .setNegativeButton("Cancel", null)
            .setPositiveButton("Open Android settings", (dialog, which) -> launchSettings(Settings.ACTION_ACCESSIBILITY_SETTINGS))
            .show();
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

    private JSONArray localAppList() {
        JSONArray rows = new JSONArray();
        Set<String> protectedApps = AppCatalog.protectedPackages(this);
        Map<String, String> names = Rules.apps(this);
        List<Map.Entry<String, String>> apps = new ArrayList<>(names.entrySet());
        apps.sort(Map.Entry.comparingByValue(String.CASE_INSENSITIVE_ORDER));
        for (var app : apps) {
            try {
                rows.put(new JSONObject().put("id", app.getKey()).put("name", app.getValue())
                    .put("blockable", !protectedApps.contains(app.getKey())));
            } catch (org.json.JSONException e) {
                throw new IllegalStateException(e);
            }
        }
        return rows;
    }

    private void showBlocks(JSONArray rows, boolean remote) {
        LinearLayout box = card(remote ? "Child app rules" : "Installed apps");
        if (rows == null) {
            text(box, "Import an activity file to see the child phone’s apps and choose which ones to block.", 15, MUTED, false);
            primary(box, "Import child activity", () -> openFile("activity"));
            return;
        }
        if (!remote) {
            serviceBadge(box);
            if (!BlockService.isConnected() || !Rules.serviceEnabled(this)) {
                text(box, "Rules only take effect while the blocking service is connected.", 14, MUTED, false);
                action(box, "Open Accessibility setup", this::explainBlocking);
            }
        }
        TextView selectionCount = text(box, "", 14, MUTED, false);
        EditText search = new EditText(this);
        search.setSingleLine(true);
        search.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        search.setTextSize(15);
        search.setHint("Search installed apps");
        search.setContentDescription("Search installed apps");
        search.setTextColor(WHITE);
        search.setHintTextColor(MUTED);
        search.setPadding(dp(14), 0, dp(14), 0);
        search.setBackground(surface(BG));
        LinearLayout.LayoutParams searchSize = new LinearLayout.LayoutParams(-1, dp(49));
        searchSize.bottomMargin = dp(9);
        box.addView(search, searchSize);

        LinearLayout list = new LinearLayout(this);
        list.setOrientation(LinearLayout.VERTICAL);
        box.addView(list, new LinearLayout.LayoutParams(-1, -2));
        Runnable updateCount = () -> {
            Set<String> selected = rules.blocks();
            int enabled = 0;
            for (int i = 0; i < rows.length(); i++) {
                JSONObject app = rows.optJSONObject(i);
                if (app != null && app.optBoolean("blockable") && selected.contains(app.optString("id"))) enabled++;
            }
            selectionCount.setText(getString(R.string.app_list_count, rows.length(), enabled));
        };
        Runnable filter = () -> {
            list.removeAllViews();
            String match = search.getText().toString().trim().toLowerCase(Locale.ROOT);
            for (int index = 0; index < rows.length(); index++) {
                JSONObject app = rows.optJSONObject(index);
                if (app == null) continue;
                String id = app.optString("id");
                String name = app.optString("name", id);
                if (id.isEmpty() || !(name.toLowerCase(Locale.ROOT).contains(match)
                    || id.toLowerCase(Locale.ROOT).contains(match))) continue;
                if (list.getChildCount() > 0) divider(list);
                appRow(list, id, name, app.optBoolean("blockable"), remote, updateCount);
            }
            if (list.getChildCount() == 0) text(list, "No matching apps.", 15, MUTED, false);
        };
        search.addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            @Override public void onTextChanged(CharSequence value, int start, int before, int count) {
                filter.run();
            }
            @Override public void afterTextChanged(Editable value) {}
        });
        updateCount.run();
        filter.run();
    }

    private void appRow(LinearLayout list, String id, String name, boolean blockable, boolean remote, Runnable updateCount) {
        LinearLayout row = new LinearLayout(this);
        row.setMinimumHeight(dp(64));
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setPadding(0, dp(8), 0, dp(8));
        list.addView(row, new LinearLayout.LayoutParams(-1, -2));

        ImageView icon = new ImageView(this);
        android.graphics.drawable.Drawable appIcon = AppCatalog.icon(this, id);
        if (appIcon != null) {
            icon.setImageDrawable(appIcon);
            icon.setScaleType(ImageView.ScaleType.FIT_CENTER);
            icon.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            row.addView(icon, new LinearLayout.LayoutParams(dp(40), dp(40)));
        } else {
            TextView initial = new TextView(this);
            initial.setGravity(Gravity.CENTER);
            initial.setTextColor(ACCENT);
            initial.setTextSize(20);
            initial.setText(name.isEmpty() ? "?" : name.substring(0, name.offsetByCodePoints(0, 1)));
            initial.setBackground(surface(0xff342746));
            initial.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            row.addView(initial, new LinearLayout.LayoutParams(dp(40), dp(40)));
        }

        LinearLayout labels = new LinearLayout(this);
        labels.setOrientation(LinearLayout.VERTICAL);
        LinearLayout.LayoutParams copy = new LinearLayout.LayoutParams(0, -2, 1);
        copy.setMargins(dp(11), 0, dp(8), 0);
        row.addView(labels, copy);
        text(labels, name, 16, WHITE, true).setPadding(0, 0, 0, 0);
        if (!blockable) {
            text(labels, "Protected system app", 12, MINT, false).setPadding(0, dp(3), 0, 0);
            return;
        }

        Switch item = new Switch(this);
        item.setContentDescription(getString(R.string.block_app_switch, name));
        item.setShowText(false);
        item.setMinHeight(dp(48));
        item.setMinWidth(dp(48));
        int[][] states = {{android.R.attr.state_checked}, {}};
        item.setThumbTintList(new ColorStateList(states, new int[]{ACCENT, 0xffc6bdcd}));
        item.setTrackTintList(new ColorStateList(states, new int[]{0xff9471c6, 0xff5d5367}));
        item.setChecked(rules.blocks().contains(id));
        item.setOnCheckedChangeListener((view, checked) -> {
            if (!remote && !Rules.canBlock(this, id)) {
                render();
                return;
            }
            rules.toggle(id, checked);
            updateCount.run();
        });
        row.addView(item, new LinearLayout.LayoutParams(-2, dp(48)));
    }

    private void serviceBadge(LinearLayout box) {
        serviceStatus = badge(box, "", PEACH);
        ui.removeCallbacks(refreshServiceStatus);
        refreshServiceStatus.run();
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

    private void showActivity(JSONObject report) {
        boolean parent = Rules.PARENT.equals(rules.mode());
        boolean tracked = report.optBoolean("tracked");
        java.time.ZoneId zone = reportZone(report);
        if (detailApp != null) {
            showAppDetails(report, zone);
            return;
        }
        activityArtwork(weekly ? R.drawable.activity_week : R.drawable.activity_today, null);
        LinearLayout range = new LinearLayout(this);
        range.setBackground(surface(0xff1a1522));
        range.setPadding(dp(3), dp(3), dp(3), dp(3));
        LinearLayout.LayoutParams rangeSize = new LinearLayout.LayoutParams(-1, -2);
        rangeSize.topMargin = dp(12);
        root.addView(range, rangeSize);
        rangeButton(range, "Today", false);
        rangeButton(range, "Last 7 Days", true);
        if (!tracked) {
            LinearLayout summary = card(report.has("apps") ? "Usage Access needed" : "No activity report");
            if (parent) primary(summary, "Import child activity", () -> openFile("activity"));
            else primary(summary, "Open Usage Access", () -> launchSettings(Settings.ACTION_USAGE_ACCESS_SETTINGS));
            return;
        }
        long end = report.optLong("at");
        if (parent) text(root, "Report from " + date(end), 13, MUTED, false);
        if (weekly) {
            showWeek(report.optJSONArray("week"));
        } else {
            showToday(report, zone, end);
        }
        if (!parent) action(root, "Refresh activity", this::render);
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
        if (appId == null) return;
        FrameLayout.LayoutParams center = new FrameLayout.LayoutParams(dp(49), dp(49), Gravity.CENTER);
        center.topMargin = dp(13);
        art.addView(activityIcon(appId), center);
    }

    private void showWeek(JSONArray week) {
        LinearLayout card = card("Activity heatmap");
        if (week == null || week.length() != 7) {
            text(card, "Weekly data is unavailable in this report.", 14, MUTED, false);
            return;
        }
        card.addView(new WeekChart(this, week, this::sessionDuration));
    }

    private void showToday(JSONObject report, java.time.ZoneId zone, long end) {
        JSONArray apps = report.optJSONArray("apps");
        if (apps == null) return;
        boolean hasToday = report.has("todayStart");
        String durationField = hasToday ? "today" : "time";
        LinearLayout summary = card(hasToday ? "Screen time by hour" : "Recorded screen time");
        JSONArray hours = report.optJSONArray("hours");
        if (hours != null) {
            HourChart chart = new HourChart(this, hours, report.optLong("hourStart"),
                report.optLong("todayStart"), end, zone);
            summary.addView(chart, new LinearLayout.LayoutParams(-1, dp(114)));
            TextView detail = text(summary, "Tap a bar for hourly usage", 13, MUTED, false);
            chart.setOnHourSelected((label, millis) -> detail.setText(label + " · " + duration(millis)));
        } else {
            text(summary, "This report predates hourly charts.", 14, MUTED, false);
        }
        Map<String, String> labels = new java.util.HashMap<>();
        List<JSONObject> used = new ArrayList<>();
        for (int i = 0; i < apps.length(); i++) {
            JSONObject app = apps.optJSONObject(i);
            if (app == null) continue;
            labels.put(app.optString("id"), app.optString("name"));
            if (app.optLong(durationField) > 0) used.add(app);
        }
        LinearLayout ranking = card("Top apps");
        if (used.isEmpty()) {
            text(ranking, "No foreground app use recorded today.", 14, MUTED, false);
            return;
        }
        used.sort((a, b) -> Long.compare(b.optLong(durationField), a.optLong(durationField)));
        long most = used.get(0).optLong(durationField);
        for (JSONObject app : used) addRankedApp(ranking, app, durationField, most);
        if (!report.has("sessions")) {
            long from = hasToday ? report.optLong("todayStart") : report.optLong("from");
            showOpenHistory(report.optJSONArray("opens"), labels, from, zone);
        }
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
                    fallbackTotal = app.optLong(report.has("todayStart") ? "today" : "time");
                    break;
                }
            }
        }
        title(name, report.has("todayStart") ? "Usage sessions" : "Recorded usage");
        LinearLayout list = card("Opened · Ended");
        long total = 0;
        JSONArray sessions = report.optJSONArray("sessions");
        java.time.format.DateTimeFormatter clock = java.time.format.DateTimeFormatter
            .ofLocalizedTime(java.time.format.FormatStyle.SHORT).withZone(zone);
        if (sessions != null) {
            for (int i = 0; i < sessions.length(); i++) {
                JSONObject item = sessions.optJSONObject(i);
                if (item == null || !detailApp.equals(item.optString("id"))) continue;
                long from = item.optLong("from");
                long to = item.optLong("to");
                if (to <= from) continue;
                long elapsed = to - from;
                total += elapsed;
                String finish = item.optBoolean("ongoing") ? "In progress" : clock.format(java.time.Instant.ofEpochMilli(to));
                sessionLine(list, clock.format(java.time.Instant.ofEpochMilli(from)) + " – " + finish, sessionDuration(elapsed));
                divider(list);
            }
        } else {
            total = fallbackTotal;
            text(list, "Session times are unavailable in this report.", 14, MUTED, false);
        }
        if (sessions != null && total == 0) text(list, "No sessions recorded for this app.", 14, MUTED, false);
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

    private void rangeButton(LinearLayout range, String title, boolean week) {
        Button item = button(title, () -> {
            if (weekly == week) return;
            weekly = week;
            render();
            enter();
        }, weekly == week);
        item.setSelected(weekly == week);
        if (weekly && week) {
            item.setBackground(ripple(surface(0xffb4ead5)));
            item.setTextColor(0xff102920);
        }
        range.addView(item, new LinearLayout.LayoutParams(0, -2, 1));
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

    private void exportActivity() {
        if (busy) return;
        if (!unlocked || !Rules.showChildNotice(this)) {
            status("Allow the supervision notification before exporting.");
            return;
        }
        busy = true;
        new Thread(() -> {
            try {
                JSONObject report = ActivityReport.read(this, rules.prefs.getLong("enrolled", 0), true);
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

    private void exportRules() {
        try {
            long revision = rules.prefs.getLong("sent_revision", 0) + 1;
            if (!rules.prefs.edit().putLong("sent_revision", revision).commit()) {
                status("Could not save the rule revision.");
                return;
            }
            JSONObject policy = new JSONObject().put("revision", revision)
                .put("apps", new JSONArray(rules.blocks()));
            saveFile("rules", rules.pack("rules", policy));
        } catch (Exception e) {
            status("Could not create the rules file.");
        }
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
        } else if ("rules".equals(kind) && Rules.CHILD.equals(rules.mode())) {
            long revision = content.getLong("revision");
            if (revision <= rules.prefs.getLong("applied_revision", 0)) {
                status("Newer or identical rules are already applied.");
                return;
            }
            JSONArray ids = content.getJSONArray("apps");
            Set<String> eligible = Rules.apps(this).keySet();
            Set<String> blocks = new HashSet<>();
            for (int i = 0; i < ids.length(); i++) {
                String id = ids.getString(i);
                if (eligible.contains(id) && Rules.canBlock(this, id)) blocks.add(id);
            }
            if (!rules.prefs.edit().putLong("applied_revision", revision).putStringSet("blocks", blocks).commit()) {
                throw new IllegalStateException();
            }
            status(blocks.size() + " blocked apps saved.");
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
