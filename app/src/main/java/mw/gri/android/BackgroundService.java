package mw.gri.android;

import android.app.*;
import android.content.Context;
import android.content.Intent;
import android.os.*;
import androidx.annotation.Nullable;
import androidx.core.app.NotificationCompat;

import java.util.List;

public class BackgroundService extends Service {

    private static final String TAG = BackgroundService.class.getSimpleName();
    
    private PowerManager.WakeLock mWakeLock;

    private final Handler mHandler = new Handler(Looper.getMainLooper());
    private boolean mStopped = false;

    private static final int SYNC_STATUS_NOTIFICATION_ID = 1;
    private NotificationCompat.Builder mNotificationBuilder;

    private final Runnable mUpdateSyncStatus = new Runnable() {
        @Override
        public void run() {
            if (mStopped) {
                return;
            }
            // Update sync status at notification.
            mNotificationBuilder.setContentText(getSyncStatusText());
            NotificationManager manager = getSystemService(NotificationManager.class);
            manager.notify(SYNC_STATUS_NOTIFICATION_ID, mNotificationBuilder.build());
            // Send broadcast to MainActivity if app exit is needed after node stop.
            if (exitAppAfterNodeStop()) {
                sendBroadcast(new Intent(MainActivity.FINISH_ACTIVITY_ACTION));
                mStopped = true;
            }
            // Repeat notification update if service is not stopped.
            if (!mStopped) {
                mHandler.postDelayed(this, 500);
            }
        }
    };

    @Override
    public void onCreate() {
        // Prevent CPU to sleep at background.
        PowerManager pm = (PowerManager) getSystemService(Context.POWER_SERVICE);
        mWakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, TAG);
        mWakeLock.acquire();
        // Create channel to show notifications.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel notificationChannel = new NotificationChannel(
                    TAG, TAG, NotificationManager.IMPORTANCE_LOW
            );

            NotificationManager manager = getSystemService(NotificationManager.class);
            manager.createNotificationChannel(notificationChannel);
        }
        // Show notification with sync status.
        Intent i = getPackageManager().getLaunchIntentForPackage(this.getPackageName());
        PendingIntent pendingIntent = PendingIntent.getActivity(this, 0, i, PendingIntent.FLAG_IMMUTABLE);
        mNotificationBuilder = new NotificationCompat.Builder(this, TAG)
                .setContentTitle(this.getSyncTitle())
                .setContentText(this.getSyncStatusText())
                .setSmallIcon(R.drawable.ic_stat_name)
                .setContentIntent(pendingIntent);
        Notification notification = mNotificationBuilder.build();
        // Start service at foreground state to prevent killing by system.
        startForeground(SYNC_STATUS_NOTIFICATION_ID, notification);
        // Update sync status at notification.
        mHandler.post(mUpdateSyncStatus);
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        return START_STICKY;
    }

    @Override
    public void onTaskRemoved(Intent rootIntent) {
        onStop();
        super.onTaskRemoved(rootIntent);
    }

    @Override
    public void onDestroy() {
        onStop();
        super.onDestroy();
    }

    @Nullable
    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    public void onStop() {
        mStopped = true;
        // Stop updating the notification.
        mHandler.removeCallbacks(mUpdateSyncStatus);
        // Remove service from foreground state.
        stopForeground(Service.STOP_FOREGROUND_REMOVE);
        // Remove notification.
        NotificationManager notificationManager = getSystemService(NotificationManager.class);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            notificationManager.deleteNotificationChannel(TAG);
        }
        notificationManager.cancel(SYNC_STATUS_NOTIFICATION_ID);
        // Release wake lock to allow CPU to sleep at background.
        if (mWakeLock.isHeld()) {
            mWakeLock.release();
            mWakeLock = null;
        }
    }

    // Start the service.
    public static void start(Context context) {
        if (!isServiceRunning(context)) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(new Intent(context, BackgroundService.class));
            } else {
                context.startService(new Intent(context, BackgroundService.class));
            }
        }
    }

    // Stop the service.
    public static void stop(Context context) {
        context.stopService(new Intent(context, BackgroundService.class));
    }

    // Check if service is running.
    private static boolean isServiceRunning(Context context) {
        ActivityManager activityManager = (ActivityManager) context.getSystemService(Context.ACTIVITY_SERVICE);
        List<ActivityManager.RunningServiceInfo> services = activityManager.getRunningServices(Integer.MAX_VALUE);

        for (ActivityManager.RunningServiceInfo runningServiceInfo : services) {
            if (runningServiceInfo.service.getClassName().equals(BackgroundService.class.getName())) {
                return true;
            }
        }

        return false;
    }

    // Get sync status text for notification from native code.
    private native String getSyncStatusText();
    // Get sync title text for notification from native code.
    private native String getSyncTitle();
    // Check if exit app is needed after node stop from native code.
    private native boolean exitAppAfterNodeStop();
}
