package com.digitaldiscipline.android;

import android.content.Intent;
import android.service.quicksettings.Tile;
import android.service.quicksettings.TileService;

public final class FocusTile extends TileService {
    @Override public void onStartListening() {
        super.onStartListening();
        refresh();
    }

    @Override public void onClick() {
        super.onClick();
        if (!Rules.INDIVIDUAL.equals(new Rules(this).mode())) return;
        if (isLocked()) unlockAndRun(this::toggle);
        else toggle();
    }

    private void toggle() {
        FocusTimer timer = new FocusTimer(this);
        FocusTimer.Session session = timer.current();
        if (session == null) timer.start(timer.plan(), "");
        else timer.finish(false);
        refresh();
    }

    private void refresh() {
        Tile tile = getQsTile();
        if (tile == null) return;
        boolean individual = Rules.INDIVIDUAL.equals(new Rules(this).mode());
        FocusTimer timer = new FocusTimer(this);
        FocusTimer.Session session = timer.current();
        tile.setState(!individual ? Tile.STATE_UNAVAILABLE : session != null && !session.completed()
            ? Tile.STATE_ACTIVE : Tile.STATE_INACTIVE);
        tile.setSubtitle(!individual ? "Set up Individual mode"
            : session == null ? "Start " + timer.plan().minutes() + " min"
            : session.completed() ? "Finish session" : "End session");
        tile.updateTile();
    }
}
