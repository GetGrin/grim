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
            mNotificationBuilder.setContentText(getSyncStatusText());
            NotificationManager manager = getSystemService(NotificationManager.class);
            manager.notify(SYNC_STATUS_NOTIFICATION_ID, mNotificationBuilder.build());

            if (exitAppAfterNodeStop()) {
                sendBroadcast(new Intent(MainActivity.FINISH_ACTIVITY_ACTION));
                mStopped = true;
            }

            if (!mStopped) {
                mHandler.postDelayed(this, 300);
            }
        }
    };

    @Override
    public void onCreate() {
        PowerManager pm = (PowerManager) getSystemService(Context.POWER_SERVICE);
        mWakeLock = pm.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, TAG);
        mWakeLock.acquire();

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel notificationChannel = new NotificationChannel(
                    TAG, TAG, NotificationManager.IMPORTANCE_LOW
            );

            NotificationManager manager = getSystemService(NotificationManager.class);
            manager.createNotificationChannel(notificationChannel);
        }

        Intent i = getPackageManager().getLaunchIntentForPackage(this.getPackageName());
        PendingIntent pendingIntent = PendingIntent.getActivity(this, 0, i, PendingIntent.FLAG_IMMUTABLE);
        mNotificationBuilder = new NotificationCompat.Builder(this, TAG)
                .setContentTitle(this.getSyncTitle())
                .setContentText(this.getSyncStatusText())
                .setSmallIcon(R.drawable.ic_stat_name)
                .setContentIntent(pendingIntent);
        Notification notification = mNotificationBuilder.build();
        startForeground(SYNC_STATUS_NOTIFICATION_ID, notification);

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

        stopForeground(Service.STOP_FOREGROUND_REMOVE);

        if (mWakeLock.isHeld()) {
            mWakeLock.release();
            mWakeLock = null;
        }

        mHandler.removeCallbacks(mUpdateSyncStatus);
    }

    public static void start(Context context) {
        if (!isServiceRunning(context)) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(new Intent(context, BackgroundService.class));
            } else {
                context.startService(new Intent(context, BackgroundService.class));
            }
        }
    }

    public static void stop(Context context) {
        context.stopService(new Intent(context, BackgroundService.class));
    }

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

    private native String getSyncStatusText();
    private native String getSyncTitle();
    private native boolean exitAppAfterNodeStop();
}
