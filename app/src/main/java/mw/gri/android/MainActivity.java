package mw.gri.android;

import android.content.*;
import android.os.Bundle;
import android.os.Process;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.KeyEvent;
import android.view.View;
import androidx.core.graphics.Insets;
import androidx.core.view.DisplayCutoutCompat;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import com.google.androidgamesdk.GameActivity;

import java.util.concurrent.atomic.AtomicBoolean;

import static android.content.ClipDescription.MIMETYPE_TEXT_HTML;
import static android.content.ClipDescription.MIMETYPE_TEXT_PLAIN;

public class MainActivity extends GameActivity {

    public static String FINISH_ACTIVITY_ACTION = "MainActivity.finish";

    static {
        System.loadLibrary("grim");
    }

    private final BroadcastReceiver mBroadcastReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context ctx, Intent i) {
            if (i.getAction().equals(FINISH_ACTIVITY_ACTION)) {
                unregisterReceiver(this);
                onExit();
            }
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Setup HOME environment variable for native code configurations.
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
        super.onCreate(null);

        // Register receiver to finish activity from the BackgroundService.
        registerReceiver(mBroadcastReceiver, new IntentFilter(FINISH_ACTIVITY_ACTION));

        // Start notification service.
        BackgroundService.start(this);

        // Listener for display insets (cutouts) to pass values into native code.
        View content = getWindow().getDecorView().findViewById(android.R.id.content);
        ViewCompat.setOnApplyWindowInsetsListener(content, (v, insets) -> {
            // Setup cutouts values.
            DisplayCutoutCompat dc = insets.getDisplayCutout();
            int cutoutTop = 0;
            int cutoutRight = 0;
            int cutoutBottom = 0;
            int cutoutLeft = 0;
            if (dc != null) {
                cutoutTop = dc.getSafeInsetTop();
                cutoutRight = dc.getSafeInsetRight();
                cutoutBottom = dc.getSafeInsetBottom();
                cutoutLeft = dc.getSafeInsetLeft();
            }

            // Get display insets.
            Insets systemBars = insets.getInsets(WindowInsetsCompat.Type.systemBars());

            // Setup values to pass into native code.
            int[] cutouts = new int[]{0, 0, 0, 0};
            cutouts[0] = Utils.pxToDp(Integer.max(cutoutTop, systemBars.top), this);
            cutouts[1] = Utils.pxToDp(Integer.max(cutoutRight, systemBars.right), this);
            cutouts[2] = Utils.pxToDp(Integer.max(cutoutBottom, systemBars.bottom), this);
            cutouts[3] = Utils.pxToDp(Integer.max(cutoutLeft, systemBars.left), this);
            onDisplayCutouts(cutouts);

            return insets;
        });
    }

    // Implemented into native code to handle display cutouts change.
    native void onDisplayCutouts(int[] cutouts);

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK)   {
            onBack();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    // Implemented into native code to handle key code BACK event.
    public native void onBack();

    private boolean mManualExit;
    private final AtomicBoolean mActivityDestroyed = new AtomicBoolean(false);

    @Override
    protected void onDestroy() {
        if (!mManualExit) {
            unregisterReceiver(mBroadcastReceiver);
            onTermination();
        }

        // Temp fix: kill process after 3 seconds to prevent app hanging at next launch.
        new Thread(() -> {
            try {
                Thread.sleep(3000);
                if (!mActivityDestroyed.get()) {
                    Process.killProcess(Process.myPid());
                }
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        }).start();
        super.onDestroy();
        mActivityDestroyed.set(true);
    }

    // Called from native code.
    public void onExit() {
        // Return if exit was already requested.
        if (mManualExit) {
            return;
        }
        mManualExit = true;
        BackgroundService.stop(this);
        finish();
    }

    // Notify native code to stop activity (e.g. node) on app destroy.
    public native void onTermination();

    // Called from native code to set text into clipboard.
    public void copyText(String data) {
        ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        ClipData clip = ClipData.newPlainText(data, data);
        clipboard.setPrimaryClip(clip);
    }

    // Called from native code to get text from clipboard.
    public String pasteText() {
        ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        String text;
        if (!(clipboard.hasPrimaryClip())) {
            text = "";
        } else if (!(clipboard.getPrimaryClipDescription().hasMimeType(MIMETYPE_TEXT_PLAIN))
                && !(clipboard.getPrimaryClipDescription().hasMimeType(MIMETYPE_TEXT_HTML))) {
            text = "";
        } else {
            ClipData.Item item = clipboard.getPrimaryClip().getItemAt(0);
            text = item.getText().toString();
        }
        return text;
    }
}