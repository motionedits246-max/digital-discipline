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
import org.json.JSONObject;

import java.time.LocalDate;
import java.time.format.TextStyle;
import java.util.Locale;
import java.util.function.LongFunction;

final class WeekChart extends LinearLayout {
    private static final int MINT = 0xffa5e7d0;
    private static final int[] LEVELS = {0xff25342f, 0xff35564a, 0xff47806d, 0xff6eb39b, MINT};

    WeekChart(Context context, JSONArray week, LongFunction<String> duration, java.util.function.LongConsumer selectDay) {
        super(context);
        setOrientation(VERTICAL);
        TextView period = label("Tap a day to see usage", 13);
        String[] days = new String[7];
        long[] totals = new long[7];
        int[] counts = new int[7];
        int[] expected = new int[7];
        long maximum = 1;
        LinearLayout heading = new LinearLayout(context);
        heading.addView(label("", 11), new LayoutParams(dp(39), -2));
        for (int i = 0; i < 6; i++) {
            TextView hour = label(String.format(Locale.ROOT, "%02d", i * 4), 11);
            hour.setGravity(Gravity.CENTER);
            heading.addView(hour, new LayoutParams(0, dp(22), 1));
        }
        addView(heading);
        View[] selected = new View[1];
        for (int day = 0; day < 7; day++) {
            JSONObject entry = week.optJSONObject(day);
            String name = dayLabel(entry, day);
            days[day] = name;
            JSONArray bins = entry == null ? null : entry.optJSONArray("bins");
            boolean hourly = bins != null && bins.length() >= 23;
            expected[day] = bins == null ? 6 : bins.length();
            if (hourly) {
                addHours(context, entry, bins, name, duration, selectDay);
                for (int slot = 0; slot < bins.length(); slot++) {
                    if (!bins.isNull(slot)) counts[day]++;
                    totals[day] += bins.optLong(slot);
                }
                maximum = Math.max(maximum, totals[day]);
                continue;
            }
            LinearLayout row = new LinearLayout(context);
            row.setGravity(Gravity.CENTER_VERTICAL);
            row.addView(label(name, 12), new LayoutParams(dp(39), -2));
            for (int bin = 0; bin < 6; bin++) {
                boolean known = bins != null && !bins.isNull(bin);
                long used = known ? Math.min(18_000_000L, Math.max(0L, bins.optLong(bin))) : 0;
                if (known) counts[day]++;
                totals[day] += used;
                int level = (int) Math.min(4, (used + 1_799_999L) / 1_800_000L);
                String time = String.format(Locale.ROOT, "%02d:00–%02d:00", bin * 4, (bin + 1) * 4);
                Mark mark = new Mark(context, LEVELS[level], known, 1);
                mark.setContentDescription(name + " " + time + " · " + (known ? duration.apply(used) : "No observation"));
                mark.setFocusable(true);
                mark.setOnClickListener(view -> {
                    if (selected[0] != null) selected[0].setSelected(false);
                    selected[0] = mark;
                    mark.setSelected(true);
                    period.setText(mark.getContentDescription());
                });
                row.addView(mark, new LayoutParams(0, dp(34), 1));
            }
            addView(row, new LayoutParams(-1, -2));
            maximum = Math.max(maximum, totals[day]);
        }
        addView(label("Less → More usage   ·   Hatched = no observation", 11));
        addView(period, new LayoutParams(-1, dp(40)));
        TextView daily = label("Daily screen time", 17);
        daily.setPadding(0, dp(8), 0, dp(8));
        addView(daily);
        addView(label("Chart maximum · " + duration.apply(maximum <= 1 ? 0 : maximum), 11));
        LinearLayout columns = new LinearLayout(context);
        TextView dayDetail = label("Tap a day for its recorded total", 13);
        for (int day = 0; day < 7; day++) {
            LinearLayout column = new LinearLayout(context);
            column.setOrientation(VERTICAL);
            column.setGravity(Gravity.CENTER);
            columns.addView(column, new LayoutParams(0, -2, 1));
            boolean known = counts[day] > 0;
            Mark bar = new Mark(context, MINT, known, totals[day] / (float) maximum);
            String caption = days[day] + " · " + (known ? duration.apply(totals[day]) : "No observation");
            if (known && counts[day] < expected[day]) caption += " · partial coverage";
            bar.setContentDescription(caption);
            bar.setFocusable(true);
            JSONObject entry = week.optJSONObject(day);
            long start = entry == null ? 0 : entry.optLong("start");
            bar.setOnClickListener(view -> {
                if (start > 0) selectDay.accept(start);
                else dayDetail.setText(bar.getContentDescription());
            });
            column.addView(bar, new LayoutParams(-1, dp(94)));
            column.addView(label(days[day], 11), new LayoutParams(-2, -2));
            if (ValueAnimator.areAnimatorsEnabled()) {
                bar.setPivotY(dp(94));
                bar.setScaleY(.06f);
                bar.animate().scaleY(1).setDuration(280).setStartDelay(day * 18L).start();
            }
        }
        addView(columns);
        addView(dayDetail, new LayoutParams(-1, dp(36)));
    }

