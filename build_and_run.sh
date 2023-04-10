export CPPFLAGS="-DMDB_USE_ROBUST=0" && export CFLAGS="-DMDB_USE_ROBUST=0" && cargo ndk -t arm64-v8a build
if [ $? -eq 0 ]
then
  yes | cp -f target/aarch64-linux-android/debug/libgrin_android.so app/src/main/jniLibs/arm64-v8a
  ./gradlew clean
  ./gradlew build
  #./gradlew installDebug
  adb install app/build/outputs/apk/debug/app-debug.apk
  adb shell am start -n mw.gri.android/.MainActivity
fi