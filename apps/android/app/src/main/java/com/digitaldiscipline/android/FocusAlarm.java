package com.digitaldiscipline.android;

import android.app.AlarmManager;
import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.media.AudioAttributes;
import android.media.MediaPlayer;
import android.os.Build;
import android.os.IBinder;
import android.os.SystemClock;

public final class FocusAlarm extends Service {
    private static final String CHANNEL = "focus-completion";
    private static final String STOP = "stop-focus-sound";
    private static final String SETTINGS = "focus-sound";
    private static final int ID = 84;
    static final String[] LABELS = {"Soft chime", "Digital bell", "Pulse"};
    private static final int[] SOUNDS = {R.raw.focus_chime, R.raw.focus_bell, R.raw.focus_pulse};
    private MediaPlayer player;

    static int choice(Context context) {
        return context.getSharedPreferences(SETTINGS, MODE_PRIVATE).getInt("choice", 0);
    }

    static void select(Context context, int index) {
        context.getSharedPreferences(SETTINGS, MODE_PRIVATE).edit().putInt("choice", index).apply();
    }

    static MediaPlayer sound(Context context, int index) {
        return MediaPlayer.create(context, SOUNDS[index],
            new AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_ALARM)
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION).build(), 0);
    }

    static boolean precise(Context context) {
        return Build.VERSION.SDK_INT < 31 || context.getSystemService(AlarmManager.class).canScheduleExactAlarms();
    }

    private static PendingIntent event(Context context, String action, int flags) {
        return PendingIntent.getBroadcast(context, ID, new Intent(context, Event.class).setAction(action),
            flags | PendingIntent.FLAG_IMMUTABLE);
    }

    static void schedule(Context context, long remaining) {
        AlarmManager alarms = context.getSystemService(AlarmManager.class);
        PendingIntent event = event(context, "focus-finished", PendingIntent.FLAG_UPDATE_CURRENT);
        long at = SystemClock.elapsedRealtime() + remaining;
        if (precise(context)) {
            try {
                alarms.setExactAndAllowWhileIdle(AlarmManager.ELAPSED_REALTIME_WAKEUP, at, event);
                return;
            } catch (SecurityException ignored) {
                // The user may have revoked exact alarm access.
            }
        }
        alarms.setAndAllowWhileIdle(AlarmManager.ELAPSED_REALTIME_WAKEUP, at, event);
    }

    static void cancel(Context context) {
        PendingIntent event = event(context, "focus-finished", PendingIntent.FLAG_NO_CREATE);
        if (event != null) context.getSystemService(AlarmManager.class).cancel(event);
        stop(context);
    }

    static void silence(Context context) {
        context.getSharedPreferences(SETTINGS, MODE_PRIVATE).edit().putLong("silenced", deadline(context)).apply();
        stop(context);
    }

    private static long deadline(Context context) {
        return context.getSharedPreferences("focus", MODE_PRIVATE).getLong("wallEnd", 0);
    }

    static void checkCompletion(Context context) {
        FocusTimer.Session session = new FocusTimer(context).current();
        if (session != null && session.completed() && !session.paused()
                && context.getSharedPreferences(SETTINGS, MODE_PRIVATE).getLong("silenced", 0) != deadline(context)) {
            ring(context);
        }
    }

    static void stop(Context context) {
        context.stopService(new Intent(context, FocusAlarm.class));
    }

    static boolean ringing;
    static void ring(Context context) {
        if (ringing) return;
        try { context.startForegroundService(new Intent(context, FocusAlarm.class)); }
        catch (IllegalStateException | SecurityException ignored) {
            // Android may defer an inexact alarm's background service start.
        }
    }

    public static final class Event extends BroadcastReceiver {
        @Override public void onReceive(Context context, Intent intent) {
            if (STOP.equals(intent.getAction())) {
                silence(context);
                return;
            }
            checkCompletion(context);
        }
    }

    @Override public IBinder onBind(Intent intent) { return null; }

    @Override public int onStartCommand(Intent intent, int flags, int startId) {
        NotificationManager manager = getSystemService(NotificationManager.class);
        NotificationChannel channel = new NotificationChannel(CHANNEL, "Focus timer completion",
            NotificationManager.IMPORTANCE_LOW);
        channel.setSound(null, null);
        manager.createNotificationChannel(channel);
        startForeground(ID, new Notification.Builder(this, CHANNEL).setSmallIcon(R.drawable.ic_discipline)
            .setContentTitle("Focus timer finished").setContentText("Tap Stop to silence the timer.")
            .setOngoing(true).addAction(new Notification.Action.Builder(null, "Stop",
                event(this, STOP, PendingIntent.FLAG_UPDATE_CURRENT)).build()).build());
        FocusTimer.Session session = new FocusTimer(this).current();
        if (session == null || !session.completed() || session.paused()
                || getSharedPreferences(SETTINGS, MODE_PRIVATE).getLong("silenced", 0) == deadline(this)) {
            stopSelf();
            return START_NOT_STICKY;
        }
        if (player == null) {
            player = sound(this, choice(this));
            if (player == null) stopSelf();
            else {
                player.setLooping(true);
                player.setOnErrorListener((audio, what, extra) -> { stopSelf(); return true; });
                player.start();
                ringing = true;
            }
        }
        return START_NOT_STICKY;
    }

    @Override public void onDestroy() {
        ringing = false;
        if (player != null) player.release();
        stopForeground(STOP_FOREGROUND_REMOVE);
        super.onDestroy();
    }
}
