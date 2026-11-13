package com.digitaldiscipline.android;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.LinearGradient;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RadialGradient;
import android.graphics.Shader;
import android.view.View;

final class BrandMark extends View {
    private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Path mark = new Path();
    private final boolean glass;
    private final Shader glow = new RadialGradient(50, 50, 50,
        new int[]{0x725c35c9, 0x37382277, 0x00382277}, null, Shader.TileMode.CLAMP);
    private final Shader body = new LinearGradient(16, 14, 86, 87,
        new int[]{0xfff1d8ff, 0xff9c68ec, 0xff342650, 0xff507b81, 0xffb3ffdd},
        new float[]{0, .22f, .49f, .75f, 1}, Shader.TileMode.CLAMP);
    private final Shader rim = new LinearGradient(14, 12, 85, 90,
        new int[]{0xfff6ddff, 0xff9464e3, 0xffbcffe3}, null, Shader.TileMode.CLAMP);

    BrandMark(Context context, boolean glass) {
        super(context);
        this.glass = glass;
        setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);
        mark.setFillType(Path.FillType.EVEN_ODD);
        mark.addRoundRect(13, 13, 87, 87, 21, 21, Path.Direction.CW);
        mark.moveTo(34, 40);
        mark.cubicTo(43, 27, 57, 24, 68, 34);
        mark.cubicTo(73, 39, 76, 46, 76, 53);
        mark.lineTo(76, 65);
        mark.cubicTo(76, 71, 72, 73, 67, 69);
        mark.lineTo(34, 46);
        mark.cubicTo(32, 44, 32, 43, 34, 40);
        mark.close();
    }

    @Override protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);
        float size = Math.min(getWidth(), getHeight());
        if (size <= 0) return;
        canvas.save();
        canvas.translate((getWidth() - size) / 2, (getHeight() - size) / 2);
        canvas.scale(size / 100, size / 100);
        paint.setStyle(Paint.Style.FILL);
        paint.setColor(0xfff5f5fa);
        if (glass) {
            paint.setShader(glow);
            canvas.drawCircle(50, 50, 50, paint);
            paint.setShader(body);
        }
        canvas.drawPath(mark, paint);
        if (glass) {
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(.85f);
            paint.setShader(rim);
            canvas.drawPath(mark, paint);
        }
        paint.setShader(null);
        canvas.restore();
    }
}
