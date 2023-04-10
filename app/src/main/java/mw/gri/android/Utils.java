package mw.gri.android;

import android.annotation.SuppressLint;
import android.content.Context;
import android.content.res.Resources;
import android.graphics.Point;
import android.os.Build;
import android.view.Display;
import android.view.WindowInsets;
import android.view.WindowManager;

public class Utils {

    public static int getStatusBarHeight(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        if (Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
            return windowManager
                    .getCurrentWindowMetrics()
                    .getWindowInsets()
                    .getInsets(WindowInsets.Type.navigationBars())
                    .bottom;
        } else {
            Resources res = context.getResources();
            int statusBarHeight = 24;
            @SuppressLint({"DiscouragedApi", "InternalInsetResource"})
            int resourceId = res.getIdentifier("status_bar_height", "dimen", "android");
            if (resourceId > 0) {
                statusBarHeight = res.getDimensionPixelSize(resourceId);
            }
            return statusBarHeight;
        }
    }

    public static int getNavigationBarHeight(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        if (Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
            return windowManager
                    .getCurrentWindowMetrics()
                    .getWindowInsets()
                    .getInsets(WindowInsets.Type.navigationBars())
                    .bottom;
        } else {
            Point appUsableSize = getAppUsableScreenSize(context);
            Point realScreenSize = getRealScreenSize(context);

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

    private static Point getAppUsableScreenSize(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        Point size = new Point();
        windowManager.getDefaultDisplay().getSize(size);
        return size;
    }

    private static Point getRealScreenSize(Context context) {
        WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
        Display display = windowManager.getDefaultDisplay();
        Point size = new Point();
        display.getRealSize(size);
        return size;
    }
}
