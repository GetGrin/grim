package mw.gri.android;

import android.content.Intent;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import com.google.androidgamesdk.GameActivity;

public class MainActivity extends GameActivity {

    static {
        System.loadLibrary("grim_android");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
        super.onCreate(savedInstanceState);
    }

    public int[] getDisplayCutouts() {
        return Utils.getDisplayCutouts(this);
    }
}