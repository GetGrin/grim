package mw.gri.android;

import android.app.NativeActivity;
import android.content.res.Configuration;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.WindowManager;

public class MainActivity extends NativeActivity {

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
        super.onCreate(savedInstanceState);
    }
}