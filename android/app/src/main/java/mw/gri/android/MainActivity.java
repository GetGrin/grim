package mw.gri.android;

import android.Manifest;
import android.annotation.SuppressLint;
import android.app.Activity;
import android.app.NativeActivity;
import android.content.*;
import android.content.pm.PackageManager;
import android.content.res.Configuration;
import android.net.Uri;
import android.os.*;
import android.os.Process;
import android.provider.Settings;
import android.system.ErrnoException;
import android.system.Os;
import android.view.View;
import android.view.inputmethod.InputMethodManager;

import androidx.annotation.NonNull;
import androidx.camera.core.*;
import androidx.camera.lifecycle.ProcessCameraProvider;
import androidx.core.content.ContextCompat;
import androidx.core.content.FileProvider;
import androidx.core.graphics.Insets;
import androidx.core.view.DisplayCutoutCompat;
import androidx.core.view.ViewCompat;
import androidx.core.view.WindowInsetsCompat;

import com.google.common.util.concurrent.ListenableFuture;

import java.io.*;
import java.util.Objects;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import static android.content.ClipDescription.MIMETYPE_TEXT_HTML;
import static android.content.ClipDescription.MIMETYPE_TEXT_PLAIN;

public class MainActivity extends NativeActivity {
    private static final int FILE_PICK_REQUEST = 1001;
    private static final int FILE_PERMISSIONS_REQUEST = 1002;

    private static final int NOTIFICATIONS_PERMISSION_CODE = 1;
    private static final int CAMERA_PERMISSION_CODE = 2;

    public static final String STOP_APP_ACTION = "STOP_APP_ACTION";

    static {
        System.loadLibrary("grim");
    }

    private final BroadcastReceiver mReceiver = new BroadcastReceiver() {
        @SuppressLint("RestrictedApi")
        @Override
        public void onReceive(Context context, Intent intent) {
            if (Objects.equals(intent.getAction(), MainActivity.STOP_APP_ACTION)) {
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
        String cacheDir = Objects.requireNonNull(getExternalCacheDir()).getPath();
        if (savedInstanceState == null) {
            Utils.deleteDirectoryContent(new File(cacheDir), false);
        }

        // Setup environment variables for native code.
        try {
            Os.setenv("HOME", Objects.requireNonNull(getExternalFilesDir("")).getPath(), true);
            Os.setenv("XDG_CACHE_HOME", cacheDir, true);
            Os.setenv("ARTI_FS_DISABLE_PERMISSION_CHECKS", "true", true);
        } catch (ErrnoException e) {
            throw new RuntimeException(e);
        }

        super.onCreate(null);

        ContextCompat.registerReceiver(this, mReceiver, new IntentFilter(STOP_APP_ACTION), ContextCompat.RECEIVER_NOT_EXPORTED);

        // Listener for display insets (cutouts) to pass values into native code.
        View content = findViewById(android.R.id.content).getRootView();
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

        content.post(() -> {
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

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        switch (requestCode) {
            case FILE_PICK_REQUEST:
                if (Build.VERSION.SDK_INT >= 30) {
                    if (Environment.isExternalStorageManager()) {
                        onFile();
                    }
                } else if (resultCode == RESULT_OK) {
                    onFile();
                }
            case FILE_PERMISSIONS_REQUEST:
                if (resultCode == Activity.RESULT_OK) {
                    String path = "";
                    if (data != null) {
                        Uri uri = data.getData();
                        String name = "pick" + Utils.getFileExtension(uri, this);
                        File file = new File(getExternalCacheDir(), name);
                        try (InputStream is = getContentResolver().openInputStream(uri);
                             OutputStream os = new FileOutputStream(file)) {
                            byte[] buffer = new byte[1024];
                            int length;
                            while ((length = is.read(buffer)) > 0) {
                                os.write(buffer, 0, length);
                            }
                        } catch (Exception e) {
                            e.printStackTrace();
                        }
                        path = file.getPath();
                    }
                    onFilePick(path);
                } else {
                    onFilePick("");
                }
        }
    }

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
                startActivityForResult(i, FILE_PERMISSIONS_REQUEST);
                return;
            }
        }
        try {
            ParcelFileDescriptor parcelFile = getContentResolver().openFileDescriptor(data, "r");
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
            e.printStackTrace();
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
                    // Start notification service.
                    BackgroundService.start(this);
                    return;
                }
                case CAMERA_PERMISSION_CODE: {
                    // Start camera.
                    startCamera();
                }
            }
        }
    }

    // Implemented into native code to handle display insets change.
    native void onDisplayInsets(int[] cutouts);

    // Called from native code to exit app.
    public void exit() {
        finishAndRemoveTask();
    }

    @Override
    protected void onDestroy() {
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

    // Called from native code to show keyboard.
    public void showKeyboard() {
        InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.showSoftInput(getWindow().getDecorView(), InputMethodManager.SHOW_IMPLICIT);
    }

    // Called from native code to hide keyboard.
    public void hideKeyboard() {
        InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
        imm.hideSoftInputFromWindow(getWindow().getDecorView().getWindowToken(), 0);
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
//        mCameraProvider.bindToLifecycle(this, cameraSelector, mImageAnalysis);
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
        intent.setType("*/*");
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
        intent.setType("*/*");
        try {
            startActivityForResult(Intent.createChooser(intent, "Pick file"), FILE_PICK_REQUEST);
        } catch (ActivityNotFoundException ex) {
            onFilePick("");
        }
    }

    // Pass picked file into native code.
    public native void onFilePick(String path);
}