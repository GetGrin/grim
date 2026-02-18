package mw.gri.android;

import android.Manifest;
import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.*;
import android.content.pm.PackageManager;
import android.content.res.Configuration;
import android.net.Uri;
import android.os.*;
import android.os.Process;
import android.provider.Settings;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;
import android.view.KeyEvent;
import android.view.View;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.annotation.NonNull;
import androidx.camera.core.*;
import androidx.camera.lifecycle.ProcessCameraProvider;
import androidx.core.content.ContextCompat;
import androidx.core.content.FileProvider;
import androidx.core.graphics.Insets;
import androidx.core.view.DisplayCutoutCompat;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;
import com.google.androidgamesdk.GameActivity;
import com.google.androidgamesdk.gametextinput.State;
import com.google.common.util.concurrent.ListenableFuture;

import java.io.*;
import java.util.Objects;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import static android.content.ClipDescription.MIMETYPE_TEXT_HTML;
import static android.content.ClipDescription.MIMETYPE_TEXT_PLAIN;

public class MainActivity extends GameActivity {
    public static String STOP_APP_ACTION = "STOP_APP";

    private static final int NOTIFICATIONS_PERMISSION_CODE = 1;
    private static final int CAMERA_PERMISSION_CODE = 2;

    static {
        System.loadLibrary("grim");
    }

