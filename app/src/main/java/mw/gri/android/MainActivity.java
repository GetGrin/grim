package mw.gri.android;

import android.content.Context;
import android.graphics.Color;
import android.graphics.Point;
import android.os.Build;
import android.os.Bundle;
import android.system.ErrnoException;
import android.system.Os;
import android.view.Display;
import android.view.WindowInsets;
import android.view.WindowManager;
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

        super.onCreate(savedInstanceState);

        findViewById(android.R.id.content).setBackgroundColor(Color.BLACK);
        findViewById(android.R.id.content).setPadding(0, 0, 0, getNavigationBarHeight());
    }

    public int getNavigationBarHeight() {
        WindowManager windowManager = (WindowManager) getSystemService(Context.WINDOW_SERVICE);
        if (Build.VERSION.SDK_INT >= 30) {
            return windowManager
                    .getCurrentWindowMetrics()
                    .getWindowInsets()
                    .getInsets(WindowInsets.Type.navigationBars())
                    .bottom;
        } else {
            Point appUsableSize = getAppUsableScreenSize(this);
            Point realScreenSize = getRealScreenSize(this);

            // navigation bar on the side
            if (appUsableSize.x < realScreenSize.x) {
                return appUsableSize.y;
            }

            // navigation bar at the bottom
            if (appUsableSize.y < realScreenSize.y) {
                return realScreenSize.y - appUsableSize.y;
            }

            // navigation bar is not present
            return 0;
        }
    }

    public Point getAppUsableScreenSize(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        Point size = new Point();
        windowManager.getDefaultDisplay().getSize(size);
        return size;
    }

    public Point getRealScreenSize(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        Display display = windowManager.getDefaultDisplay();
        Point size = new Point();
        display.getRealSize(size);
        return size;
    }
}