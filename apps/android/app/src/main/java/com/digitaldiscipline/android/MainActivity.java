package com.digitaldiscipline.android;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.res.ColorStateList;
import android.graphics.Color;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.text.InputType;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.EditText;
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
import java.util.Date;
import java.util.HashSet;
import java.util.Set;
import java.util.concurrent.TimeUnit;

public final class MainActivity extends Activity {
    private static final int BG = 0xff0b0b0d;
    private static final int PANEL = 0xff141416;
    private static final int BORDER = 0xff2c2c30;
    private static final int WHITE = 0xfff2f2f4;
    private static final int MUTED = 0xffa6a6af;
    private static final int WRITE_FILE = 40;
    private static final int READ_FILE = 41;
    private static final int MAX_PACKET = 512 * 1024;
    private Rules rules;
    private LinearLayout root;
    private boolean unlocked;
    private boolean enrolling;
    private EditText enrollmentKey;
    private EditText enrollmentPin;
    private EditText confirmationPin;
    private String pendingPacket;
    private String pendingReadKind;
    private boolean busy;
    private int historyLimit = 30;
    private int tab;
    private static final int ACTIVITY = 0;
    private static final int BLOCKING = 1;
    private static final int SETTINGS = 2;

    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        if (Build.VERSION.SDK_INT >= 30) getWindow().setDecorFitsSystemWindows(false);
        rules = new Rules(this);
        if (state != null) tab = state.getInt("tab");
    }

    @Override protected void onSaveInstanceState(Bundle out) {
        out.putInt("tab", tab);
        super.onSaveInstanceState(out);
    }

    @Override protected void onResume() {
        super.onResume();
        if (Rules.CHILD.equals(rules.mode())) Rules.showChildNotice(this);
        if (!enrolling) render();
    }

    @Override protected void onStop() {
        super.onStop();
        if (Rules.CHILD.equals(rules.mode()) && unlocked) {
            unlocked = false;
            tab = SETTINGS;
            render();
        }
    }

    private void render() {
        ScrollView scroll = new ScrollView(this);
        scroll.setFillViewport(true);
        scroll.setBackgroundColor(BG);
        root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setPadding(dp(20), dp(18), dp(20), dp(24));
        scroll.addView(root);
        scroll.setOnApplyWindowInsetsListener((view, insets) -> {
            if (Build.VERSION.SDK_INT >= 30) {
                android.graphics.Insets system = insets.getInsets(WindowInsets.Type.systemBars()
                    | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
                view.setPadding(system.left, system.top, system.right, system.bottom);
            }
            return insets;
        });
        setContentView(scroll);
        scroll.requestApplyInsets();
        String mode = rules.mode();
        if (mode.isEmpty()) {
            title("Digital Discipline", "Choose how you’ll use this device");
            action(root, "Individual", () -> {
                rules.prefs.edit().putString("mode", Rules.INDIVIDUAL)
                    .putLong("enrolled", System.currentTimeMillis()).apply();
                render();
            });
            action(root, "Parent phone", () -> {
                rules.prefs.edit().putString("mode", Rules.PARENT)
                    .putString("key", Rules.newKey()).apply();
                tab = SETTINGS;
                render();
            });
            action(root, "Child phone", this::showEnrollment);
            return;
        }
        if (Rules.CHILD.equals(mode) && !unlocked) {
            title("Digital Discipline", "This phone is supervised by your parent.");
            action(root, "Parent access", this::askPin);
            return;
        }
        title("Digital Discipline", switch (mode) {
            case Rules.CHILD -> "Child device · Parent access";
            case Rules.PARENT -> "Parent dashboard";
            default -> "Individual dashboard";
        });
        LinearLayout nav = new LinearLayout(this);
        nav.setOrientation(LinearLayout.HORIZONTAL);
        root.addView(nav);
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

    private void settings(String mode) {
        if (Rules.PARENT.equals(mode)) {
            LinearLayout pair = card("Connect a child phone");
            text(pair, "On the child phone, choose Child phone and enter this pairing code. Set a separate parent PIN on that phone.", 15, MUTED, false);
            action(pair, "Show pairing code", this::showKey);
            LinearLayout files = card("Exchange files");
            text(files, "Import the child’s activity file here. In Blocking, select apps and export a rules file for the child phone.", 15, MUTED, false);
            action(files, "Import child activity", () -> openFile("activity"));
            return;
        }
        LinearLayout permissions = card("Android permissions");
        text(permissions, ActivityReport.allowed(this) ? "Usage access enabled" : "Usage access needed for timestamps and duration.", 15, MUTED, false);
        action(permissions, "Open Usage access", () -> launchSettings(Settings.ACTION_USAGE_ACCESS_SETTINGS));
        text(permissions, Rules.serviceEnabled(this) ? "App blocking enabled" : "App blocking needs the Digital Discipline accessibility service.", 15, MUTED, false);
        action(permissions, "Set up app blocking", this::explainBlocking);
        if (Rules.CHILD.equals(mode)) {
            LinearLayout exchange = card("Parent file exchange");
            text(exchange, "The activity file includes installed launcher apps, app-open times and foreground duration from the last 24 hours.", 15, MUTED, false);
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

    private void showEnrollment() {
        enrolling = true;
        root.removeAllViews();
        title("Child phone", "Set up with a parent");
        LinearLayout box = card("Supervision");
        text(box, "Your parent can block selected apps and receive app-open times and foreground usage. Android shows a Digital Discipline supervision notification.", 15, MUTED, false);
        enrollmentKey = field(box, "Pairing code", InputType.TYPE_CLASS_TEXT
            | InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        enrollmentPin = field(box, "Parent PIN · 6–12 digits", InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD);
        confirmationPin = field(box, "Confirm parent PIN", InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD);
        if (Build.VERSION.SDK_INT >= 33 && checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED) {
            action(box, "Allow supervision notification", () ->
                requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 44));
        }
        action(box, "Enable supervision", () -> {
            try {
                String key = enrollmentKey.getText().toString().trim();
                String pin = enrollmentPin.getText().toString();
                Rules.keyBytes(key);
                if (!pin.matches("[0-9]{6,12}") || !pin.equals(confirmationPin.getText().toString())) {
                    status("Enter matching PINs of 6–12 digits.");
                    return;
                }
                if (!Rules.noticeAllowed(this)) {
                    status("Enable Digital Discipline notifications in Android first.");
                    return;
                }
                rules.enroll(key, pin);
                Rules.showChildNotice(this);
                enrolling = false;
                unlocked = true;
                tab = SETTINGS;
                render();
            } catch (Exception e) {
                status("Check the pairing code and try again.");
            }
        });
        action(root, "Back", () -> {
            enrolling = false;
            render();
        });
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
        TextView code = new TextView(this);
        code.setPadding(dp(24), dp(16), dp(24), dp(16));
        code.setTextColor(WHITE);
        code.setTextSize(17);
        code.setTextIsSelectable(true);
        code.setText(rules.prefs.getString("key", ""));
        new AlertDialog.Builder(this).setTitle("Pairing code").setView(code)
            .setPositiveButton("Done", null).show();
    }

    private JSONArray localAppList() {
        JSONArray rows = new JSONArray();
        for (var app : Rules.apps(this).entrySet()) {
            if (!Rules.canBlock(this, app.getKey())) continue;
            try {
                rows.put(new JSONObject().put("id", app.getKey()).put("name", app.getValue()).put("blockable", true));
            } catch (org.json.JSONException e) {
                throw new IllegalStateException(e);
            }
        }
        return rows;
    }

    private void showBlocks(JSONArray rows, boolean remote) {
        LinearLayout box = card(remote ? "Child app rules" : "Block apps");
        if (rows == null) {
            text(box, "Import a child activity file in Settings to see apps on that phone.", 15, MUTED, false);
            return;
        }
        if (!remote && !Rules.serviceEnabled(this)) {
            action(box, "Enable blocking", this::explainBlocking);
        }
        Set<String> selected = rules.blocks();
        int listed = 0;
        for (int index = 0; index < rows.length(); index++) {
            JSONObject app = rows.optJSONObject(index);
            if (app == null || !app.optBoolean("blockable")) continue;
            String id = app.optString("id");
            if (id.isEmpty()) continue;
            Switch item = new Switch(this);
            item.setText(app.optString("name", id));
            item.setTextColor(WHITE);
            item.setTextSize(16);
            item.setMinHeight(dp(48));
            item.setChecked(selected.contains(id));
            item.setOnCheckedChangeListener((view, checked) -> rules.toggle(id, checked));
            box.addView(item, new LinearLayout.LayoutParams(-1, -2));
            listed++;
        }
        if (listed == 0) text(box, "No eligible apps found.", 15, MUTED, false);
    }

    private void showActivity(JSONObject report) {
        LinearLayout summary = card("Last 24 hours");
        if (!report.has("apps")) {
            text(summary, "Import a child activity file in Settings.", 15, MUTED, false);
            return;
        }
        if (Rules.PARENT.equals(rules.mode())) {
            text(summary, "Report from " + date(report.optLong("at")), 14, MUTED, false);
        } else {
            action(summary, "Refresh", this::render);
        }
        if (!report.optBoolean("tracked")) {
            text(summary, "Usage access is unavailable on the reporting phone.", 15, MUTED, false);
            return;
        }
        JSONArray apps = report.optJSONArray("apps");
        if (apps == null) return;
        java.util.Map<String, String> labels = new java.util.HashMap<>();
        java.util.List<JSONObject> used = new java.util.ArrayList<>();
        long total = 0;
        for (int i = 0; i < apps.length(); i++) {
            JSONObject app = apps.optJSONObject(i);
            if (app == null) continue;
            labels.put(app.optString("id"), app.optString("name"));
            long time = app.optLong("time");
            if (time > 0) {
                used.add(app);
                total += time;
            }
        }
        text(summary, duration(total), 31, WHITE, true);
        text(summary, "Foreground app usage · since " + date(report.optLong("from")), 14, MUTED, false);
        if (!used.isEmpty()) {
            LinearLayout breakdown = card("Time by app");
            used.sort((a, b) -> Long.compare(b.optLong("time"), a.optLong("time")));
            for (JSONObject app : used) {
                text(breakdown, app.optString("name") + "  ·  " + duration(app.optLong("time")), 15, WHITE, false);
            }
        }
        JSONArray opens = report.optJSONArray("opens");
        if (opens == null || opens.length() == 0) return;
        LinearLayout history = card("App-open history");
        for (int i = opens.length() - 1; i >= Math.max(0, opens.length() - historyLimit); i--) {
            JSONObject event = opens.optJSONObject(i);
            if (event == null) continue;
            String id = event.optString("id");
            text(history, labels.getOrDefault(id, id) + "  ·  " + date(event.optLong("time")), 14, MUTED, false);
        }
        if (opens.length() > historyLimit) {
            action(history, "Show 30 more", () -> {
                historyLimit += 30;
                render();
            });
        }
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
        text(root, title, 32, WHITE, true);
        text(root, description, 15, MUTED, false);
    }

    private LinearLayout card(String heading) {
        LinearLayout panel = new LinearLayout(this);
        panel.setOrientation(LinearLayout.VERTICAL);
        panel.setPadding(dp(16), dp(14), dp(16), dp(14));
        panel.setBackground(surface(PANEL));
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(-1, -2);
        layout.topMargin = dp(13);
        root.addView(panel, layout);
        text(panel, heading, 19, WHITE, true);
        return panel;
    }

    private void navButton(LinearLayout row, String title, int page) {
        Button button = button(title, () -> {
            tab = page;
            historyLimit = 30;
            render();
        }, tab == page);
        LinearLayout.LayoutParams item = new LinearLayout.LayoutParams(0, -2, 1);
        item.setMargins(dp(2), dp(8), dp(2), 0);
        row.addView(button, item);
    }

    private void action(LinearLayout parent, String title, Runnable listener) {
        Button button = button(title, listener, false);
        LinearLayout.LayoutParams layout = new LinearLayout.LayoutParams(-1, -2);
        layout.topMargin = dp(8);
        parent.addView(button, layout);
    }

    private Button button(String label, Runnable action, boolean selected) {
        Button button = new Button(this);
        button.setText(label);
        button.setAllCaps(false);
        button.setTextSize(14);
        button.setMinHeight(dp(48));
        button.setMinWidth(0);
        button.setTextColor(selected ? Color.BLACK : WHITE);
        button.setPadding(dp(8), dp(7), dp(8), dp(7));
        button.setBackground(new android.graphics.drawable.RippleDrawable(ColorStateList.valueOf(0x45a6a6af),
            surface(selected ? WHITE : 0xff242428), null));
        button.setOnClickListener(view -> action.run());
        return button;
    }

    private EditText field(LinearLayout parent, String hint, int input) {
        EditText text = new EditText(this);
        text.setInputType(input);
        text.setSingleLine(true);
        text.setHint(hint);
        text.setTextColor(WHITE);
        text.setHintTextColor(MUTED);
        text.setImportantForAutofill(View.IMPORTANT_FOR_AUTOFILL_NO);
        parent.addView(text, new LinearLayout.LayoutParams(-1, -2));
        return text;
    }

    private void text(LinearLayout parent, String value, int size, int color, boolean bold) {
        TextView text = new TextView(this);
        text.setText(value);
        text.setTextSize(size);
        text.setTextColor(color);
        text.setPadding(0, 0, 0, dp(8));
        if (bold) text.setTypeface(Typeface.DEFAULT, Typeface.BOLD);
        parent.addView(text, new LinearLayout.LayoutParams(-1, -2));
    }

    private GradientDrawable surface(int fill) {
        GradientDrawable shape = new GradientDrawable();
        shape.setColor(fill);
        shape.setStroke(dp(1), BORDER);
        shape.setCornerRadius(dp(13));
        return shape;
    }

    private int dp(float n) { return Math.round(n * getResources().getDisplayMetrics().density); }

    private void status(String message) { Toast.makeText(this, message, Toast.LENGTH_LONG).show(); }
}
