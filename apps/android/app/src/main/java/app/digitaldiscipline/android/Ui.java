package app.digitaldiscipline.android;

import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Color;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.graphics.drawable.RippleDrawable;
import android.view.Gravity;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

final class Ui {
    static final int BG = Color.rgb(11, 11, 13);
    static final int WHITE = Color.rgb(242, 242, 244);
    static final int MUTED = Color.rgb(173, 173, 182);
    static final int PANEL = Color.rgb(23, 23, 27);
    static final int LINE = Color.rgb(57, 57, 65);

    private Ui() {}

    static int dp(Context context, int value) {
        return Math.round(value * context.getResources().getDisplayMetrics().density);
    }

    static GradientDrawable surface(Context context, int color, int border, int radius) {
        GradientDrawable shape = new GradientDrawable();
        shape.setColor(color);
        shape.setCornerRadius(dp(context, radius));
        shape.setStroke(dp(context, 1), border);
        return shape;
    }

    static TextView text(Context context, String value, int size, int color, boolean bold) {
        TextView text = new TextView(context);
        text.setText(value);
        text.setTextSize(size);
        text.setTextColor(color);
        if (bold) text.setTypeface(Typeface.create("sans-serif-medium", Typeface.NORMAL));
        return text;
    }

    static LinearLayout column(Context context) {
        LinearLayout column = new LinearLayout(context);
        column.setOrientation(LinearLayout.VERTICAL);
        return column;
    }

    static LinearLayout card(Context context) {
        LinearLayout card = column(context);
        card.setBackground(surface(context, PANEL, LINE, 14));
        int pad = dp(context, 16);
        card.setPadding(pad, pad, pad, pad);
        return card;
    }

    static void add(LinearLayout parent, android.view.View view, int top) {
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(
            LinearLayout.LayoutParams.MATCH_PARENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        lp.topMargin = dp(parent.getContext(), top);
        parent.addView(view, lp);
    }

    static void body(LinearLayout parent, String value) {
        TextView label = text(parent.getContext(), value, 14, MUTED, false);
        label.setLineSpacing(dp(parent.getContext(), 2), 1);
        add(parent, label, 7);
    }

    static Button button(Context context, String label, boolean primary, Runnable tap) {
        Button button = new Button(context);
        button.setText(label);
        button.setAllCaps(false);
        button.setTextSize(14);
        button.setTextColor(primary ? BG : WHITE);
        button.setGravity(Gravity.CENTER);
        button.setMinimumHeight(dp(context, 48));
        button.setMinHeight(dp(context, 48));
        button.setBackground(new RippleDrawable(ColorStateList.valueOf(primary ? 0x24000000 : 0x33FFFFFF),
            surface(context, primary ? WHITE : 0xFF28282E, primary ? WHITE : LINE, 9), null));
        button.setOnClickListener(v -> tap.run());
        return button;
    }

    static void action(LinearLayout parent, String label, boolean primary, Runnable tap) {
        add(parent, button(parent.getContext(), label, primary, tap), 10);
    }
}
