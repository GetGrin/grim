package mw.gri.android;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

import java.util.Objects;

public class NotificationActionsReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent i) {
        String a = i.getAction();
        if (Objects.equals(a, BackgroundService.ACTION_START_NODE)) {
            startNode();
        } else if (Objects.equals(a, BackgroundService.ACTION_STOP_NODE)) {
            stopNode();
        } else {
            stopNodeToExit();
        }
    }

    // Start integrated node.
    native void startNode();
    // Stop integrated node.
    native void stopNode();
    // Stop node and exit from the app.
    native void stopNodeToExit();
}
