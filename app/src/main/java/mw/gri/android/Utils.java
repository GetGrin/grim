package mw.gri.android;

import android.app.Activity;
import android.content.Context;
import android.os.Build;
import android.view.DisplayCutout;
import android.view.WindowInsets;
import android.view.WindowManager;
import androidx.core.graphics.Insets;
import androidx.core.view.DisplayCutoutCompat;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;

public class Utils {

    public static int[] getDisplayCutouts(Activity context) {
        int[] cutouts = new int[]{0, 0, 0, 0};
        if (Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
            WindowManager windowManager = (WindowManager) context.getSystemService(Context.WINDOW_SERVICE);
            WindowInsets insets = windowManager.getCurrentWindowMetrics().getWindowInsets();
            android.graphics.Insets barsInsets = insets.getInsets(WindowInsets.Type.systemBars());
            android.graphics.Insets cutoutsInsets = insets.getInsets(WindowInsets.Type.displayCutout());
            cutouts[0] = pxToDp(Integer.max(barsInsets.top, cutoutsInsets.top), context);
            cutouts[1] = pxToDp(Integer.max(barsInsets.right, cutoutsInsets.right), context);
            cutouts[2] = pxToDp(Integer.max(barsInsets.bottom, cutoutsInsets.bottom), context);
            cutouts[3] = pxToDp(Integer.max(barsInsets.left, cutoutsInsets.left), context);
        } else if (Build.VERSION.SDK_INT == android.os.Build.VERSION_CODES.Q) {
            DisplayCutout displayCutout = context.getWindowManager().getDefaultDisplay().getCutout();
            cutouts[0] = displayCutout.getSafeInsetBottom();
            cutouts[1] = displayCutout.getSafeInsetRight();
            cutouts[2] = displayCutout.getSafeInsetBottom();
            cutouts[3] = displayCutout.getSafeInsetLeft();
        } else {
            WindowInsetsCompat insets = ViewCompat.getRootWindowInsets(context.getWindow().getDecorView());
            if (insets != null) {
                DisplayCutoutCompat displayCutout = insets.getDisplayCutout();
                Insets systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars());
                if (displayCutout != null) {
                    cutouts[0] = pxToDp(Integer.max(displayCutout.getSafeInsetTop(), systemBars.top), context);
                    cutouts[1] = pxToDp(Integer.max(displayCutout.getSafeInsetRight(), systemBars.right), context);
                    cutouts[2] = pxToDp(Integer.max(displayCutout.getSafeInsetBottom(), systemBars.bottom), context);
                    cutouts[3] = pxToDp(Integer.max(displayCutout.getSafeInsetLeft(), systemBars.left), context);
                }
            }
        }
        return cutouts;
    }

    private static int pxToDp(int px, Context context) {
        return (int) (px / context.getResources().getDisplayMetrics().density);
    }
}
