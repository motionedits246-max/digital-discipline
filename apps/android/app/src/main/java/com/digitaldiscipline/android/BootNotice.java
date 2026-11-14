package com.digitaldiscipline.android;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

public final class BootNotice extends BroadcastReceiver {
    @Override public void onReceive(Context context, Intent intent) {
        if (!Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction())
                && !android.app.AlarmManager.ACTION_SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED.equals(intent.getAction())) return;
        if (Intent.ACTION_BOOT_COMPLETED.equals(intent.getAction()) && Rules.CHILD.equals(new Rules(context).mode())) {
            Rules.showChildNotice(context);
        }
        FocusTimer.Session session = new FocusTimer(context).current();
        if (session != null && !session.paused() && !session.completed()) {
            FocusAlarm.schedule(context, session.remaining());
        }
    }
}
