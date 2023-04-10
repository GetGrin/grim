package mw.gri.android;

import android.app.NativeActivity;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;

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

        int navBarHeight = Utils.getNavigationBarHeight(getApplicationContext());
//        int statusBarHeight = Utils.getStatusBarHeight(getApplicationContext());
        findViewById(android.R.id.content).setPadding(0, 0, 0, navBarHeight);
    }
}