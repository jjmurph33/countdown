extern crate sdl2;

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

enum ButtonType {
    Play,
    Refresh,
    Hide,
    Mute,
    Ok,
    Cancel,
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
    timer_current: i32, // countdown time in milliseconds
    timer_max: i32,     // start value of timer
    window_borders: bool,
    muted: bool,
}

static BUTTONS: Mutex<Option<[Button; 4]>> = Mutex::new(None);
static PROMPT_BUTTONS: Mutex<Option<[Button; 2]>> = Mutex::new(None);

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
    let timer_ms = timer_minutes * 60 * 1000;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();
    let font = ttf_context.load_font("res/font.ttf", 16).unwrap();

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
    *BUTTONS.lock().unwrap() = Some(init_buttons());
    *PROMPT_BUTTONS.lock().unwrap() = Some(init_prompt_buttons());

    let mut app = App {
        state: State::Running,
        timer_current: timer_ms,
        timer_max: timer_ms,
        window_borders: true,
        muted: false,
    };

    let mut ticks = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        if app.state == State::Exiting {
            break;
        }

        handle_events(&mut event_pump, &mut app,&mut canvas);

        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        let elapsed_time = ticks - last_ticks;

        if app.state == State::Running {
            update(elapsed_time,&mut app);
        }

        draw(&mut app,&mut canvas, &textures, &font);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn handle_events(event_pump: &mut EventPump, app: &mut App,canvas: &mut WindowCanvas) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                app.state = State::Prompt(PromptType::Exit);
            }
            Event::KeyDown { keycode, .. } => match keycode {
                Some(Keycode::Escape) => {
                    match app.state {
                        // if in a prompt, cancel it
                        State::Prompt(_) => app.state = State::Paused,
                        // else, prompt to exit
                        _ => app.state = State::Prompt(PromptType::Exit),
                    }
                }
                _ => {}
            },
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                if mouse_btn == mouse::MouseButton::Left {
                    match app.state {
                        State::Prompt(_) => check_prompt_buttons(x, y,app),
                        _ => check_buttons(x, y, app,canvas.window_mut()),
                    }
                }
            }
            _ => {}
        }
    }
}

fn update(elapsed_time: u64,app: &mut App) {
    app.timer_current -= elapsed_time as i32;
    if app.timer_current <= 0 {
        app.timer_current = 0;
        app.state = State::Done;
    }
}

