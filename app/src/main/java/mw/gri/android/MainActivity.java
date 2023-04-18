package mw.gri.android;

import android.graphics.Color;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {

    static {
        System.loadLibrary("grin_android");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
//        getDisplayCutouts();
        super.onCreate(savedInstanceState);
    }

    public int[] getDisplayCutouts() {
        return Utils.getDisplayCutouts(this);
    }
}