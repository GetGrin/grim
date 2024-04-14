<img height="80" src="https://github.com/ardocrat/grim/blob/master/img/logo.png?raw=true">

# Grim

Cross-platform GUI for [Grin](https://github.com/mimblewimble/grin) on [Rust](https://stackoverflow.blog/2020/01/20/what-is-rust-and-why-is-it-so-popular/) with focus on usability and availability to be used by anyone, anywhere.

Named by the character [Grim](https://harrypotter.fandom.com/wiki/Grim) - the shape of a large, black, menacing, spectral giant dog.

## Build instructions
### Install Rust
`curl https://sh.rustup.rs -sSf | sh`

### Desktop

To build and run application go to project directory and run:

```
cargo build release
./target/release/grim
```

### Android
#### Set up the environment

Install Android SDK / NDK / Platform Tools
```
brew cask install android-sdk android-ndk android-platform-tools
```

Add to your `.bashprofile` or `.zshrc`:
```
export ANDROID_HOME="$(brew --prefix)/share/android-sdk"
export PATH=$PATH:$ANDROID_HOME/emulator:$ANDROID_HOME/tools:$ANDROID_HOME/tools/bin:$ANDROID_HOME/platform-tools
export ANDROID_NDK_HOME="$(brew --prefix)/share/android-ndk"
```

#### Build the project
Run Android emulator or connect a real device. Command `adb devices` should show at least one device.
In the root of the repo run `./build_run_android.sh release {arch}`, where is `arch` is `v7` or `v8` based on device CPU architecture.

## License

Apache License v2.0.