package com.digitaldiscipline.android;

import android.animation.ValueAnimator;
import android.content.Context;
import android.graphics.Canvas;
import android.graphics.LinearGradient;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RadialGradient;
import android.graphics.RectF;
import android.graphics.Shader;
import android.view.View;
import android.view.animation.LinearInterpolator;

final class DisciplineArt extends View {
    private static final int GOLD = 0xffffbd73;
    private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Path path = new Path();
    private final String kind;
    private ValueAnimator animation;
    private float progress;

    DisciplineArt(Context context, String kind) {
        super(context);
        this.kind = kind;
        setImportantForAccessibility(IMPORTANT_FOR_ACCESSIBILITY_NO);
    }

    @Override protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        if (!("hero".equals(kind) || kind.startsWith("scene-")) || !ValueAnimator.areAnimatorsEnabled()) return;
        animation = ValueAnimator.ofFloat(0, 1);
        animation.setDuration(7200);
        animation.setRepeatCount(ValueAnimator.INFINITE);
        animation.setInterpolator(new LinearInterpolator());
        animation.addUpdateListener(frame -> {
            progress = (float) frame.getAnimatedValue();
            invalidate();
        });
        animation.start();
    }

    @Override protected void onDetachedFromWindow() {
        if (animation != null) {
            animation.cancel();
            animation = null;
        }
        super.onDetachedFromWindow();
    }

    @Override protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);
        if ("hero".equals(kind)) {
            canvas.save();
            float scale = Math.min(getWidth() / 330f, getHeight() / 310f);
            canvas.translate(getWidth() / 2f, getHeight() / 2f);
            canvas.scale(scale, scale);
            drawHero(canvas);
            canvas.restore();
        } else if (kind.startsWith("scene-")) {
            canvas.save();
            float scale = Math.min(getWidth() / 118f, getHeight() / 132f);
            canvas.translate(getWidth() / 2f, getHeight() / 2f);
            canvas.scale(scale, scale);
            drawScene(canvas);
            canvas.restore();
        } else {
            float size = Math.min(getWidth(), getHeight());
            canvas.save();
            canvas.translate((getWidth() - size) / 2, (getHeight() - size) / 2);
            canvas.scale(size / 24f, size / 24f);
            drawIcon(canvas);
            canvas.restore();
        }
    }

    private void drawScene(Canvas c) {
        int tint = switch (kind) {
            case "scene-Blocking" -> 0xffffb6a4;
            case "scene-Settings" -> 0xffa9e8d5;
            default -> 0xffd4bbff;
        };
        double turn = progress * Math.PI * 2;
        float bob = 5 * (float) Math.sin(turn);
        fill(tint);
        paint.setShader(new RadialGradient(0, 5, 61,
            new int[]{(tint & 0xffffff) | 0x5a000000, (tint & 0xffffff) | 0x16000000, 0},
            null, Shader.TileMode.CLAMP));
        c.drawCircle(0, 5, 62, paint);
        paint.setShader(null);
        c.save();
        c.rotate(-26);
        RectF orbit = new RectF(-53, -26, 53, 26);
        stroke((tint & 0xffffff) | 0x52000000, 1.2f);
        c.drawOval(orbit, paint);
        stroke(tint, 1.65f);
        c.drawArc(orbit, progress * 360, 47, false, paint);
        fill(tint);
        c.drawCircle(53 * (float) Math.cos(turn), 26 * (float) Math.sin(turn), 2.7f, paint);
        c.restore();

        c.save();
        c.translate(0, bob);
        c.rotate(-12);
        fill(0x66100a1c);
        c.drawRoundRect(-31, -32, 37, 44, 16, 16, paint);
        fill(tint);
        paint.setShader(new LinearGradient(-30, -40, 30, 37, tint, 0xff413255, Shader.TileMode.CLAMP));
        c.drawRoundRect(-34, -39, 34, 39, 17, 17, paint);
        paint.setShader(null);
        stroke(0xbafdf8ff, 1.1f);
        c.drawRoundRect(-33, -38, 33, 38, 16, 16, paint);
        fill(0xff211831);
        c.drawRoundRect(-26, -30, 26, 30, 12, 12, paint);
        stroke(tint, 2.3f);
        if ("scene-Blocking".equals(kind)) {
            path.reset();
            path.moveTo(0, -18); path.lineTo(16, -11); path.lineTo(14, 4);
            path.quadTo(10, 14, 0, 19); path.quadTo(-10, 14, -14, 4);
            path.lineTo(-16, -11); path.close();
            c.drawPath(path, paint);
            c.drawLine(-5, 0, -1, 5, paint);
            c.drawLine(-1, 5, 7, -5, paint);
        } else if ("scene-Settings".equals(kind)) {
            for (int i = 0; i < 3; i++) {
                float y = -12 + i * 12;
                c.drawLine(-15, y, 15, y, paint);
                fill(tint);
                c.drawCircle(i == 1 ? 6 : -5, y, 3.5f, paint);
                stroke(tint, 2.3f);
            }
        } else {
            c.drawCircle(0, 0, 17, paint);
            c.drawArc(-21, -21, 21, 21, -95, 76, false, paint);
            c.drawLine(0, 0, 0, -10, paint);
            c.drawLine(0, 0, 7, 4, paint);
        }
        c.restore();
        fill(tint);
        c.drawCircle(-43, -38, 2, paint);
        c.drawCircle(43, 34, 2.8f, paint);
        c.drawCircle(38, -42, 1.4f, paint);
    }

    private void drawHero(Canvas c) {
        fill(GOLD);
        paint.setShader(new RadialGradient(0, 0, 158,
            new int[]{0x555c3721, 0x273e291f, 0x000b0b0d}, null, Shader.TileMode.CLAMP));
        c.drawCircle(0, 0, 158, paint);
        paint.setShader(null);

        c.save();
        c.rotate(-24);
        stroke(0x4499643c, 1.1f);
        RectF orbit = new RectF(-148, -84, 148, 84);
        c.drawOval(orbit, paint);
        stroke(GOLD, 1.7f);
        c.drawArc(orbit, 200 + progress * 360, 36, false, paint);
        double angle = (236 + progress * 360) * Math.PI / 180;
        fill(GOLD);
        c.drawCircle(148 * (float) Math.cos(angle), 84 * (float) Math.sin(angle), 3.2f, paint);
        c.restore();

        float bob = 6 * (float) Math.sin(progress * Math.PI * 2);
        c.save();
        c.translate(0, bob);
        c.rotate(-11);
        fill(0x66000000);
        c.drawOval(-100, 75, 106, 117, paint);
        fill(0xff16151a);
        c.drawCircle(7, 13, 93, paint);
        paint.setShader(new LinearGradient(-75, -85, 82, 78,
            new int[]{0xff625449, 0xff242227, 0xff141418}, null, Shader.TileMode.CLAMP));
        c.drawCircle(0, 0, 94, paint);
        paint.setShader(null);
        stroke(0xff82705e, 1.2f);
        c.drawCircle(0, 0, 93, paint);
        fill(0xff151518);
        c.drawCircle(0, 0, 78, paint);
        stroke(0xff363137, 1);
        c.drawCircle(0, 0, 76, paint);
        for (int i = 0; i < 60; i++) {
            c.save();
            c.rotate(i * 6);
            stroke(i % 5 == 0 ? 0xffa6988d : 0xff4b4446, i % 5 == 0 ? 2f : 1f);
            c.drawLine(0, -66, 0, i % 5 == 0 ? -58 : -62, paint);
            c.restore();
        }
        stroke(GOLD, 4);
        c.drawArc(-69, -69, 69, 69, -88, 67, false, paint);
        stroke(0xfff5e6db, 5);
        c.drawLine(0, 0, 0, -40, paint);
        c.drawLine(0, 0, 29, 17, paint);
        c.save();
        c.rotate(progress * 360);
        stroke(GOLD, 1.8f);
        c.drawLine(0, 9, 0, -53, paint);
        c.restore();
        fill(GOLD);
        c.drawCircle(0, 0, 5.4f, paint);
        c.restore();

        star(c, -116, -75, 7);
        star(c, 108, 74, 5);
        star(c, 113, -58, 3.5f);
    }

    private void star(Canvas c, float x, float y, float r) {
        stroke(0xffc0895c, 1.1f);
        c.drawLine(x - r, y, x + r, y, paint);
        c.drawLine(x, y - r, x, y + r, paint);
    }

    private void drawIcon(Canvas c) {
        stroke(0xfff3e8df, 1.65f);
        path.reset();
        switch (kind) {
            case "Activity":
                path.moveTo(3, 17); path.lineTo(3, 20); path.lineTo(21, 20);
                path.moveTo(5, 13); path.lineTo(9, 8); path.lineTo(13, 12); path.lineTo(20, 5);
                break;
            case "Blocking":
                c.drawRoundRect(5, 10, 19, 21, 3, 3, paint);
                c.drawArc(8, 3, 16, 12, 180, 180, false, paint);
                c.drawLine(8, 7, 8, 10, paint); c.drawLine(16, 7, 16, 10, paint);
                c.drawLine(12, 14, 12, 17, paint);
                break;
            case "Settings":
                c.drawCircle(12, 12, 7.5f, paint);
                c.drawCircle(12, 12, 2.5f, paint);
                for (int i = 0; i < 8; i++) {
                    c.save(); c.rotate(i * 45, 12, 12);
                    c.drawLine(12, 2, 12, 4.5f, paint); c.restore();
                }
                break;
            case "pencil":
                stroke(GOLD, 1.65f);
                path.moveTo(4, 20); path.lineTo(5.2f, 15.3f); path.lineTo(16.7f, 3.8f);
                path.lineTo(20.2f, 7.3f); path.lineTo(8.7f, 18.8f); path.close();
                path.moveTo(14.5f, 6); path.lineTo(18, 9.5f);
                break;
            default:
                c.drawCircle(12, 12, 8.5f, paint);
                path.moveTo(12, 6.5f); path.lineTo(12, 12); path.lineTo(16, 14.5f);
                break;
        }
        c.drawPath(path, paint);
    }

    private void fill(int color) {
        paint.setStyle(Paint.Style.FILL);
        paint.setColor(color);
    }

    private void stroke(int color, float width) {
        paint.setStyle(Paint.Style.STROKE);
        paint.setColor(color);
        paint.setStrokeWidth(width);
        paint.setStrokeCap(Paint.Cap.ROUND);
        paint.setStrokeJoin(Paint.Join.ROUND);
    }
}
