# Countdown

A desktop countdown timer built with Rust and SDL.

## Requirements

- SDL2 development libraries, including SDL2_image and SDL2_ttf

On Debian or Ubuntu:

```sh
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
```

## Build

```sh
cargo build --release
```

The build script copies the `res` folder into the target output directory.

## Run

```sh
cargo run -- 15
```

The argument is the starting time in minutes between 1 and 99. Defaults to 15 minutes.

Other options:

```sh
cargo run -- -x 100 -y 100 25
cargo run -- -b 10
```

- `-x <pixels>`: window X positio
- `-y <pixels>`: window Y position
- `-b`: hide window borders
