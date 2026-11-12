package com.digitaldiscipline.android;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

public final class BootNotice extends BroadcastReceiver {
    @Override public void onReceive(Context context, Intent intent) {
        if (Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())
            && Rules.CHILD.equals(new Rules(context).mode())) {
            Rules.showChildNotice(context);
        }
    }
}
