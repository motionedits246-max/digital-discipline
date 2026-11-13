package com.digitaldiscipline.android;

import android.animation.ValueAnimator;
import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.view.Gravity;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.TextView;

import org.json.JSONArray;

import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;

final class HourChart extends LinearLayout {
    private static final long HOUR = 3_600_000L;
    private final ArrayList<Bar> bars = new ArrayList<>();
    private long maximum = 1;
    private float growth = 1;
    private Bar selected;
    private Selection listener;

    interface Selection {
        void onHour(String label, long duration);
    }

    HourChart(Context context, JSONArray values, long first, long from, long end, ZoneId zone) {
        super(context);
        setOrientation(VERTICAL);
        LinearLayout row = new LinearLayout(context);
        LinearLayout labels = new LinearLayout(context);
        addView(row, new LayoutParams(-1, 0, 1));
        addView(labels, new LayoutParams(-1, -2));
        DateTimeFormatter format = DateTimeFormatter.ofPattern("HH:mm").withZone(zone);
        for (int i = 0; i < values.length(); i++) {
            long at = first + i * HOUR;
            if (at + HOUR <= from || at > end) continue;
            long duration = values.optLong(i);
            String label = format.format(Instant.ofEpochMilli(Math.max(from, at))) + "–"
                + format.format(Instant.ofEpochMilli(Math.min(end, at + HOUR)));
            Bar bar = new Bar(context, label, duration);
            maximum = Math.max(maximum, duration);
            bars.add(bar);
            row.addView(bar, new LayoutParams(0, -1, 1));
        }
        int count = Math.min(5, bars.size());
        for (int i = 0; i < count; i++) {
            int index = count == 1 ? 0 : Math.round((bars.size() - 1) * i / (float) (count - 1));
            TextView label = new TextView(context);
            label.setText(bars.get(index).label.substring(0, 5));
            label.setTextSize(11);
            label.setTextColor(0xffc6bad5);
            label.setGravity(i == 0 ? Gravity.START : i == count - 1 ? Gravity.END : Gravity.CENTER);
            label.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            labels.addView(label, new LayoutParams(0, -2, 1));
        }
        if (ValueAnimator.areAnimatorsEnabled()) {
            growth = 0;
            post(() -> {
                if (!isAttachedToWindow()) return;
                ValueAnimator rise = ValueAnimator.ofFloat(0, 1);
                rise.setDuration(390);
                rise.addUpdateListener(frame -> {
                    growth = (float) frame.getAnimatedValue();
                    for (Bar bar : bars) bar.invalidate();
                });
                rise.start();
            });
        }
    }

    void setOnHourSelected(Selection listener) { this.listener = listener; }

    private final class Bar extends View {
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final String label;
        private final long duration;

        Bar(Context context, String label, long duration) {
            super(context);
            this.label = label;
            this.duration = duration;
            setContentDescription(label + ", " + Math.round(duration / 60000f) + " minutes");
            setFocusable(true);
            setOnClickListener(view -> {
                if (selected != null) {
                    selected.setSelected(false);
                    selected.invalidate();
                }
                selected = this;
                setSelected(true);
                invalidate();
                if (listener != null) listener.onHour(label, duration);
            });
        }

        @Override protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            float bottom = getHeight() - 5;
            float height = duration == 0 ? 2 : 4 + (bottom - 10) * growth * duration / maximum;
            paint.setColor(isSelected() ? 0xffb0f8de : duration == 0 ? 0xff554766
                : duration == maximum ? 0xffffc5ab : 0xffa37bd7);
            float pad = Math.max(1, getWidth() * .16f);
            canvas.drawRoundRect(pad, bottom - height, getWidth() - pad, bottom, 4, 4, paint);
        }
    }
}
