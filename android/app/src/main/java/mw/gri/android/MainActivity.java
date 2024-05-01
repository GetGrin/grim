package mw.gri.android;

import android.Manifest;
import android.content.*;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Process;
import android.system.ErrnoException;
import android.system.Os;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.InputMethodManager;
import androidx.annotation.NonNull;
import androidx.core.graphics.Insets;
import androidx.core.view.DisplayCutoutCompat;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import com.google.androidgamesdk.GameActivity;
import org.jetbrains.annotations.NotNull;

import static android.content.ClipDescription.MIMETYPE_TEXT_HTML;
import static android.content.ClipDescription.MIMETYPE_TEXT_PLAIN;

public class MainActivity extends GameActivity {
    public static String STOP_APP_ACTION = "STOP_APP";

    private static final int NOTIFICATIONS_PERMISSION_CODE = 1;

    static {
        System.loadLibrary("grim");
    }

    private final BroadcastReceiver mBroadcastReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context ctx, Intent i) {
            if (i.getAction().equals(STOP_APP_ACTION)) {
                onExit();
                Process.killProcess(Process.myPid());
            }
        }
    };

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Setup HOME environment variable for native code configurations.
        try {
            Os.setenv("HOME", getExternalFilesDir("").getPath(), true);
            Os.setenv("XDG_CACHE_HOME", getExternalCacheDir().getPath(), true);
            Os.setenv("ARTI_FS_DISABLE_PERMISSION_CHECKS", "true", true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }
        super.onCreate(null);

        // Register receiver to finish activity from the BackgroundService.
        registerReceiver(mBroadcastReceiver, new IntentFilter(STOP_APP_ACTION));

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
            int[] values = new int[]{0, 0, 0, 0};
            values[0] = Utils.pxToDp(Integer.max(cutoutTop, systemBars.top), this);
            values[1] = Utils.pxToDp(Integer.max(cutoutRight, systemBars.right), this);
            values[2] = Utils.pxToDp(Integer.max(cutoutBottom, systemBars.bottom), this);
            values[3] = Utils.pxToDp(Integer.max(cutoutLeft, systemBars.left), this);
            onDisplayInsets(values);

            return insets;
        });

        findViewById(android.R.id.content).post(() -> {
            // Request notifications permissions.
            if (Build.VERSION.SDK_INT >= 33) {
                String notificationsPermission = Manifest.permission.POST_NOTIFICATIONS;
                if (checkSelfPermission(notificationsPermission) != PackageManager.PERMISSION_GRANTED) {
                    requestPermissions(new String[] { notificationsPermission }, NOTIFICATIONS_PERMISSION_CODE);
                } else {
                    // Start notification service.
                    BackgroundService.start(this);
                }
            } else {
                // Start notification service.
                BackgroundService.start(this);
            }
        });
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull @NotNull String[] permissions, @NonNull @NotNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == NOTIFICATIONS_PERMISSION_CODE && grantResults.length != 0 &&
                grantResults[0] == PackageManager.PERMISSION_GRANTED) {
            // Start notification service.
            BackgroundService.start(this);
        }
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        // To support non-english input.
        if (event.getAction() == KeyEvent.ACTION_MULTIPLE && event.getKeyCode() == KeyEvent.KEYCODE_UNKNOWN) {
            if (!event.getCharacters().isEmpty()) {
                onInput(event.getCharacters());
                return false;
            }
        // Pass any other input values into native code.
        } else if (event.getAction() == KeyEvent.ACTION_UP &&
                event.getKeyCode() != KeyEvent.KEYCODE_ENTER &&
                event.getKeyCode() != KeyEvent.KEYCODE_BACK) {
            onInput(String.valueOf((char)event.getUnicodeChar()));
            return false;
        }
        return super.dispatchKeyEvent(event);
    }

    // Provide last entered character from soft keyboard into native code.
    public native void onInput(String character);

    // Implemented into native code to handle display insets change.
    native void onDisplayInsets(int[] cutouts);

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_BACK) {
            onBack();
            return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    // Implemented into native code to handle key code BACK event.
    public native void onBack();

    // Actions on app exit.
    private void onExit() {
        unregisterReceiver(mBroadcastReceiver);
        BackgroundService.stop(this);
    }

    @Override
    protected void onDestroy() {
        onExit();

        // Kill process after 3 seconds if app was terminated from recent apps to prevent app hanging.
        new Thread(() -> {
            try {
                onTermination();
                Thread.sleep(3000);
                Process.killProcess(Process.myPid());
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        }).start();

        // Destroy an app and kill process.
        super.onDestroy();
        Process.killProcess(Process.myPid());
    }

    // Notify native code to stop activity (e.g. node) if app was terminated from recent apps.
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

    public void showKeyboard() {
        InputMethodManager imm = (InputMethodManager )getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.showSoftInput(getWindow().getDecorView(), InputMethodManager.SHOW_IMPLICIT);
    }

    public void hideKeyboard() {
        InputMethodManager imm = (InputMethodManager )getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.hideSoftInputFromWindow(getWindow().getDecorView().getWindowToken(), 0);
    }
}