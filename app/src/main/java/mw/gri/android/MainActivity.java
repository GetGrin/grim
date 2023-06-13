package mw.gri.android;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.hardware.SensorManager;
import android.os.Bundle;
import android.os.Process;
import android.system.ErrnoException;
import android.system.Os;
import android.view.KeyEvent;
import android.view.OrientationEventListener;
import com.google.androidgamesdk.GameActivity;

import java.util.concurrent.atomic.AtomicBoolean;

public class MainActivity extends GameActivity {

    public static String FINISH_ACTIVITY_ACTION = "MainActivity.finish";

    static {
        System.loadLibrary("grim");
    }

    private final BroadcastReceiver mBroadcastReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context ctx, Intent i) {
            if (i.getAction().equals(FINISH_ACTIVITY_ACTION)) {
                onExit();
            }
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Setup HOME environment variable for native code configurations.
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
        super.onCreate(null);

        // Callback to update display cutouts at native code.
        OrientationEventListener orientationEventListener = new OrientationEventListener(this,
                SensorManager.SENSOR_DELAY_NORMAL) {
            @Override
            public void onOrientationChanged(int orientation) {
                onDisplayCutoutsChanged(Utils.getDisplayCutouts(MainActivity.this));
            }
        };
        if (orientationEventListener.canDetectOrientation()) {
            orientationEventListener.enable();
        }
        onDisplayCutoutsChanged(Utils.getDisplayCutouts(this));

        // Register receiver to finish activity from the BackgroundService.
        registerReceiver(mBroadcastReceiver, new IntentFilter(FINISH_ACTIVITY_ACTION));

        // Start notification service.
        BackgroundService.start(this);
    }

    native void onDisplayCutoutsChanged(int[] cutouts);

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK)   {
            onBackButtonPress();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    public native void onBackButtonPress();

    private boolean mManualExit;
    private final AtomicBoolean mActivityDestroyed = new AtomicBoolean(false);

    @Override
    protected void onDestroy() {
        unregisterReceiver(mBroadcastReceiver);

        if (!mManualExit) {
            onTermination();
        }
        // Temp fix: kill process after 3 seconds to prevent app hanging at next launch.
        new Thread(() -> {
            try {
                Thread.sleep(3000);
                if (!mActivityDestroyed.get()) {
                    Process.killProcess(Process.myPid());
                }
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        }).start();
        super.onDestroy();
        mActivityDestroyed.set(true);
    }

    // Called from native code.
    public void onExit() {
        // Return if exit was already requested.
        if (mManualExit) {
            return;
        }
        mManualExit = true;
        BackgroundService.stop(this);
        finish();
    }

    public native void onTermination();
}