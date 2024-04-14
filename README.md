# <img height="20" src="https://github.com/ardocrat/grim/blob/master/app/src/main/ic_launcher-playstore.png?raw=true"> Grim <img height="20" src="https://github.com/mimblewimble/site/blob/master/assets/images/grin-logo.png?raw=true"> <img height="20" src="https://github.com/ardocrat/grim/blob/master/img/logo.png?raw=true">
Cross-platform GUI for [Grin](https://github.com/mimblewimble/grin) on [Rust](https://stackoverflow.blog/2020/01/20/what-is-rust-and-why-is-it-so-popular/) with focus on usability and availability to be used by anyone, anywhere.

Named by the character [Grim](http://harrypotter.wikia.com/wiki/Grim) - the shape of a large, black, menacing, spectral giant dog.

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

Install Android SDK / NDK / Platform Tools for your OS according to this [FAQ](https://github.com/codepath/android_guides/wiki/installing-android-sdk-tools).

#### Build the project
Run Android emulator or connect a real device. Command `adb devices` should show at least one device.
In the root of the repo run `./build_run_android.sh release arch`, where is `arch` is `v7` or `v8` based on device CPU architecture.

## License

Apache License v2.0.