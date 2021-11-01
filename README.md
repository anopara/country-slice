# country-slice

My small weekend project, written in [Bevy Engine](https://github.com/bevyengine/bevy). Still **active WIP**! Making it public early in case someone might find this example of simple proc stuff in Bevy useful 😇

## Build and Run

1. download the repo
2. cd to the `country-slice` directory
3. execute `cargo run --release`, this will run the app rightaway

OR

3. execute `cargo build --release`
4. move the executable from `target/release` to the `country-slice` directory (so that it's with the `assets` directory)
5. Launch it (`country-slice.exe` on Windows, `./country-slice` on Mac and Linux)

## Compilation problems

You may have compilation problems if some dependencies are missing.

On Debian for example, you may have to do

    sudo apt install libasound2-dev libudev-dev
