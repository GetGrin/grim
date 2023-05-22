package mw.gri.android;

import android.os.Bundle;
import android.os.Process;
import android.system.ErrnoException;
import android.system.Os;
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
        BackgroundService.start(getApplicationContext());
    }

    @Override
    protected void onDestroy() {
        if (!mManualExit) {
            BackgroundService.stop(getApplicationContext());
            // Temporary fix to prevent app hanging when closed from recent apps
            Process.killProcess(Process.myPid());
        }
        super.onDestroy();
    }

    public int[] getDisplayCutouts() {
        return Utils.getDisplayCutouts(this);
    }

    private boolean mManualExit = false;

    // Called from native code
    public void onExit() {
        mManualExit = true;
        BackgroundService.stop(getApplicationContext());
        finish();
    }
}