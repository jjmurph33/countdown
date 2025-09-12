extern crate sdl2;

use once_cell::sync::Lazy;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::TextureQuery;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::{EventPump, mouse};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use std::time::Duration;

const WINDOW_WIDTH: i32 = 185;
const WINDOW_HEIGHT: i32 = 70;

const PROMPT_OK: (i32, i32, i32, i32) = (WINDOW_WIDTH / 4, WINDOW_HEIGHT / 2, 17, 14);
const PROMPT_CANCEL: (i32, i32, i32, i32) =
    (WINDOW_WIDTH - WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, 39, 14);

#[derive(PartialEq, Clone, Copy)]
enum PromptType {
    Reset,
    Exit,
}

#[derive(PartialEq, Clone, Copy)]
enum State {
    Running,
    Paused,
    Done,
    Prompt(PromptType),
    Exiting,
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
    Settings,
    Mute,
}

struct Button {
    name: ButtonType,
    rect: Rect,             // position on the screen
    texture_rect: Rect,     // position in the texture
    texture_rect_alt: Rect, // position in the texture of the alternate icon (ex: play/pause)
}

impl Button {
    fn new(
        name: ButtonType,
        screen_x: i32,
        screen_y: i32,
        x_offset: i32,
        y_offset: i32,
        alt_x_offset: i32,
        alt_y_offset: i32,
    ) -> Self {
        let rect = Rect::new(screen_x, screen_y, 24, 24);
        let texture_rect = Rect::new(64 * x_offset, 64 * y_offset, 64, 64);
        let texture_rect_alt = Rect::new(64 * alt_x_offset, 64 * alt_y_offset, 64, 64);
        Button {
            name,
            rect,
            texture_rect,
            texture_rect_alt,
        }
    }
}

struct App {
    state: State,
    current: i32, // countdown time in milliseconds
    max: i32,     // start value of timer
    buttons: [Button; 5],
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

    let ttf_context = sdl2::ttf::init().unwrap();
    let font_path = "/usr/share/fonts/truetype/liberation/LiberationMono-Bold.ttf";
    let font = ttf_context.load_font(font_path, 12).unwrap();

    let window = video_subsystem
        .window("countdown", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    // load the textures
    let mut textures = HashMap::new();
    let texture = texture_creator.load_texture("res/chars.png").unwrap();
    textures.insert("chars", texture);
    let texture = texture_creator.load_texture("res/buttons.png").unwrap();
    textures.insert("buttons", texture);

    // create the buttons
    *BUTTONS.lock().unwrap() = init_buttons();

    let mut ticks = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        let state = (*TIMER.lock().unwrap()).state;

        if state == State::Exiting {
            break;
        }

        handle_events(&mut event_pump, &mut canvas);

        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        let elapsed_time = ticks - last_ticks;

        if state == State::Running {
            update(elapsed_time);
        }

        draw(&mut canvas, &textures, &font);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn handle_events(event_pump: &mut EventPump, canvas: &mut WindowCanvas) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                let mut timer = *TIMER.lock().unwrap();
                timer.state = State::Prompt(PromptType::Exit);
                *TIMER.lock().unwrap() = timer;
            }
            Event::KeyDown { keycode, .. } => match keycode {
                Some(Keycode::Escape) => {
                    let mut timer = *TIMER.lock().unwrap();
                    match timer.state {
                        // if in a prompt, cancel it
                        State::Prompt(_) => timer.state = State::Paused,
                        // else, prompt to exit
                        _ => timer.state = State::Prompt(PromptType::Exit),
                    }
                    *TIMER.lock().unwrap() = timer;
                }
                _ => {}
            },
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                if mouse_btn == mouse::MouseButton::Left {
                    check_buttons(x, y, canvas.window_mut());
                }
            }
            _ => {}
        }
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

fn draw(canvas: &mut WindowCanvas, textures: &HashMap<&str, Texture>, font: &Font) {
    let timer = *TIMER.lock().unwrap();

    // clear the screen
    let bg_color = if timer.state == State::Done {
        Color::RGB(255, 100, 100) // Red when done
    } else {
        Color::RGB(200, 200, 255)
    };
    canvas.set_draw_color(bg_color);
    canvas.clear();

    if matches!(timer.state, State::Prompt(_)) {
        draw_prompt(canvas, font);
    } else {
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

        draw_buttons(canvas, &textures.get("buttons").unwrap());
    }

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
    let mut x = WINDOW_WIDTH - 62 - offset;
    let y = WINDOW_HEIGHT - 24 - offset;

    v.push(Button::new(ButtonType::Settings, x, y, 4, 0, 4, 0));
    x = x - 24 - offset;
    v.push(Button::new(ButtonType::Mute, x, y, 4, 1, 3, 1));
    x = x - 24 - offset;
    v.push(Button::new(ButtonType::Hide, x, y, 6, 0, 5, 0));
    x = x - 24 - offset;
    v.push(Button::new(ButtonType::Refresh, x, y, 3, 0, 3, 0));
    x = x - 24 - offset;
    v.push(Button::new(ButtonType::Play, x, y, 1, 0, 0, 0));
    v
}

