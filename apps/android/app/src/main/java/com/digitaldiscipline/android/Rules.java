package com.digitaldiscipline.android;

import android.Manifest;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.os.Build;
import android.provider.Settings;
import android.util.Base64;

import org.json.JSONObject;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.SecureRandom;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import javax.crypto.Cipher;
import javax.crypto.SecretKeyFactory;
import javax.crypto.spec.GCMParameterSpec;
import javax.crypto.spec.PBEKeySpec;
import javax.crypto.spec.SecretKeySpec;

final class Rules {
    static final String INDIVIDUAL = "individual";
    static final String PARENT = "parent";
    static final String CHILD = "child";
    private static final int BASE64 = Base64.URL_SAFE | Base64.NO_WRAP | Base64.NO_PADDING;
    private static final int NOTICE = 10;
    private static final String CHANNEL = "child-supervision";
    private static final SecureRandom RANDOM = new SecureRandom();
    final SharedPreferences prefs;

    Rules(Context context) {
        prefs = context.getSharedPreferences("discipline", Context.MODE_PRIVATE);
    }

    String mode() { return prefs.getString("mode", ""); }

    Set<String> blocks() { return new HashSet<>(prefs.getStringSet("blocks", java.util.Collections.emptySet())); }

    void toggle(String id, boolean blocked) {
        Set<String> next = blocks();
        if (blocked) next.add(id); else next.remove(id);
        prefs.edit().putStringSet("blocks", next).apply();
    }

    static Map<String, String> apps(Context context) {
        return AppCatalog.names(context);
    }

    static boolean canBlock(Context context, String id) {
        return AppCatalog.isLaunchable(context, id) && !AppCatalog.protectedPackages(context).contains(id);
    }

    static byte[] random(int size) {
        byte[] bytes = new byte[size];
        RANDOM.nextBytes(bytes);
        return bytes;
    }

    static String newKey() { return Base64.encodeToString(random(32), BASE64); }

    static byte[] keyBytes(String key) {
        if (!PairingInput.isCode(key)) throw new IllegalArgumentException("Invalid pairing code");
        byte[] bytes = Base64.decode(key, BASE64);
        if (bytes.length != 32) throw new IllegalArgumentException("Invalid pairing code");
        return bytes;
    }

    private static byte[] derive(char[] pin, byte[] salt) throws Exception {
        PBEKeySpec spec = new PBEKeySpec(pin, salt, 180000, 256);
        try {
            return SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(spec).getEncoded();
        } finally {
            spec.clearPassword();
            java.util.Arrays.fill(pin, '\0');
        }
    }

    void enroll(String key, String pin) throws Exception {
        keyBytes(key);
        byte[] salt = random(16);
        byte[] verifier = derive(pin.toCharArray(), salt);
        if (!prefs.edit().putString("mode", CHILD).putString("key", key)
            .putString("salt", Base64.encodeToString(salt, BASE64))
            .putString("verifier", Base64.encodeToString(verifier, BASE64))
            .putLong("enrolled", System.currentTimeMillis()).commit()) {
            throw new IllegalStateException("Setup could not be saved");
        }
    }

    boolean verify(String pin) throws Exception {
        if (!PairingInput.isPin(pin)) return false;
        byte[] salt = Base64.decode(prefs.getString("salt", ""), BASE64);
        byte[] expected = Base64.decode(prefs.getString("verifier", ""), BASE64);
        return MessageDigest.isEqual(expected, derive(pin.toCharArray(), salt));
    }

    String pack(String kind, JSONObject payload) throws Exception {
        byte[] nonce = random(12);
        Cipher cipher = packetCipher(Cipher.ENCRYPT_MODE, kind, nonce);
        byte[] body = cipher.doFinal(payload.toString().getBytes(StandardCharsets.UTF_8));
        return "DD1." + kind + "." + Base64.encodeToString(nonce, BASE64) + "."
            + Base64.encodeToString(body, BASE64);
    }

    JSONObject unpack(String kind, String packet) throws Exception {
        String[] parts = packet.trim().split("\\.", -1);
        if (parts.length != 4 || !"DD1".equals(parts[0]) || !kind.equals(parts[1])) {
            throw new IllegalArgumentException("Wrong file type");
        }
        byte[] nonce = Base64.decode(parts[2], BASE64);
        if (nonce.length != 12) throw new IllegalArgumentException("Invalid file");
        byte[] body = Base64.decode(parts[3], BASE64);
        Cipher cipher = packetCipher(Cipher.DECRYPT_MODE, kind, nonce);
        return new JSONObject(new String(cipher.doFinal(body), StandardCharsets.UTF_8));
    }

    private Cipher packetCipher(int mode, String kind, byte[] nonce) throws Exception {
        byte[] key = keyBytes(prefs.getString("key", ""));
        Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
        cipher.init(mode, new SecretKeySpec(key, "AES"), new GCMParameterSpec(128, nonce));
        cipher.updateAAD(("Digital Discipline 1/" + kind).getBytes(StandardCharsets.UTF_8));
        return cipher;
    }

    static boolean noticeAllowed(Context context) {
        NotificationManager manager = context.getSystemService(NotificationManager.class);
        manager.createNotificationChannel(new NotificationChannel(CHANNEL, "Child supervision",
            NotificationManager.IMPORTANCE_LOW));
        return manager.areNotificationsEnabled()
            && manager.getNotificationChannel(CHANNEL).getImportance() != NotificationManager.IMPORTANCE_NONE;
    }

    static boolean showChildNotice(Context context) {
        if (!noticeAllowed(context)) return false;
        if (Build.VERSION.SDK_INT >= 33
            && context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED) return false;
        NotificationManager manager = context.getSystemService(NotificationManager.class);
        PendingIntent open = PendingIntent.getActivity(context, 0, new Intent(context, MainActivity.class),
            PendingIntent.FLAG_IMMUTABLE);
        Notification notice = new Notification.Builder(context, CHANNEL)
            .setSmallIcon(R.drawable.ic_discipline).setContentTitle("Digital Discipline supervision")
            .setContentText("App rules and usage sharing are managed by your parent.")
            .setOngoing(true).setOnlyAlertOnce(true).setContentIntent(open).build();
        manager.notify(NOTICE, notice);
        return true;
    }

    static boolean serviceEnabled(Context context) {
        String enabled = Settings.Secure.getString(context.getContentResolver(),
            Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES);
        if (enabled == null) return false;
        for (String item : enabled.split(":")) {
            android.content.ComponentName name = android.content.ComponentName.unflattenFromString(item);
            if (name != null && name.getPackageName().equals(context.getPackageName())
                && name.getClassName().equals(BlockService.class.getName())) return true;
        }
        return false;
    }
}