    private void addHours(Context context, JSONObject entry, JSONArray bins, String name,
                          LongFunction<String> duration, java.util.function.LongConsumer selectDay) {
        LinearLayout row = new LinearLayout(context);
        row.setGravity(Gravity.CENTER_VERTICAL);
        TextView day = label(name, 12);
        row.addView(day, new LayoutParams(dp(39), -2));
        for (int slot = 0; slot < bins.length(); slot++) {
            boolean known = !bins.isNull(slot);
            long used = Math.max(0L, bins.optLong(slot));
            int level = (int) Math.min(4, (used + 899_999L) / 900_000L);
            Mark mark = new Mark(context, LEVELS[level], known, 1);
            row.addView(mark, new LayoutParams(0, dp(32), 1));
        }
        row.setFocusable(true);
        int observed = 0;
        long total = 0;
        for (int slot = 0; slot < bins.length(); slot++) {
            if (!bins.isNull(slot)) observed++;
            total += bins.optLong(slot);
        }
        row.setContentDescription(name + " · " + (observed > 0 ? duration.apply(total) : "No observation")
            + " · Open day");
        row.setOnClickListener(view -> selectDay.accept(entry.optLong("start")));
        addView(row, new LayoutParams(-1, -2));
    }

    private String dayLabel(JSONObject entry, int index) {
        if (entry != null) {
            try {
                return LocalDate.parse(entry.optString("day")).getDayOfWeek()
                    .getDisplayName(TextStyle.SHORT, Locale.getDefault());
            } catch (java.time.DateTimeException ignored) {
                // An older or incomplete import may be missing this date.
            }
        }
        return Integer.toString(index + 1);
    }

    private TextView label(String text, int size) {
        TextView label = new TextView(getContext());
        label.setText(text);
        label.setTextColor(0xffe9e2ef);
        label.setTextSize(size);
        return label;
    }

    private int dp(float value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }

    private final class Mark extends View {
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final int color;
        private final boolean known;
        private final float proportion;

        Mark(Context context, int color, boolean known, float proportion) {
            super(context);
            this.color = color;
            this.known = known;
            this.proportion = proportion;
        }

        @Override protected void onDraw(Canvas canvas) {
            float x = Math.min(dp(3), getWidth() * .18f);
            float right = getWidth() - x;
            float bottom = getHeight() - dp(4);
            float top = known ? bottom - Math.max(dp(2), (bottom - dp(4)) * proportion)
                : (getHeight() > dp(40) ? bottom - dp(8) : dp(4));
            paint.setColor(known ? color : 0xff352e40);
            canvas.drawRoundRect(x, top, right, bottom, dp(4), dp(4), paint);
            if (!known) {
                canvas.save();
                canvas.clipRect(x, top, right, bottom);
                paint.setColor(0xff60546f);
                paint.setStrokeWidth(dp(1));
                for (float i = x - (bottom - top); i < right; i += dp(5)) {
                    canvas.drawLine(i, bottom, i + bottom - top, top, paint);
                }
                canvas.restore();
            }
            if (isSelected() || isFocused()) {
                paint.setStyle(Paint.Style.STROKE);
                paint.setStrokeWidth(dp(2));
                paint.setColor(0xffe9fff7);
                canvas.drawRoundRect(x, top, right, bottom, dp(4), dp(4), paint);
                paint.setStyle(Paint.Style.FILL);
            }
        }

        @Override public void setSelected(boolean selected) {
            super.setSelected(selected);
            invalidate();
        }
    }
}
