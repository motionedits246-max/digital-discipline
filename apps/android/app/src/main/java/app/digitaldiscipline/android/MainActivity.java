package app.digitaldiscipline.android;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.Color;
import android.graphics.Typeface;
import android.os.Build;
import android.os.Bundle;
import android.provider.Settings;
import android.text.InputFilter;
import android.text.InputType;
import android.view.Gravity;
import android.view.WindowManager;
import android.widget.EditText;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

import com.google.zxing.BarcodeFormat;
import com.google.zxing.WriterException;
import com.google.zxing.common.BitMatrix;
import com.google.zxing.qrcode.QRCodeWriter;
import com.google.zxing.integration.android.IntentIntegrator;
import com.google.zxing.integration.android.IntentResult;

import org.json.JSONArray;
import org.json.JSONObject;

import java.text.DateFormat;
import java.util.Date;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;

public final class MainActivity extends Activity {
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private DeviceState state;
    private LinearLayout page;
    private ScrollView scroll;
    private JSONObject snapshot;
    private boolean family;
    private boolean childSetup;
    private boolean unlocked;
    private boolean busy;
    private boolean resumed;
    private int screenSession;
    private boolean history;
    private int scrollPosition;
    private String status = "";
    private String scannedCode;

    @Override protected void onCreate(Bundle saved) {
        super.onCreate(saved);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE);
        state = new DeviceState(this);
    }

    @Override protected void onResume() {
        super.onResume();
        resumed = true;
        render();
        if (isLocalManager() && !busy) refreshLocal();
        if (DeviceState.PARENT.equals(state.role()) && scannedCode != null) {
            String code = scannedCode;
            scannedCode = null;
            connect(code);
        } else if (DeviceState.PARENT.equals(state.role()) && !state.parentCode().isEmpty() && snapshot == null) {
            request(state.parentCode(), new JSONObject(), result -> { snapshot = result; announce(""); });
        }
    }

    @Override protected void onStop() {
        super.onStop();
        resumed = false;
        screenSession++;
        if (DeviceState.CHILD.equals(state.role())) {
            scrollPosition = 0;
            status = "";
            unlocked = false;
            snapshot = null;
        }
    }

    @Override protected void onDestroy() {
        worker.shutdownNow();
        super.onDestroy();
    }

    private boolean isLocalManager() {
        return DeviceState.INDIVIDUAL.equals(state.role()) ||
            (DeviceState.CHILD.equals(state.role()) && unlocked);
    }

    private void render() {
        if (isFinishing() || isDestroyed()) return;
        scroll = new ScrollView(this);
        scroll.setBackgroundColor(Ui.BG);
        scroll.setFillViewport(true);
        scroll.setClipToPadding(false);
        page = Ui.column(this);
        int side = Ui.dp(this, 20);
        page.setPadding(side, Ui.dp(this, 22), side, Ui.dp(this, 30));
        page.setFocusableInTouchMode(true);
        scroll.addView(page);
        setContentView(scroll);
        page.requestFocus();

        LinearLayout header = new LinearLayout(this);
        header.setGravity(Gravity.CENTER_VERTICAL);
        ImageView icon = new ImageView(this);
        icon.setImageResource(R.drawable.ic_discipline);
        header.addView(icon, new LinearLayout.LayoutParams(Ui.dp(this, 34), Ui.dp(this, 34)));
        TextView wordmark = Ui.text(this, "Digital Discipline", 17, Ui.WHITE, true);
        wordmark.setPadding(Ui.dp(this, 11), 0, 0, 0);
        header.addView(wordmark);
        page.addView(header);
        if (!status.isEmpty()) {
            LinearLayout feedback = Ui.card(this);
            TextView message = Ui.text(this, status, 14, Ui.WHITE, false);
            message.setAccessibilityLiveRegion(TextView.ACCESSIBILITY_LIVE_REGION_POLITE);
            feedback.addView(message);
            Ui.add(page, feedback, 14);
        }
        String role = state.role();
        if (role.isEmpty()) setup();
        else if (role.equals(DeviceState.PARENT)) parent();
        else if (role.equals(DeviceState.CHILD) && !unlocked) childLock();
        else dashboard(role.equals(DeviceState.CHILD));
        int restore = scrollPosition;
        ScrollView current = scroll;
        if (restore > 0) current.post(() -> current.scrollTo(0, restore));
    }

    private void title(String title, String detail) {
        Ui.add(page, Ui.text(this, title, 30, Ui.WHITE, true), 26);
        if (!detail.isEmpty()) Ui.body(page, detail);
    }

    private LinearLayout section(String title) {
        LinearLayout card = Ui.card(this);
        card.addView(Ui.text(this, title, 17, Ui.WHITE, true));
        Ui.add(page, card, 14);
        return card;
    }

    private EditText field(LinearLayout card, String hint, boolean pin) {
        EditText input = new EditText(this);
        input.setTextColor(Ui.WHITE);
        input.setHintTextColor(Ui.MUTED);
        input.setHint(hint);
        input.setTextSize(15);
        input.setSingleLine(true);
        if (pin) {
            input.setInputType(InputType.TYPE_CLASS_NUMBER | InputType.TYPE_NUMBER_VARIATION_PASSWORD);
            input.setFilters(new InputFilter[]{new InputFilter.LengthFilter(6)});
        } else {
            input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
            input.setFilters(new InputFilter[]{new InputFilter.LengthFilter(96)});
        }
        input.setImportantForAutofill(EditText.IMPORTANT_FOR_AUTOFILL_NO);
        input.setTypeface(Typeface.MONOSPACE);
        input.setBackground(Ui.surface(this, 0xFF202024, Ui.LINE, 9));
        int pad = Ui.dp(this, 12);
        input.setPadding(pad, pad, pad, pad);
        Ui.add(card, input, 12);
        return input;
    }

    private void setup() {
        if (childSetup) {
            title("Set parent PIN", "The parent uses this PIN to open settings on this phone.");
            LinearLayout card = section("Child phone");
            EditText pin = field(card, "Six-digit PIN", true);
            EditText repeat = field(card, "Confirm PIN", true);
            Ui.action(card, busy ? "Setting PIN…" : "Continue", true, () -> {
                String chosen = pin.getText().toString();
                if (!chosen.matches("[0-9]{6}") || !chosen.equals(repeat.getText().toString())) {
                    announce("Enter the same six-digit PIN in both fields.");
                    return;
                }
                perform(() -> { state.createChild(chosen); return true; }, ok -> {
                    unlocked = true;
                    refreshAfterSetup();
                });
            });
            Ui.action(card, "Back", false, () -> { if (!busy) { childSetup = false; announce(""); } });
        } else if (family) {
            title("Parent & child", "Install Digital Discipline on both phones. Choose a role on each phone.");
            LinearLayout card = section("This phone");
            Ui.action(card, "Parent phone", true, () -> {
                state.choose(DeviceState.PARENT);
                announce("");
            });
            Ui.action(card, "Child phone", false, () -> { childSetup = true; announce(""); });
            Ui.action(page, "Back", false, () -> { family = false; announce(""); });
        } else {
            title("Choose how to use it", "");
            LinearLayout individual = section("Individual");
            Ui.body(individual, "Manage your own app blocks and see your last 24 hours of usage.");
            Ui.action(individual, "Use on my phone", true, () -> {
                state.choose(DeviceState.INDIVIDUAL);
                refreshAfterSetup();
            });
            LinearLayout pair = section("Parent & child");
            Ui.body(pair, "Manage a child phone from a parent phone on the same Wi-Fi or hotspot.");
            Ui.action(pair, "Set up family mode", false, () -> { family = true; announce(""); });
        }
    }

    private void refreshAfterSetup() {
        status = "";
        render();
        refreshLocal();
    }

    private void childLock() {
        title("Family mode", "App blocks are managed by your parent.");
        LinearLayout card = section("Parent access");
        EditText pin = field(card, "Parent PIN", true);
        Ui.action(card, busy ? "Checking…" : "Unlock settings", true, () -> {
            String input = pin.getText().toString();
            perform(() -> state.checkPin(input), result -> {
                if (!result.isEmpty()) { announce(result); return; }
                unlocked = true;
                refreshAfterSetup();
            });
        });
    }

    private void parent() {
        title("Child device", "Family controls");
        String saved = state.parentCode();
        if (saved.isEmpty()) {
            LinearLayout card = section("Pair child phone");
            Ui.body(card, "Connect both phones to the same Wi-Fi or hotspot. On the child phone, open Parent access and show the pairing code.");
            EditText code = field(card, "Child pairing code", false);
            Ui.action(card, busy ? "Connecting…" : "Scan child pairing QR", true, () -> {
                if (busy) return;
                new IntentIntegrator(this).setDesiredBarcodeFormats(IntentIntegrator.QR_CODE)
                    .setBeepEnabled(false).setPrompt("Scan the QR shown on the child phone.").initiateScan();
            });
            Ui.action(card, "Connect using code", false, () -> connect(code.getText().toString().trim()));
            Ui.action(page, "Change mode", false, this::reset);
        } else {
            LinearLayout card = section("Family connection");
            Ui.body(card, "The child phone must be reachable on the local network. If its address changes, pair again.");
            Ui.action(card, busy ? "Connecting…" : "Refresh child data", true, () ->
                request(saved, new JSONObject(), result -> { snapshot = result; announce(""); }));
            Ui.action(card, "Forget child phone", false, () -> {
                if (busy) return;
                state.prefs.edit().remove("parentCode").apply();
                snapshot = null;
                announce("Pairing removed from this phone.");
            });
            if (snapshot != null) report(snapshot, true);
        }
    }

    @Override protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        IntentResult result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data);
        if (result != null && DeviceState.PARENT.equals(state.role()) && state.parentCode().isEmpty()) {
            scannedCode = result.getContents();
        }
    }

    private void connect(String code) {
        request(code, new JSONObject(), result -> {
            state.prefs.edit().putString("parentCode", code).apply();
            snapshot = result;
            announce("Child phone connected.");
        });
    }

    private void dashboard(boolean child) {
        title(child ? "Parent settings" : "Overview", child ? "Manage this child phone." : "Activity and app blocking on this phone.");
        LinearLayout permissions = section("Device access");
        Ui.body(permissions, BlockService.running ? "App blocker is enabled." : "Enable the app blocker in Android Accessibility.");
        Ui.action(permissions, "Open Accessibility settings", false, () ->
            startActivity(new Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)));
        if (state.trackingSince() == 0 || !state.usageAllowed()) {
            Ui.body(permissions, "Usage access measures app-open times and foreground duration.");
            Ui.action(permissions, "Enable activity tracking", false, () -> {
                if (state.trackingSince() == 0) {
                    state.prefs.edit().putLong("trackingSince", System.currentTimeMillis()).apply();
                }
                if (!state.usageAllowed()) startActivity(new Intent(Settings.ACTION_USAGE_ACCESS_SETTINGS));
                else refreshLocal();
            });
        } else {
            Ui.body(permissions, "Activity tracking is enabled.");
            Ui.action(permissions, "Turn off tracking", false, () -> {
                state.prefs.edit().remove("trackingSince").apply();
                snapshot = null;
                refreshLocal();
            });
        }
        if (child) childPairing();
        Ui.action(page, busy ? "Loading activity…" : "Refresh activity", false, this::refreshLocal);
        if (snapshot != null) report(snapshot, false);
        if (child) Ui.action(page, "Lock parent settings", false, () -> {
            unlocked = false;
            snapshot = null;
            announce("");
        });
        Ui.action(page, "Change mode", false, this::reset);
    }

    private void childPairing() {
        LinearLayout card = section("Pair parent phone");
        Ui.body(card, "Pairing shares installed app names and, when enabled, usage history with the parent phone. Keep the pairing code private.");
        if (!BlockService.mayShare(state)) {
            Ui.action(card, "Allow family status notification", false, () -> {
                if (Build.VERSION.SDK_INT >= 33 && checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                    != android.content.pm.PackageManager.PERMISSION_GRANTED &&
                    !state.prefs.getBoolean("notificationAsked", false)) {
                    state.prefs.edit().putBoolean("notificationAsked", true).apply();
                    requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 20);
                } else {
                    startActivity(new Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
                        .putExtra(Settings.EXTRA_APP_PACKAGE, getPackageName()));
                }
            });
        }
        Ui.action(card, "Show pairing code", true, () -> {
            if (!BlockService.running || !BlockService.mayShare(state)) {
                announce("Enable the app blocker and Family connection notifications first.");
                return;
            }
            if (state.childKey().isEmpty()) {
                state.prefs.edit().putString("childKey", FamilyLink.encode(FamilyLink.random(32))).apply();
            }
            String code = BlockService.pairingCode;
            if (code.isEmpty()) {
                announce("Connect to Wi-Fi or a hotspot, then try again.");
                return;
            }
            showPairingDialog(code);
        });
        if (!state.childKey().isEmpty()) {
            Ui.action(card, "Disconnect parent phone", false, () -> {
                synchronized (DeviceState.RULE_LOCK) {
                    state.prefs.edit().remove("childKey").apply();
                }
                announce("Family connection disabled. Existing blocks still apply.");
            });
        }
    }

    private void showPairingDialog(String code) {
        LinearLayout content = Ui.column(this);
        content.setGravity(Gravity.CENTER_HORIZONTAL);
        content.setPadding(Ui.dp(this, 18), Ui.dp(this, 12), Ui.dp(this, 18), Ui.dp(this, 14));
        try {
            int size = 360;
            BitMatrix qr = new QRCodeWriter().encode(code, BarcodeFormat.QR_CODE, size, size);
            int[] pixels = new int[size * size];
            for (int y = 0; y < size; y++) {
                for (int x = 0; x < size; x++) {
                    pixels[y * size + x] = qr.get(x, y) ? Color.BLACK : Color.WHITE;
                }
            }
            ImageView image = new ImageView(this);
            image.setImageBitmap(Bitmap.createBitmap(pixels, size, size, Bitmap.Config.ARGB_8888));
            image.setAdjustViewBounds(true);
            image.setContentDescription("Child phone pairing QR");
            int side = Ui.dp(this, 220);
            content.addView(image, new LinearLayout.LayoutParams(side, side));
        } catch (WriterException ignored) {
            // The text code still works.
        }
        TextView text = Ui.text(this, code, 14, Ui.WHITE, true);
        text.setTypeface(Typeface.MONOSPACE);
        text.setTextIsSelectable(true);
        Ui.add(content, text, 10);
        AlertDialog dialog = new AlertDialog.Builder(this)
            .setTitle("Pair parent phone").setView(content).setPositiveButton("Done", null).create();
        dialog.getWindow().addFlags(WindowManager.LayoutParams.FLAG_SECURE);
        dialog.show();
    }

    private void report(JSONObject data, boolean remote) {
        JSONArray apps = data.optJSONArray("apps");
        if (apps == null) return;
        int blocked = 0;
        long total = 0;
        for (int i = 0; i < apps.length(); i++) {
            JSONObject row = apps.optJSONObject(i);
            if (row == null) continue;
            if (row.optBoolean("blocked")) blocked++;
            total += row.optLong("ms");
        }
        boolean tracking = data.optBoolean("tracking");
        LinearLayout overview = section("Last 24 hours");
        Ui.add(overview, Ui.text(this, tracking ? duration(total) : "Tracking off", 29, Ui.WHITE, true), 5);
        Ui.body(overview, blocked + " apps blocked · " + (data.optBoolean("blocker") ? "Blocker enabled" : "Blocker off"));
        Ui.action(overview, history ? "Show app controls" : "Show activity timeline", false, () -> {
            scrollPosition = scroll.getScrollY();
            history = !history;
            render();
        });
        if (history) timeline(data.optJSONArray("history"), tracking);
        else appControls(apps, remote);
    }

    private void appControls(JSONArray apps, boolean remote) {
        LinearLayout list = section("App controls");
        if (apps.length() == 0) Ui.body(list, "No apps available to manage.");
        for (int i = 0; i < apps.length(); i++) {
            JSONObject row = apps.optJSONObject(i);
            if (row == null) continue;
            LinearLayout line = new LinearLayout(this);
            line.setGravity(Gravity.CENTER_VERTICAL);
            LinearLayout label = Ui.column(this);
            label.addView(Ui.text(this, row.optString("name"), 14, Ui.WHITE, true));
            label.addView(Ui.text(this, duration(row.optLong("ms")), 12, Ui.MUTED, false));
            line.addView(label, new LinearLayout.LayoutParams(0, -2, 1));
            boolean blocked = row.optBoolean("blocked");
            String pkg = row.optString("pkg");
            android.widget.Button button = Ui.button(this, blocked ? "Unblock" : "Block", false, () -> {
                if (busy) return;
                scrollPosition = scroll.getScrollY();
                if (remote) request(state.parentCode(), blockRequest(pkg, !blocked), result -> {
                    snapshot = result;
                    announce(blocked ? "App unblocked." : "App blocked.");
                });
                else perform(() -> {
                    if (!state.changeBlock(pkg, !blocked)) {
                        throw new IllegalArgumentException("This app is unavailable or protected.");
                    }
                    return state.snapshot();
                }, result -> { snapshot = result; announce(""); });
            });
            button.setContentDescription((blocked ? "Unblock " : "Block ") + row.optString("name"));
            button.setEnabled(!busy);
            line.addView(button, new LinearLayout.LayoutParams(Ui.dp(this, 93), -2));
            Ui.add(list, line, 10);
        }
    }

    private void timeline(JSONArray entries, boolean tracking) {
        LinearLayout list = section("Recent app opens");
        if (entries == null || entries.length() == 0) {
            Ui.body(list, tracking ? "No app opens recorded yet." : "Enable activity tracking on the device to see app opens.");
            return;
        }
        DateFormat time = DateFormat.getDateTimeInstance(DateFormat.MEDIUM, DateFormat.SHORT);
        for (int i = 0; i < entries.length(); i++) {
            JSONObject entry = entries.optJSONObject(i);
            if (entry == null) continue;
            Ui.add(list, Ui.text(this, entry.optString("name"), 14, Ui.WHITE, true), 12);
            Ui.body(list, time.format(new Date(entry.optLong("start"))) + " · " + duration(entry.optLong("ms")));
        }
    }

    private static String duration(long ms) {
        long minutes = ms / 60_000;
        if (minutes == 0) return ms == 0 ? "0 min" : Math.max(1, ms / 1000) + " sec";
        if (minutes < 60) return minutes + " min";
        return (minutes / 60) + " hr " + (minutes % 60) + " min";
    }

    private void refreshLocal() {
        perform(state::snapshot, result -> {
            if (isLocalManager()) snapshot = result;
            announce("");
        });
    }

    private JSONObject blockRequest(String pkg, boolean blocked) {
        try {
            return new JSONObject().put("op", "block").put("pkg", pkg).put("blocked", blocked);
        } catch (org.json.JSONException impossible) {
            throw new AssertionError(impossible);
        }
    }

    private void request(String code, JSONObject payload, Consumer<JSONObject> success) {
        perform(() -> {
            if (!payload.has("op")) payload.put("op", "read");
            JSONObject response = FamilyLink.request(code, payload);
            if (response.has("error")) throw new IllegalArgumentException(response.getString("error"));
            return response;
        }, success);
    }

    private <T> void perform(Callable<T> task, Consumer<T> success) {
        if (busy) return;
        busy = true;
        int session = screenSession;
        worker.execute(() -> {
            try {
                T result = task.call();
                runOnUiThread(() -> {
                    if (isDestroyed()) return;
                    busy = false;
                    if (resumed && session == screenSession) success.accept(result);
                    else if (resumed) render();
                });
            } catch (Exception error) {
                runOnUiThread(() -> {
                    if (isDestroyed()) return;
                    busy = false;
                    String message = error instanceof IllegalArgumentException ? error.getMessage() :
                        (DeviceState.PARENT.equals(state.role()) ? "Couldn’t reach the child phone. Check the network and pairing code." :
                            "Couldn’t load device activity. Try again.");
                    if (resumed && session == screenSession) announce(message);
                    else if (resumed) render();
                });
            }
        });
    }

    private void announce(String message) {
        status = message;
        render();
    }

    private void reset() {
        if (busy) return;
        new AlertDialog.Builder(this).setTitle("Change mode?")
            .setMessage("This clears this phone’s pairing, PIN and app rules.")
            .setNegativeButton("Cancel", null)
            .setPositiveButton("Clear settings", (dialog, button) -> {
                synchronized (DeviceState.RULE_LOCK) { state.choose(""); }
                family = false;
                childSetup = false;
                unlocked = false;
                snapshot = null;
                history = false;
                scrollPosition = 0;
                announce("");
            }).show();
    }
}
