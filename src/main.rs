// hide console on Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buttons;

extern crate sdl2;

use clap::Parser;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureQuery, WindowCanvas};
use sdl2::ttf::Font;
use sdl2::video::WindowPos;
use sdl2::{EventPump, mouse};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

const WINDOW_WIDTH: i32 = 185;
const WINDOW_HEIGHT: i32 = 70;

#[derive(Parser)]
struct Args {
    #[arg(short, default_value_t = 0)]
    x: i32, // window X position
    #[arg(short, default_value_t = 0)]
    y: i32, // window Y position
    #[arg(short = 'b')]
    hide_borders: bool, // hide window borders / decorations
    #[arg(default_value_t = 15,value_parser = clap::value_parser!(i32).range(1..=100))]
    minutes: i32, // timer start value (between 1 and 100, defaults to 15)
}

#[derive(PartialEq, Clone, Copy)]
pub enum PromptType {
    Reset,
    Exit,
}

#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Running,
    Paused,
    Done,
    Prompt(PromptType),
    Exiting,
}

pub struct App {
    pub state: State,
    pub timer_current: i32, // countdown time in milliseconds
    pub timer_max: i32,     // start value of timer
    pub window_borders: bool,
    pub muted: bool,
}

pub fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let timer_ms = args.minutes * 60 * 1000;
    
    // SDL initialization
    let sdl_context = sdl2::init().map_err(|e| format!("Failed to initialize SDL2: {}", e))?;
    let video_subsystem = sdl_context
        .video()
        .map_err(|e| format!("Failed to initialize video subsystem: {}", e))?;
    let timer_subsystem = sdl_context
        .timer()
        .map_err(|e| format!("Failed to initialize timer subsystem: {}", e))?;
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG)
        .map_err(|e| format!("Failed to initialize image context: {}", e))?;

    // font initialization
    let ttf_context =
        sdl2::ttf::init().map_err(|e| format!("Failed to initialize TTF context: {}", e))?;
    let font = ttf_context
        .load_font("res/font.ttf", 16)
        .map_err(|e| format!("Failed to load font 'res/font.ttf': {}", e))?;

    // create the window
    let mut window = video_subsystem
        .window("countdown", WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32)
        .hidden()
        .position_centered()
        .build()
        .map_err(|e| format!("Failed to create window: {}", e))?;

    // adjust the window based on args
    if args.x != 0 && args.y != 0 {
        window.set_position(WindowPos::Positioned(args.x), WindowPos::Positioned(args.y));
    }
    if args.hide_borders {
        window.set_bordered(false);
    }
    window.show();

    // create the canvas
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| format!("Failed to create canvas: {}", e))?;
    let texture_creator = canvas.texture_creator();

    // load the textures
    let mut textures = HashMap::new();
    let texture = texture_creator
        .load_texture("res/chars.png")
        .map_err(|e| format!("Failed to load texture 'res/chars.png': {}", e))?;
    textures.insert("chars", texture);
    let texture = texture_creator
        .load_texture("res/buttons.png")
        .map_err(|e| format!("Failed to load texture 'res/buttons.png': {}", e))?;
    textures.insert("buttons", texture);

    // create the buttons
    buttons::init(WINDOW_WIDTH, WINDOW_HEIGHT);

    let mut app = App {
        state: State::Running,
        timer_current: timer_ms,
        timer_max: timer_ms,
        window_borders: !args.hide_borders,
        muted: false,
    };

    let mut ticks = 0;

    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|e| format!("Failed to get event pump: {}", e))?;

    loop {
        handle_events(&mut event_pump, &mut app, &mut canvas)?;

        if app.state == State::Exiting {
            break;
        }

        let last_ticks = ticks;
        ticks = timer_subsystem.ticks64();
        let elapsed_time = ticks - last_ticks;

        if app.state == State::Running {
            update(elapsed_time, &mut app);
        }

        draw(&mut app, &mut canvas, &textures, &font)?;

        std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

fn handle_events(
    event_pump: &mut EventPump,
    app: &mut App,
    canvas: &mut WindowCanvas,
) -> Result<(), Box<dyn Error>> {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                app.state = State::Prompt(PromptType::Exit);
            }
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                match keycode {
                    Keycode::Escape => {
                        match app.state {
                            // if in a prompt, cancel it
                            State::Prompt(_) => app.state = State::Paused,
                            // else, prompt to exit
                            _ => app.state = State::Prompt(PromptType::Exit),
                        }
                    }
                    Keycode::Return | Keycode::KpEnter => {
                        buttons::click_ok(app);
                    }
                    _ => {}
                }
            }
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                if mouse_btn == mouse::MouseButton::Left {
                    match app.state {
                        State::Prompt(_) => buttons::check_prompt(x, y, app, true),
                        _ => buttons::check(x, y, app, canvas.window_mut(), true),
                    }
                }
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                if mouse_btn == mouse::MouseButton::Left {
                    match app.state {
                        State::Prompt(_) => buttons::check_prompt(x, y, app, false),
                        _ => buttons::check(x, y, app, canvas.window_mut(), false),
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn update(elapsed_time: u64, app: &mut App) {
    app.timer_current -= elapsed_time as i32;
    if app.timer_current <= 0 {
        app.timer_current = 0;
        app.state = State::Done;
    }
}

fn draw(
    app: &App,
    canvas: &mut WindowCanvas,
    textures: &HashMap<&str, Texture>,
    font: &Font,
) -> Result<(), Box<dyn Error>> {

    let bg_color = match app.state {
        // red when done
        State::Done => Color::RGB(255, 100, 100),
        // gray when paused
        State::Paused => Color::RGB(150,150,150),
        // blue when running
        _ => Color::RGB(200, 200, 255)
    };
    canvas.set_draw_color(bg_color);
    canvas.clear();

    match app.state {
        State::Prompt(_) => draw_prompt(app, canvas, font, &textures.get("buttons").unwrap())?,
        _ => {
            // draw the timer
            let offset = 8;
            let mut x = offset;
            let y = offset;
            let timer_str = timer_to_string(app.timer_current);
            let texture = textures.get("chars").unwrap();
            for c in timer_str.chars() {
                draw_char(c, x, y, canvas, texture)?;
                x += 32;
            }
            buttons::draw(app, canvas, &textures.get("buttons").unwrap())?;
        }
    }
    canvas.present();
    Ok(())
}

fn timer_to_string(timer: i32) -> String {
    let secs = timer / 1000;
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

fn draw_char(
    c: char,
    x: i32,
    y: i32,
    canvas: &mut WindowCanvas,
    char_texture: &Texture,
) -> Result<(), Box<dyn Error>> {
    let src_rect = char_rect(c);
    let dst_rect = Rect::new(x, y, 32, 32);
    canvas
        .copy(&char_texture, src_rect, dst_rect)
        .map_err(|e| format!("Failed to copy char texture: {}", e).into())
}

fn char_rect(c: char) -> Rect {
    // the position of the character in the texture
    let ord = (c as i32) - 32;
    let x = (ord % 10) * 64;
    let y = (ord / 10) * 64;
    Rect::new(x, y, 64, 64)
}

fn draw_prompt(
    app: &App,
    canvas: &mut WindowCanvas,
    font: &Font,
    button_texture: &Texture,
) -> Result<(), Box<dyn Error>> {
    let message = match app.state {
        State::Prompt(PromptType::Reset) => "Reset Timer?",
        State::Prompt(PromptType::Exit) =>  "  Exit?",
        _ => "",
    };
    draw_text(message, WINDOW_WIDTH / 4, 10, canvas, font)?;
    let buttons = buttons::PROMPT_BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_ref() {
        for b in buttons.iter() {
            let mut dst_rect = b.rect;
            // offset the image if the button is pressed
            if b.pressed {
                dst_rect.x += 1;
                dst_rect.y += 1;
            }
            canvas
                .copy(&button_texture, b.texture_rect, dst_rect)
                .map_err(|e| format!("Failed to copy button texture: {}", e))?;
        }
    }
    Ok(())
}

fn draw_text(
    text: &str,
    x: i32,
    y: i32,
    canvas: &mut WindowCanvas,
    font: &Font,
) -> Result<Rect, Box<dyn Error>> {
    // render the text to a surface
    let surface = font
        .render(text)
        .blended(Color::RGB(0, 0, 0))
        .map_err(|e| format!("Failed to render text: {}", e))?;
    // convert the surface to a texture
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| format!("Failed to create texture from surface: {}", e))?;
    // get the size of the texture
    let TextureQuery { width, height, .. } = texture.query();
    let target_rect = Rect::new(x, y, width, height);
    // copy the texture to the canvas
    canvas
        .copy(&texture, None, target_rect)
        .map_err(|e| format!("Failed to copy text texture: {}", e))?;
    Ok(target_rect)
}
