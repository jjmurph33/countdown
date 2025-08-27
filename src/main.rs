extern crate sdl2;

use once_cell::sync::Lazy;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::video::Window;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use std::time::Duration;

const WINDOW_WIDTH: i32 = 285;
const WINDOW_HEIGHT: i32 = 50;

#[derive(PartialEq, Clone, Copy)]
enum State {
    Running,
    Paused,
    Done,
}

#[derive(Copy, Clone)]
struct Timer {
    state: State,
    current: i32, // countdown time in milliseconds
    max: i32,     // start value of timer
}

#[derive(Debug)]
enum ButtonType {
    Play,
    Refresh,
    Hide,
}

struct Button {
    name: ButtonType,
    rect: Rect,         // position on the screen
    texture_rect: Rect, // position in the texture
}

static TIMER: Mutex<Timer> = Mutex::new(Timer {
    state: State::Running,
    current: 0,
    max: 0,
});

static BUTTONS: Lazy<Mutex<Vec<Button>>> = Lazy::new(|| Mutex::new(Vec::new()));
static BORDERED: Mutex<bool> = Mutex::new(true); // window borders

pub fn main() {
    let args: Vec<String> = env::args().collect();

    // get the countdown time from the command line
    // default is 15 minutes
    // max is 100 minutes
    let mut timer_minutes = 15;
    if args.len() > 1 {
        timer_minutes = args[1].parse().unwrap_or(15);
    }
    if timer_minutes > 100 {
        timer_minutes = 100;
    }

    {
        // set the timer
        let mut timer = *TIMER.lock().unwrap();
        timer.max = timer_minutes * 60 * 1000;
        timer.current = timer.max;
        *TIMER.lock().unwrap() = timer;
    }
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
    let texture = texture_creator.load_texture("res/buttons.png").unwrap();
    textures.insert("buttons", texture);

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
                        //let keycode = keycode.unwrap_or(Keycode::Space);
                        //let keyname = keycode.name();
                        //println!("{keyname}");
                    }
                },
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    match mouse_btn {
                        mouse::MouseButton::Left => {
                            check_buttons(x, y, canvas.window_mut());
                            //println!("left {x} {y}");
                        }
                        mouse::MouseButton::Right => {
                            //println!("right click {x} {y}");
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        let elapsed_time = ticks - last_ticks;

        let state = (*TIMER.lock().unwrap()).state;
        if state == State::Running {
            update(elapsed_time);
        }

        draw(&mut canvas, &textures);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn update(elapsed_time: u64) {
    let mut timer = *TIMER.lock().unwrap();
    timer.current -= elapsed_time as i32;
    if timer.current <= 0 {
        timer.current = 0;
        timer.state = State::Done;
        println!("Done!");
    }
    *TIMER.lock().unwrap() = timer;
}

fn draw(canvas: &mut WindowCanvas, textures: &HashMap<&str, Texture>) {
    let timer = *TIMER.lock().unwrap();

    // clear the screen
    let bg_color = if timer.state == State::Done {
        Color::RGB(255, 100, 100) // Red when done
    } else {
        Color::RGB(200, 200, 255)
    };
    canvas.set_draw_color(bg_color);
    canvas.clear();

    // draw the timer
    let offset = 8;
    let mut x = offset;
    let y = offset;
    let timer_str = timer_to_string(timer.current);
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
    let offset = 0;
    let mut x = WINDOW_WIDTH - 24 - offset;
    let y = WINDOW_HEIGHT - 24 - offset;
    v.push(Button {
        name: ButtonType::Hide,
        rect: Rect::new(x, y, 24, 24),
        texture_rect: Rect::new(64 * 3, 0, 64, 64),
    });
    x = x - 24 - offset;
    v.push(Button {
        name: ButtonType::Refresh,
        rect: Rect::new(x, y, 24, 24),
        texture_rect: Rect::new(64 * 2, 0, 64, 64),
    });
    x = x - 24 - offset;
    v.push(Button {
        name: ButtonType::Play,
        rect: Rect::new(x, y, 24, 24),
        texture_rect: Rect::new(0, 0, 64, 64),
    });
    v
}

fn check_buttons(x: i32, y: i32, window: &mut Window) {
    let p = Point::new(x, y);
    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        if b.rect.contains_point(p) {
            match b.name {
                ButtonType::Hide => on_hide_clicked(window),
                ButtonType::Refresh => on_refresh_clicked(),
                ButtonType::Play => on_play_clicked(),
            }
        }
    }
}

fn on_play_clicked() {
    let mut timer = *TIMER.lock().unwrap();
    match timer.state {
        State::Paused => {
            timer.state = State::Running;
            *TIMER.lock().unwrap() = timer;
        }
        State::Running => {
            timer.state = State::Paused;
            *TIMER.lock().unwrap() = timer;
        }
        _ => {}
    }
}

fn on_refresh_clicked() {
    let mut timer = *TIMER.lock().unwrap();
    timer.current = timer.max;
    if timer.state == State::Done {
        timer.state = State::Paused;
    }
    *TIMER.lock().unwrap() = timer;
}

fn on_hide_clicked(window: &mut Window) {
    let bordered = !(*BORDERED.lock().unwrap());
    window.set_bordered(bordered);
    *BORDERED.lock().unwrap() = bordered;
}

fn draw_buttons(canvas: &mut WindowCanvas, button_texture: &Texture) {
    let timer = *TIMER.lock().unwrap();
    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        let mut src_rect = b.texture_rect;
        match b.name {
            ButtonType::Play => {
                if timer.state == State::Running {
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
