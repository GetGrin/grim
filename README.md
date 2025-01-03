> [!IMPORTANT]
> Project moved to [https://gri.mw](https://gri.mw/code/GUI/grim) (personal Forgejo instance)

# <img height="22" src="https://github.com/ardocrat/grim/blob/master/android/app/src/main/ic_launcher-playstore.png?raw=true"> Grim <img height="20" src="https://github.com/mimblewimble/site/blob/master/assets/images/grin-logo.png?raw=true"> <img height="20" src="https://github.com/ardocrat/grim/blob/master/img/logo.png?raw=true">
Cross-platform GUI for [GRiN ツ](https://grin.mw) in [Rust](https://www.rust-lang.org/) 
for maximum compatibility with original [Mimblewimble](https://github.com/mimblewimble/grin) implementation.
Initially supported platforms are Linux, Mac, Windows, limited Android and possible web support with help of [egui](https://github.com/emilk/egui) - immediate mode GUI library in pure Rust.

Named by the character [Grim](http://harrypotter.wikia.com/wiki/Grim) - the shape of a large, black, menacing, spectral giant dog.

![image](https://github.com/user-attachments/assets/a925b1c8-02c9-4b08-b888-0315d11138b6)


## Build instructions
### Install Rust

Follow instructions on [Windows](https://forge.rust-lang.org/infra/other-installation-methods.html).

`curl https://sh.rustup.rs -sSf | sh`

### Desktop

To build and run application go to project directory and run:

```
cargo build --release
./target/release/grim
```

### Android
#### Set up the environment

Install Android SDK / NDK / Platform Tools for your OS according to this [FAQ](https://github.com/codepath/android_guides/wiki/installing-android-sdk-tools).

#### Build the project
Run Android emulator or connect a real device. Command `adb devices` should show at least one device.
In the root of the repo run `./scripts/build_run_android.sh debug|release v7|v8`, where is `v7`, `v8` - device CPU architecture.

## License

Apache License v2.0.
