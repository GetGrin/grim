package mw.gri.android;

import android.hardware.SensorManager;
import android.os.Bundle;
import android.os.Process;
import android.system.ErrnoException;
import android.system.Os;
import android.view.KeyEvent;
import android.view.OrientationEventListener;
import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {

    static {
        System.loadLibrary("grim");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
        super.onCreate(savedInstanceState);

        OrientationEventListener orientationEventListener = new OrientationEventListener(this,
                SensorManager.SENSOR_DELAY_GAME) {
            @Override
            public void onOrientationChanged(int orientation) {
                onDisplayCutoutsChanged(Utils.getDisplayCutouts(MainActivity.this));
            }
        };
        if (orientationEventListener.canDetectOrientation()) {
            orientationEventListener.enable();
        }
        onDisplayCutoutsChanged(Utils.getDisplayCutouts(MainActivity.this));

        BackgroundService.start(getApplicationContext());
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

    @Override
    protected void onDestroy() {
        if (!mManualExit) {
            BackgroundService.stop(getApplicationContext());
            // Temporary fix to prevent app hanging when closed from recent apps
            Process.killProcess(Process.myPid());
        }
        super.onDestroy();
    }


    private boolean mManualExit = false;

    // Called from native code
    public void onExit() {
        mManualExit = true;
        BackgroundService.stop(getApplicationContext());
        finish();
    }
}