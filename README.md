This is a minimal test application based on `GameActivity` that just
runs a mainloop based on android_activity::poll_events() and traces
the events received without doing any rendering. It also saves and
restores some minimal application state.

```
export ANDROID_NDK_HOME="path/to/ndk"
export ANDROID_HOME="path/to/sdk"

rustup target add aarch64-linux-android
cargo install cargo-ndk

export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0" && cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ build
./gradlew build
./gradlew installDebug
adb shell am start -n mw.gri.android/.MainActivity
```