fn draw(app: &mut App,canvas: &mut WindowCanvas, textures: &HashMap<&str, Texture>, font: &Font) {
    // clear the screen
    let bg_color = if app.state == State::Done {
        Color::RGB(255, 100, 100) // Red when done
    } else {
        Color::RGB(200, 200, 255)
    };
    canvas.set_draw_color(bg_color);
    canvas.clear();

    match app.state {
        State::Prompt(_) => draw_prompt(app,canvas, font, &textures.get("buttons").unwrap()),
        _ => {
            // draw the timer
            let offset = 8;
            let mut x = offset;
            let y = offset;
            let timer_str = timer_to_string(app.timer_current);
            let texture = textures.get("chars").unwrap();
            for c in timer_str.chars() {
                draw_char(c, x, y,canvas, texture);
                x += 32;
            }
            draw_buttons(app,canvas, &textures.get("buttons").unwrap());
        }
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

fn init_buttons() -> [Button; 4] {
    let offset = 0;
    let mut x = WINDOW_WIDTH - 62 - offset;
    let y = WINDOW_HEIGHT - 24 - offset;

    let mute = Button::new(ButtonType::Mute, x, y, 4, 1, 3, 1);
    x = x - 24 - offset;
    let hide = Button::new(ButtonType::Hide, x, y, 6, 0, 5, 0);
    x = x - 24 - offset;
    let refresh = Button::new(ButtonType::Refresh, x, y, 3, 0, 3, 0);
    x = x - 24 - offset;
    let play = Button::new(ButtonType::Play, x, y, 1, 0, 0, 0);
    [mute, hide, refresh, play]
}

fn init_prompt_buttons() -> [Button; 2] {
    let offset = 0;
    let mut x = WINDOW_WIDTH / 4;
    let y = WINDOW_HEIGHT / 2;
    let ok = Button::new(ButtonType::Ok, x, y, 5, 1, 5, 1);
    x = x + 64 + offset;
    let cancel = Button::new(ButtonType::Cancel, x, y, 6, 1, 6, 1);
    [ok, cancel]
}

fn check_buttons(x: i32, y: i32, app: &mut App,window: &mut Window) {
    let p = Point::new(x, y);
    let buttons = BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_ref() {
        for b in buttons.iter() {
            if b.rect.contains_point(p) {
                match b.name {
                    ButtonType::Hide => on_hide_clicked(app,window),
                    ButtonType::Refresh => on_refresh_clicked(app),
                    ButtonType::Play => on_play_clicked(app),
                    ButtonType::Mute => on_mute_clicked(app),
                    _ => {}
                }
                return; // Early return after handling a button click
            }
        }
    }
}

fn check_prompt_buttons(x: i32, y: i32,app: &mut App) {
    let p = Point::new(x, y);
    let buttons = PROMPT_BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_ref() {
        for b in buttons.iter() {
            if b.rect.contains_point(p) {
                match b.name {
                    ButtonType::Ok => on_ok_clicked(app),
                    ButtonType::Cancel => on_cancel_clicked(app),
                    _ => {}
                }
                return; // Early return after handling a button click
            }
        }
    }
}

fn on_play_clicked(app: &mut App) {
    match app.state {
        State::Paused => {
            app.state = State::Running;
        }
        State::Running => {
            app.state = State::Paused;
        }
        _ => {}
    }
}

fn on_refresh_clicked(app: &mut App) {
    app.state = State::Prompt(PromptType::Reset);
}

fn on_hide_clicked(app: &mut App,window: &mut Window) {
    app.window_borders = !app.window_borders;
    window.set_bordered(app.window_borders);
}

fn on_mute_clicked(app: &mut App) {
    app.muted = !app.muted;
    println!("Mute clicked");
}

fn on_ok_clicked(app: &mut App) {
    match app.state {
        State::Prompt(prompt_type) => {
            if prompt_type == PromptType::Reset {
                // reset the timer
                app.timer_current = app.timer_max;
                app.state = State::Paused;
            } else if prompt_type == PromptType::Exit {
                // exit the app
                app.state = State::Exiting;
            }
        }
        _ => {}
    }
}

fn on_cancel_clicked(app: &mut App) {
    match app.state {
        State::Prompt(_) => {
            // cancel the prompt
            app.state = State::Paused;
        }
        _ => {}
    }
}

fn draw_buttons(app: &mut App,canvas: &mut WindowCanvas, button_texture: &Texture) {
    let buttons = BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_ref() {
        for b in buttons.iter() {
            let mut src_rect = b.texture_rect;
            match b.name {
                ButtonType::Play => {
                    if app.state == State::Paused {
                        src_rect = b.texture_rect_alt;
                    }
                }
                ButtonType::Hide => {
                    if !app.window_borders {
                        src_rect = b.texture_rect_alt;
                    }
                }
                ButtonType::Mute => {
                    if !app.muted {
                        src_rect = b.texture_rect_alt;
                    }
                }
                _ => {}
            }
            canvas.copy(&button_texture, src_rect, b.rect).unwrap();
        }
    }
}

fn draw_char(c: char, x: i32, y: i32,canvas: &mut WindowCanvas, char_texture: &Texture) {
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

fn draw_prompt(app: &mut App,canvas: &mut WindowCanvas, font: &Font, button_texture: &Texture) {
    let message = match app.state {
        State::Prompt(PromptType::Reset) => "Reset Timer?",
        State::Prompt(PromptType::Exit) => "Exit?",
        _ => "",
    };
    draw_text( message, WINDOW_WIDTH / 4, 10,canvas, font);
    let buttons = PROMPT_BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_ref() {
        for b in buttons.iter() {
            canvas
                .copy(&button_texture, b.texture_rect, b.rect)
                .unwrap();
        }
    }
}

fn draw_text(text: &str, x: i32, y: i32,canvas: &mut WindowCanvas, font: &Font) -> Rect {
    // render the text to a surface
    let surface = font.render(text).blended(Color::RGB(0, 0, 0)).unwrap();
    // convert the surface to a texture
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    // get the size of the texture
    let TextureQuery { width, height, .. } = texture.query();
    let target_rect = Rect::new(x, y, width, height);
    // copy the texture to the canvas
    canvas.copy(&texture, None, target_rect).unwrap();
    target_rect
}