    private final BroadcastReceiver mBroadcastReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context ctx, Intent i) {
            if (Objects.equals(i.getAction(), STOP_APP_ACTION)) {
                exit();
            }
        }
    };

    private final ImageAnalysis mImageAnalysis = new ImageAnalysis.Builder()
            .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
            .build();

    private ListenableFuture<ProcessCameraProvider> mCameraProviderFuture = null;
    private ProcessCameraProvider mCameraProvider = null;
    private ExecutorService mCameraExecutor = null;
    private boolean mUseBackCamera = true;

    private ActivityResultLauncher<Intent> mFilePickResult = null;
    private ActivityResultLauncher<Intent> mOpenFilePermissionsResult = null;

    @SuppressLint("UnspecifiedRegisterReceiverFlag")
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        // Check if activity was launched to exclude from recent apps on exit.
        if ((getIntent().getFlags() & Intent.FLAG_ACTIVITY_EXCLUDE_FROM_RECENTS) != 0) {
            super.onCreate(null);
            finish();
            return;
        }

        // Clear cache on start.
        if (savedInstanceState == null && getExternalCacheDir() != null) {
            Utils.deleteDirectoryContent(new File(getExternalCacheDir().getPath()), false);
        }

        // Setup environment variables for native code.
        try {
            Os.setenv("HOME", Objects.requireNonNull(getExternalFilesDir("")).getPath(), true);
            Os.setenv("XDG_CACHE_HOME", Objects.requireNonNull(getExternalCacheDir()).getPath(), true);
            Os.setenv("ARTI_FS_DISABLE_PERMISSION_CHECKS", "true", true);
            Os.setenv("NATIVE_LIBS_DIR", getApplicationInfo().nativeLibraryDir, true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }

        super.onCreate(null);

        // Register receiver to finish activity from the BackgroundService.
        ContextCompat.registerReceiver(this, mBroadcastReceiver, new IntentFilter(STOP_APP_ACTION), ContextCompat.RECEIVER_NOT_EXPORTED);

        // Register associated file opening result.
        mOpenFilePermissionsResult = registerForActivityResult(
                new ActivityResultContracts.StartActivityForResult(),
                result -> {
                    if (Build.VERSION.SDK_INT >= 30) {
                        if (Environment.isExternalStorageManager()) {
                            onFile();
                        }
                    } else if (result.getResultCode() == RESULT_OK) {
                        onFile();
                    }
                }
        );
        // Register file pick result.
        mFilePickResult = registerForActivityResult(
                new ActivityResultContracts.StartActivityForResult(),
                result -> {
                    int resultCode = result.getResultCode();
                    Intent data = result.getData();
                    if (resultCode == Activity.RESULT_OK) {
                        String path = "";
                        if (data != null && data.getData() != null) {
                            Uri uri = data.getData();
                            String name = "pick" + Utils.getFileExtension(uri, this);
                            File file = new File(getExternalCacheDir(), name);
                            try (InputStream is = getContentResolver().openInputStream(uri);
                                 OutputStream os = new FileOutputStream(file)) {
                                byte[] buffer = new byte[1024];
                                int length;
                                while (true) {
                                    assert is != null;
                                    if (!((length = is.read(buffer)) > 0)) break;
                                    os.write(buffer, 0, length);
                                }
                            } catch (Exception e) {
                                Log.e("grim", e.toString());
                            }
                            path = file.getPath();
                        }
                        onFilePick(path);
                    } else {
                        onFilePick("");
                    }
                });

        // Listener for display insets (cutouts) to pass values into native code.
        View content = getWindow().getDecorView().findViewById(android.R.id.content);
        ViewCompat.setOnApplyWindowInsetsListener(content, (v, insets) -> {
            // Get display cutouts.
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

            // Pass values into native code.
            int[] values = new int[]{0, 0, 0, 0};
            values[0] = Utils.pxToDp(Integer.max(cutoutTop, systemBars.top), this);
            values[1] = Utils.pxToDp(Integer.max(cutoutRight, systemBars.right), this);
            values[2] = Utils.pxToDp(Integer.max(cutoutBottom, systemBars.bottom), this);
            values[3] = Utils.pxToDp(Integer.max(cutoutLeft, systemBars.left), this);
            onDisplayInsets(values);

            return insets;
        });

        findViewById(android.R.id.content).post(() -> {
            // Request notifications permissions if needed.
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

        // Check if intent has data on launch.
        if (savedInstanceState == null) {
            onNewIntent(getIntent());
        }
    }

    // Pass display insets into native code.
    public native void onDisplayInsets(int[] cutouts);

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        String action = intent.getAction();
        // Check if file was open with the application.
        if (action != null && action.equals(Intent.ACTION_VIEW)) {
            Intent i = getIntent();
            i.setData(intent.getData());
            setIntent(i);
            onFile();
        }
    }

    // Callback when associated file was open.
    private void onFile() {
        Uri data = getIntent().getData();
        if (data == null) {
            return;
        }
        if (Build.VERSION.SDK_INT >= 30) {
            if (!Environment.isExternalStorageManager()) {
                Intent i = new Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION);
                mOpenFilePermissionsResult.launch(i);
                return;
            }
        }
        try {
            ParcelFileDescriptor parcelFile = getContentResolver().openFileDescriptor(data, "r");
            assert parcelFile != null;
            FileReader fileReader = new FileReader(parcelFile.getFileDescriptor());
            BufferedReader reader = new BufferedReader(fileReader);
            String line;
            StringBuilder buff = new StringBuilder();
            while ((line = reader.readLine()) != null) {
                buff.append(line);
            }
            reader.close();
            fileReader.close();

            // Provide file content into native code.
            onData(buff.toString());
        } catch (Exception e) {
            Log.e("grim", e.toString());
        }
    }

    // Pass data into native code.
    public native void onData(String data);

    @Override
    public void onConfigurationChanged(Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        // Called on screen orientation change to restart camera.
        if (mCameraProvider != null) {
            stopCamera();
            startCamera();
        }
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions, @NonNull int[] results) {
        super.onRequestPermissionsResult(requestCode, permissions, results);
        if (results.length != 0 && results[0] == PackageManager.PERMISSION_GRANTED) {
            switch (requestCode) {
                case NOTIFICATIONS_PERMISSION_CODE: {
                    BackgroundService.start(this);
                    return;
                }
                case CAMERA_PERMISSION_CODE: {
                    startCamera();
                }
            }
        }
    }

    @Override
    protected void onTextInputEventNative(long l, State state) {
        super.onTextInputEventNative(l, state);
        if (state.selectionEnd > state.composingRegionStart && state.composingRegionStart >= 0) {
            String input = String.valueOf(state.text.charAt(state.composingRegionStart));
            if (input.contains("\n")) {
                onEnterInput();
            } else {
                onTextInput(input);
            }
        }
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.getAction() == KeyEvent.ACTION_DOWN) {
            if (event.getKeyCode() == KeyEvent.KEYCODE_BACK) {
                onBack();
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_DEL) {
                onClearInput();
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_ENTER) {
                onEnterInput();
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_0) {
                onTextInput("0");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_1) {
                onTextInput("1");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_2) {
                onTextInput("2");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_3) {
                onTextInput("3");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_4) {
                onTextInput("4");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_5) {
                onTextInput("5");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_6) {
                onTextInput("6");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_7) {
                onTextInput("7");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_8) {
                onTextInput("8");
                return false;
            } else if (event.getKeyCode() == KeyEvent.KEYCODE_9) {
                onTextInput("9");
                return false;
            }
        }
        return super.dispatchKeyEvent(event);
    }

    // Pass back navigation event into native code.
    public native void onBack();

    // Pass clear key event into native code.
    public native void onClearInput();

    // Pass enter key event into native code.
    public native void onEnterInput();

    // Pass last entered character from soft keyboard into native code.
    public native void onTextInput(String character);

    // Called from native code to exit app.
    public void exit() {
        finishAndRemoveTask();
    }

    @Override
    protected void onDestroy() {
        unregisterReceiver(mBroadcastReceiver);
        BackgroundService.stop(this);

        // Kill process after 3 secs if app was terminated from recent apps to prevent app hang.
        new Thread(() -> {
            try {
                onTermination();
                Thread.sleep(3000);
                Process.killProcess(Process.myPid());
            } catch (InterruptedException e) {
                throw new RuntimeException(e);
            }
        }).start();

        super.onDestroy();
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
        ClipDescription desc = clipboard.getPrimaryClipDescription();
        ClipData data = clipboard.getPrimaryClip();
        String text = "";
        if (!(clipboard.hasPrimaryClip())) {
            text = "";
        } else if (desc != null && (!(desc.hasMimeType(MIMETYPE_TEXT_PLAIN))
                && !(desc.hasMimeType(MIMETYPE_TEXT_HTML)))) {
            text = "";
        } else if (data != null) {
            ClipData.Item item = data.getItemAt(0);
            text = item.getText().toString();
        }
        return text;
    }

    // Called from native code to start camera.
    public void startCamera() {
        String notificationsPermission = Manifest.permission.CAMERA;
        if (checkSelfPermission(notificationsPermission) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[] { notificationsPermission }, CAMERA_PERMISSION_CODE);
        } else {
            if (mCameraProviderFuture == null) {
                mCameraProviderFuture = ProcessCameraProvider.getInstance(this);
                mCameraProviderFuture.addListener(() -> {
                    try {
                        mCameraProvider = mCameraProviderFuture.get();
                        // Start camera.
                        openCamera();
                    } catch (Exception e) {
                        View content = findViewById(android.R.id.content);
                        if (content != null) {
                            content.post(this::stopCamera);
                        }
                    }
                }, ContextCompat.getMainExecutor(this));
            } else {
                View content = findViewById(android.R.id.content);
                if (content != null) {
                    content.post(this::openCamera);
                }
            }
        }
    }

    // Open camera after initialization or start after stop.
    private void openCamera() {
        // Set up the image analysis use case which will process frames in real time.
        if (mCameraExecutor == null) {
            mCameraExecutor = Executors.newSingleThreadExecutor();
            mImageAnalysis.setAnalyzer(mCameraExecutor, image -> {
                // Convert image to JPEG.
                byte[] data = Utils.convertCameraImage(image);
                // Send image to native code.
                onCameraImage(data, image.getImageInfo().getRotationDegrees());
                image.close();
            });
        }

        // Select back camera initially.
        CameraSelector cameraSelector = CameraSelector.DEFAULT_BACK_CAMERA;
        if (!mUseBackCamera) {
            cameraSelector = CameraSelector.DEFAULT_FRONT_CAMERA;
        }
        // Apply declared configs to CameraX using the same lifecycle owner
        mCameraProvider.unbindAll();
        mCameraProvider.bindToLifecycle(this, cameraSelector, mImageAnalysis);
    }

    // Called from native code to stop camera.
    public void stopCamera() {
        View content = findViewById(android.R.id.content);
        if (content != null) {
            content.post(() -> {
                if (mCameraProvider != null) {
                    mCameraProvider.unbindAll();
                }
            });
        }
    }

    // Called from native code to get number of cameras.
    public int camerasAmount() {
        if (mCameraProvider == null) {
            return 0;
        }
        return mCameraProvider.getAvailableCameraInfos().size();
    }

    // Called from native code to switch camera.
    public void switchCamera() {
        mUseBackCamera = !mUseBackCamera;
        stopCamera();
        startCamera();
    }

    // Pass image from camera into native code.
    public native void onCameraImage(byte[] buff, int rotation);

    // Called from native code to share data from provided path.
    public void shareData(String path) {
        File file = new File(path);
        Uri uri = FileProvider.getUriForFile(this, "mw.gri.android.fileprovider", file);
        Intent intent = new Intent(Intent.ACTION_SEND);
        intent.putExtra(Intent.EXTRA_STREAM, uri);
        intent.setType("text/*");
        startActivity(Intent.createChooser(intent, "Share data"));
    }

    // Called from native code to check if device is using dark theme.
    public boolean useDarkTheme() {
        int currentNightMode = getResources().getConfiguration().uiMode & Configuration.UI_MODE_NIGHT_MASK;
        return  currentNightMode == Configuration.UI_MODE_NIGHT_YES;
    }

    // Called from native code to pick the file.
    public void pickFile() {
        Intent intent = new Intent(Intent.ACTION_GET_CONTENT);
        intent.setType("text/*");
        try {
            mFilePickResult.launch(Intent.createChooser(intent, "Pick file"));
        } catch (android.content.ActivityNotFoundException ex) {
            onFilePick("");
        }
    }

    // Pass picked file into native code.
    public native void onFilePick(String path);
}