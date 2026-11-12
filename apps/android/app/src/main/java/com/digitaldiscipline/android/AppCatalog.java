package com.digitaldiscipline.android;

import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.pm.ResolveInfo;
import android.graphics.drawable.Drawable;
import android.provider.Settings;

import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

final class AppCatalog {
    private static final Intent LAUNCHER = new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER);

    static Map<String, String> names(Context context) {
        PackageManager pm = context.getPackageManager();
        Map<String, String> apps = new TreeMap<>();
        for (ResolveInfo result : pm.queryIntentActivities(LAUNCHER, 0)) {
            String id = result.activityInfo.packageName;
            if (!id.equals(context.getPackageName())) apps.putIfAbsent(id, result.loadLabel(pm).toString());
        }
        return apps;
    }

    static boolean isLaunchable(Context context, String id) {
        return !context.getPackageName().equals(id)
            && !context.getPackageManager().queryIntentActivities(new Intent(LAUNCHER).setPackage(id), 0).isEmpty();
    }

    static Drawable icon(Context context, String id) {
        try {
            return context.getPackageManager().getApplicationIcon(id);
        } catch (PackageManager.NameNotFoundException e) {
            return null;
        }
    }

    static Set<String> protectedPackages(Context context) {
        PackageManager pm = context.getPackageManager();
        Set<String> ids = new HashSet<>();
        ids.add(context.getPackageName());
        ids.add("android");
        ids.add("com.android.systemui");
        ids.add("com.android.settings");
        ids.add("com.android.permissioncontroller");
        ids.add("com.google.android.permissioncontroller");
        ids.add("com.android.packageinstaller");
        ids.add("com.google.android.packageinstaller");
        Intent home = new Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME);
        for (ResolveInfo launcher : pm.queryIntentActivities(home, PackageManager.MATCH_DEFAULT_ONLY)) {
            ids.add(launcher.activityInfo.packageName);
        }
        ResolveInfo settings = pm.resolveActivity(new Intent(Settings.ACTION_SETTINGS), PackageManager.MATCH_DEFAULT_ONLY);
        if (settings != null) ids.add(settings.activityInfo.packageName);
        String keyboard = Settings.Secure.getString(context.getContentResolver(), Settings.Secure.DEFAULT_INPUT_METHOD);
        if (keyboard != null && keyboard.contains("/")) ids.add(keyboard.substring(0, keyboard.indexOf('/')));
        android.telecom.TelecomManager telecom = context.getSystemService(android.telecom.TelecomManager.class);
        if (telecom != null) {
            ids.add(telecom.getDefaultDialerPackage());
            ids.add(telecom.getSystemDialerPackage());
        }
        ids.add(android.provider.Telephony.Sms.getDefaultSmsPackage(context));
        return ids;
    }

    private AppCatalog() {}
}
