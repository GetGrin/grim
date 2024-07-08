package mw.gri.android;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;

public class NotificationActionsReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent i) {
        String a = i.getAction();
        if (a.equals(BackgroundService.ACTION_START_NODE)) {
            startNode();
            context.sendBroadcast(new Intent(BackgroundService.ACTION_REFRESH));
        } else if (a.equals(BackgroundService.ACTION_STOP_NODE)) {
            stopNode();
            context.sendBroadcast(new Intent(BackgroundService.ACTION_REFRESH));
        } else {
            if (isNodeRunning()) {
                stopNodeToExit();
                context.sendBroadcast(new Intent(BackgroundService.ACTION_REFRESH));
            } else {
                context.sendBroadcast(new Intent(MainActivity.STOP_APP_ACTION));
            }
        }
    }

    // Start integrated node.
    native void startNode();
    // Stop integrated node.
    native void stopNode();
    // Stop node and exit from the app.
    native void stopNodeToExit();
    // Check if node is running.
    native boolean isNodeRunning();
}
