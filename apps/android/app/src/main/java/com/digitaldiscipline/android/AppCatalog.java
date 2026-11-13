package com.digitaldiscipline.android;

import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.content.pm.ResolveInfo;
import android.graphics.drawable.Drawable;

import java.util.Map;
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

    static Drawable icon(Context context, String id) {
        try {
            return context.getPackageManager().getApplicationIcon(id);
        } catch (PackageManager.NameNotFoundException e) {
            return null;
        }
    }

    private AppCatalog() {}
}
