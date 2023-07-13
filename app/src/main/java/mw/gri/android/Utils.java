package mw.gri.android;

import android.content.Context;

public class Utils {
    // Convert Pixels to DensityPixels
    public static int pxToDp(int px, Context context) {
        return (int) (px / context.getResources().getDisplayMetrics().density);
    }
}
