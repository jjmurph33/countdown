# Countdown

A desktop countdown timer built with Rust and SDL.
<br><br>

<div align="center">

![countdown](data/images/countdown.png) &nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 
![countdown-done](data/images/countdown-done.png) &nbsp;&nbsp;&nbsp;&nbsp;&nbsp; 
![countdown-desktop](data/images/countdown-desktop.png)

</div>

## Requirements

SDL2 development libraries, including SDL2_image and SDL2_ttf

### Linux (Debian/Ubuntu)

```sh
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev
```

### Linux (Fedora)

```sh
sudo dnf install SDL2-devel SDL2_image-devel SDL2_ttf-devel
```

### macOS

```sh
brew install sdl2 sdl2_image sdl2_ttf
```

### Windows

1. Download the MSVC development libraries from [libsdl.org](http://www.libsdl.org/):  
    `SDL2-devel-2.x.x-VC.zip`  
    `SDL2_image-devel-2.x.x-VC.zip`  
    `SDL2_ttf-devel-2.x.x-VC.zip`  
2. Copy the lib and DLL files to the project directory

<br>

## Build

```sh
cargo build --release
```

<br>

## Run

```sh
target/release/Countdown 10
```

The argument is the starting time in minutes between 1 and 99. Defaults to <b>15</b> minutes.

Other options:

```sh
target/release/Countdown -x 100 -y 100 -b 5
```

- `-x <pixels>`: window X position
- `-y <pixels>`: window Y position
- `-b`: hide window borders
- `5`: 5 minutes
