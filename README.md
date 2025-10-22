# Grim <img height="20" src="https://code.gri.mw/GUI/grim/raw/branch/master/img/grin-logo.png"/> <img height="20" src="https://code.gri.mw/GUI/grim/raw/branch/master/img/logo.png"/>
Cross-platform GUI for [GRiN ツ](https://grin.mw) in [Rust](https://www.rust-lang.org/) 
for maximum compatibility with original [Mimblewimble](https://github.com/mimblewimble/grin) implementation.
Initially supported platforms are Linux, Mac, Windows, limited Android and possible web support with help of [egui](https://github.com/emilk/egui) - immediate mode GUI library in pure Rust.

Named by the character [Grim](http://harrypotter.wikia.com/wiki/Grim) - the shape of a large, black, menacing, spectral giant dog.

![image](https://code.gri.mw/GUI/grim/raw/branch/master/img/cover.png)


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