fn check_buttons(x: i32, y: i32, window: &mut Window) {
    let p = Point::new(x, y);
    let timer = *TIMER.lock().unwrap();
    match timer.state {
        State::Prompt(prompt_type) => {
            // check the Ok / Cancel buttons if in a prompt
            let ok_rect = Rect::new(
                PROMPT_OK.0,
                PROMPT_OK.1,
                PROMPT_OK.2 as u32,
                PROMPT_OK.3 as u32,
            );
            let cancel_rect = Rect::new(
                PROMPT_CANCEL.0,
                PROMPT_CANCEL.1,
                PROMPT_CANCEL.2 as u32,
                PROMPT_CANCEL.3 as u32,
            );
            if ok_rect.contains_point(p) {
                if prompt_type == PromptType::Reset {
                    // reset the timer
                    let mut timer = *TIMER.lock().unwrap();
                    timer.current = timer.max;
                    timer.state = State::Paused;
                    *TIMER.lock().unwrap() = timer;
                } else if prompt_type == PromptType::Exit {
                    // exit the app
                    let mut timer = *TIMER.lock().unwrap();
                    timer.state = State::Exiting;
                    *TIMER.lock().unwrap() = timer;
                }
            } else if cancel_rect.contains_point(p) {
                // cancel the prompt
                let mut timer = *TIMER.lock().unwrap();
                timer.state = State::Paused;
                *TIMER.lock().unwrap() = timer;
            }
        }
        _ => {
            // check the regular buttons
            let buttons = BUTTONS.lock().unwrap();
            for b in buttons.iter() {
                if b.rect.contains_point(p) {
                    match b.name {
                        ButtonType::Hide => on_hide_clicked(window),
                        ButtonType::Refresh => on_refresh_clicked(),
                        ButtonType::Play => on_play_clicked(),
                        ButtonType::Settings => on_settings_clicked(),
                        ButtonType::Mute => on_mute_clicked(),
                    }
                }
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
    timer.state = State::Prompt(PromptType::Reset);
    *TIMER.lock().unwrap() = timer;
}

fn on_hide_clicked(window: &mut Window) {
    let bordered = !(*BORDERED.lock().unwrap());
    window.set_bordered(bordered);
    *BORDERED.lock().unwrap() = bordered;
}

fn on_settings_clicked() {
    println!("settings clicked");
}

fn on_mute_clicked() {
    println!("Mute clicked");
}

fn draw_buttons(canvas: &mut WindowCanvas, button_texture: &Texture) {
    let timer = *TIMER.lock().unwrap();
    let buttons = BUTTONS.lock().unwrap();
    for b in buttons.iter() {
        let mut src_rect = b.texture_rect;
        match b.name {
            ButtonType::Play => {
                if timer.state == State::Paused {
                    src_rect = b.texture_rect_alt;
                }
            }
            ButtonType::Hide => {
                if !(*BORDERED.lock().unwrap()) {
                    src_rect = b.texture_rect_alt;
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
    // the position of the character in the texture
    let ord = (c as i32) - 32;
    let x = (ord % 10) * 64;
    let y = (ord / 10) * 64;
    Rect::new(x, y, 64, 64)
}

fn draw_prompt(canvas: &mut WindowCanvas, font: &Font) {
    let timer = *TIMER.lock().unwrap();
    let message = match timer.state {
        State::Prompt(PromptType::Reset) => "Reset Timer?",
        State::Prompt(PromptType::Exit) => "Exit?",
        _ => "",
    };
    draw_text(
        canvas,
        font,
        message,
        WINDOW_WIDTH / 2 - WINDOW_WIDTH / 4,
        10,
    );
    draw_text(canvas, font, "OK", PROMPT_OK.0, PROMPT_OK.1);
    draw_text(canvas, font, "Cancel", PROMPT_CANCEL.0, PROMPT_CANCEL.1);
    let ok_rect = Rect::new(
        PROMPT_OK.0 - 2,
        PROMPT_OK.1 - 2,
        PROMPT_OK.2 as u32 + 4,
        PROMPT_OK.3 as u32 + 4,
    );
    let cancel_rect = Rect::new(
        PROMPT_CANCEL.0 - 2,
        PROMPT_CANCEL.1 - 2,
        PROMPT_CANCEL.2 as u32 + 4,
        PROMPT_CANCEL.3 as u32 + 4,
    );
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.draw_rect(ok_rect).unwrap();
    canvas.draw_rect(cancel_rect).unwrap();
}

fn draw_text(canvas: &mut WindowCanvas, font: &Font, text: &str, x: i32, y: i32) -> Rect {
    // render the text to a surface
    let surface = font.render(text).blended(Color::RGB(0, 0, 0)).unwrap();

    // convert the surface to a texture
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

    // get the size of the texture
    let TextureQuery { width, height, .. } = texture.query();
    let target = Rect::new(x, y, width, height);

    // copy the texture to the canvas
    canvas.copy(&texture, None, target).unwrap();

    target
}
