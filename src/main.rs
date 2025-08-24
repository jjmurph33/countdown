extern crate sdl2;

use once_cell::sync::Lazy;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use std::time::Duration;

const WINDOW_WIDTH: i32 = 250;
const WINDOW_HEIGHT: i32 = 70;

#[derive(Debug)]
enum ButtonType {
    Play,
    Refresh,
}

struct Button {
    name: ButtonType,
    rect: Rect,                         // position on the screen
    texture_rect: Rect,                 // position in the texture
    click: Box<dyn Fn() + Send + Sync>, // function to call when clicked
}

static BUTTONS: Lazy<Mutex<Vec<Button>>> = Lazy::new(|| Mutex::new(Vec::new()));
static PAUSED: Mutex<bool> = Mutex::new(false);
static TIMER_MAX: Mutex<i32> = Mutex::new(0); // start value of timer
static TIMER: Mutex<i32> = Mutex::new(0); // countdown timer in milliseconds

pub fn main() {
    let args: Vec<String> = env::args().collect();

    // set the countdown timer from the command line
    // default is 15 minutes
    // max is 100 minutes
    let mut timer_minutes = 15;
    if args.len() > 1 {
        timer_minutes = args[1].parse().unwrap_or(15);
    }
    if timer_minutes > 100 {
        timer_minutes = 100;
    }
    *TIMER_MAX.lock().unwrap() = timer_minutes * 60 * 1000;
    *TIMER.lock().unwrap() = *TIMER_MAX.lock().unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let window = video_subsystem
        .window("countdown", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut textures = HashMap::new();
    let texture = texture_creator.load_texture("res/chars.png").unwrap();
    textures.insert("chars", texture);
    let texure = texture_creator.load_texture("res/buttons.png").unwrap();
    textures.insert("buttons", texure);

    *BUTTONS.lock().unwrap() = init_buttons();

    let mut ticks = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Escape) => break 'running,
                    _ => {
                        let keycode = keycode.unwrap_or(Keycode::Space);
                        let keyname = keycode.name();
                        println!("{keyname}");
                    }
                },
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    match mouse_btn {
                        mouse::MouseButton::Left => {
                            check_buttons(x, y);
                            //println!("left {x} {y}");
                        }
                        mouse::MouseButton::Right => {
                            println!("right click {x} {y}");
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        let elapsed_time = (ticks - last_ticks) as i32;

        let paused = *PAUSED.lock().unwrap();
        if !paused {
            update(elapsed_time);
        }

        draw(&mut canvas, &textures);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn update(elapsed_time: i32) {
    let mut timer = *TIMER.lock().unwrap();
    timer -= elapsed_time;
    if timer < 0 {
        timer = 0;
    }
    *TIMER.lock().unwrap() = timer;
}

fn draw(canvas: &mut WindowCanvas, textures: &HashMap<&str, Texture>) {
    // clear the screen
    canvas.set_draw_color(Color::RGB(200, 200, 255));
    canvas.clear();

    // draw the timer
    let offset: i32 = 8;
    let mut x: i32 = offset;
    let y: i32 = offset;
    let timer_str = timer_to_string(*TIMER.lock().unwrap());
    let texture = textures.get("chars").unwrap();
    for c in timer_str.chars() {
        draw_char(canvas, texture, c, x, y);
        x += 32;
    }

    // draw the buttons
    draw_buttons(canvas, &textures.get("buttons").unwrap());

    canvas.present();
}

fn timer_to_string(timer: i32) -> String {
    let secs = timer / 1000;
    let mins = secs / 60;
    let secs = secs % 60;

    let mut mins = mins.to_string();
    if mins.len() == 1 {
        mins.insert(0, '0');
    }
    let mut secs = secs.to_string();
    if secs.len() == 1 {
        secs.insert(0, '0');
    }
    let mut timer_str = String::new();
    timer_str.push_str(&mins);
    timer_str.push_str(":");
    timer_str.push_str(&secs);
    timer_str
}

fn init_buttons() -> Vec<Button> {
    let mut v = Vec::new();
    let offset = 8;
    let mut x = WINDOW_WIDTH - 24 - offset;
    let y = WINDOW_HEIGHT - 24 - offset;
    v.push(Button {
        name: ButtonType::Refresh,
        rect: Rect::new(x, y, 24, 24),
        texture_rect: Rect::new(64 * 2, 0, 64, 64),
        click: Box::new(on_refresh_clicked),
    });
    x = x - 24 - offset;
    v.push(Button {
        name: ButtonType::Play,
        rect: Rect::new(x, y, 24, 24),
        texture_rect: Rect::new(0, 0, 64, 64),
        click: Box::new(on_play_clicked),
    });
    v
}

fn check_buttons(x: i32, y: i32) {
    let p = Point::new(x, y);
    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        if b.rect.contains_point(p) {
            (b.click)();
        }
    }
}

fn on_play_clicked() {
    let paused = *PAUSED.lock().unwrap();
    if paused {
        *PAUSED.lock().unwrap() = false;
    } else {
        *PAUSED.lock().unwrap() = true;
    }
}

fn on_refresh_clicked() {
    *TIMER.lock().unwrap() = *TIMER_MAX.lock().unwrap();
}

fn draw_buttons(canvas: &mut WindowCanvas, button_texture: &Texture) {
    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        let mut src_rect = b.texture_rect;
        match b.name {
            ButtonType::Play => {
                let paused = *PAUSED.lock().unwrap();
                if !paused {
                    src_rect.x = 64 // show the pause image
                }
            }
            _ => {}
        }
        canvas.copy(&button_texture, src_rect, b.rect).unwrap();
    }
}

fn draw_char(canvas: &mut WindowCanvas, char_texture: &Texture, c: char, x: i32, y: i32) {
    let src_rect = char_rect(c);
    let dst_rect = Rect::new(x, y, 32, 32);
    canvas.copy(&char_texture, src_rect, dst_rect).unwrap();
}

fn char_rect(c: char) -> Rect {
    let ord = (c as i32) - 32;
    let x = (ord % 10) * 64;
    let y = (ord / 10) * 64;
    Rect::new(x, y, 64, 64)
}
