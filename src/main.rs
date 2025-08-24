extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect,Point};
use sdl2::render::{Texture, WindowCanvas};
use sdl2::image::LoadTexture;
use sdl2::mouse;
use std::time::Duration;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const WINDOW_WIDTH: i32 = 200;
const WINDOW_HEIGHT: i32 = 100;

enum ButtonType {
    Play,
    Pause,
    Refresh,
}

struct Button {
    rect: Rect,
}

static BUTTONS: Lazy<Mutex<Vec<Button>>> = Lazy::new(|| Mutex::new(Vec::new()));



pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let window = video_subsystem.window("countdown", WINDOW_WIDTH as u32,WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let char_texture = texture_creator.load_texture("res/chars.png").unwrap();
    let button_texture = texture_creator.load_texture("res/buttons.png").unwrap();

    let mut ticks = timer_subsystem.ticks64();
    let mut elapsed_ms = 0;
    let mut timer = 500;

    // create the buttons
    {
        let mut buttons = BUTTONS.lock().unwrap();
        *buttons = init_buttons();
    }

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(Keycode::Escape) => {
                            break 'running
                        }
                        _ => {
                            let keycode = keycode.unwrap_or(Keycode::Space);
                            let keyname = keycode.name();
                            println!("{keyname}");
                        }
                    }
                },
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    match mouse_btn {
                        mouse::MouseButton::Left => {
                            check_buttons(x,y);
                            //println!("left {x} {y}");
                        },
                        mouse::MouseButton::Right => {
                            println!("right {x} {y}");
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        // update the timer
        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        elapsed_ms += ticks - last_ticks;
        if elapsed_ms >= 1000 {
            timer -= 1;
            elapsed_ms = 0;
        }
        if timer < 0 {
            timer = 0;
        }

        // clear the screen
        canvas.set_draw_color(Color::RGB(200, 200, 255));
        canvas.clear();

        // draw the timer
        let offset: i32 = 8;
        let mut x: i32 = offset;
        let mut y: i32 = offset;
        let timer_str = timer_to_string(timer);
        for c in timer_str.chars() {
            draw_char(&mut canvas,&char_texture,c,x,y);
            x += 32;
        }

        // draw the buttons
        x = WINDOW_WIDTH - 32 - offset;
        y = WINDOW_HEIGHT - 32 - offset;
        draw_button(&mut canvas,&button_texture,ButtonType::Refresh,x,y);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

fn init_buttons() -> Vec<Button> {
    let mut v = Vec::new();
    // Add the refresh button
    let x = WINDOW_WIDTH - 32 - 8;
    let y = WINDOW_HEIGHT - 32 - 8;
    v.push(Button { rect: Rect::new(x, y, 32, 32) });
    v
}

fn check_buttons(x: i32, y:i32) {
    let p = Point::new(x,y);

    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        if b.rect.contains_point(p) {
            println!("Button clicked at {:?}", p);
        }
    }
}

fn timer_to_string(timer: i32) -> String {
    let mins = timer / 60;
    let secs = timer % 60;

    let mut mins = mins.to_string();
    if mins.len() == 1 {
        mins.insert(0,'0');
    }
    let mut secs = secs.to_string();
    if secs.len() == 1 {
        secs.insert(0,'0');
    }
    let mut timer_str = String::new();
    timer_str.push_str(&mins);
    timer_str.push_str(":");
    timer_str.push_str(&secs);

    timer_str
}

fn draw_button(canvas: &mut WindowCanvas,button_texture: &Texture,button: ButtonType,x: i32,y: i32) {
    let mut src_rect = Rect::new(0,0,64,64);
    match button {
        ButtonType::Refresh => {
            src_rect.set_x(64*2);
        },
        _ => {}
    }
    let dst_rect = Rect::new(x,y,32,32);
    canvas.copy(&button_texture,src_rect,dst_rect).unwrap();    
}

fn draw_char(canvas: &mut WindowCanvas,char_texture: &Texture,c: char,x: i32,y:i32) {
    let src_rect = char_rect(c);
    let dst_rect = Rect::new(x,y,32,32);
    canvas.copy(&char_texture,src_rect,dst_rect).unwrap();
}

fn char_rect(c: char) -> Rect {
    let ord = (c as i32) - 32;
    let x = (ord % 10) * 64;
    let y = (ord / 10) * 64;
    Rect::new(x,y,64,64)
}
