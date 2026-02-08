use sdl2::rect::{Point, Rect};
use sdl2::render::Texture;
use sdl2::video::Window;
use std::sync::Mutex;

use crate::{App,State,PromptType};

#[derive(PartialEq, Clone, Copy)]
pub enum ButtonType {
    Play,
    Refresh,
    Hide,
    Mute,
    Exit,
    Ok,
    Cancel,
}

pub struct Button {
    pub name: ButtonType,
    pub rect: Rect,             // position on the screen
    pub texture_rect: Rect,     // position in the texture
    pub texture_rect_alt: Rect, // position in the texture of the alternate icon (ex: play/pause)
    pub pressed: bool,
}

impl Button {
    pub fn new(
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
            pressed: false,
        }
    }
}

pub static BUTTONS: Mutex<Option<[Button; 5]>> = Mutex::new(None);
pub static PROMPT_BUTTONS: Mutex<Option<[Button; 2]>> = Mutex::new(None);

pub fn init(window_width: i32, window_height: i32) {
    let mut x = window_width - 24;
    let mut y = window_height - 24;
    let exit = Button::new(ButtonType::Exit, x, y, 6, 1, 6, 1);
    x -= 24;
    let mute = Button::new(ButtonType::Mute, x, y, 4, 1, 3, 1);
    x -= 24;
    let hide = Button::new(ButtonType::Hide, x, y, 6, 0, 5, 0);
    x -= 24;
    let refresh = Button::new(ButtonType::Refresh, x, y, 3, 0, 3, 0);
    x -= 24;
    let play = Button::new(ButtonType::Play, x, y, 1, 0, 0, 0);
    *BUTTONS.lock().unwrap() = Some([exit, mute, hide, refresh, play]);

    x = window_width / 4;
    y = window_height / 2;
    let ok = Button::new(ButtonType::Ok, x, y, 5, 1, 5, 1);
    x = x + 64;
    let cancel = Button::new(ButtonType::Cancel, x, y, 6, 1, 6, 1);
    *PROMPT_BUTTONS.lock().unwrap() = Some([ok, cancel]);
}

pub fn check(x: i32, y: i32, app: &mut App, window: &mut Window, down: bool) {
    let p = Point::new(x, y);
    let mut buttons = BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_mut() {
        for b in buttons.iter_mut() {
            b.pressed = false;
            if b.rect.contains_point(p) {
                if down {
                    b.pressed = true;
                } else {
                    // mouse button released
                    match b.name {
                        ButtonType::Hide => click_hide(app, window),
                        ButtonType::Refresh => click_refresh(app),
                        ButtonType::Play => click_play(app),
                        ButtonType::Mute => click_mute(app),
                        ButtonType::Exit => click_exit(app),
                        ButtonType::Ok | ButtonType::Cancel => {} // not in buttons array
                    }
                    return;
                }
            }
        }
    }
}

pub fn check_prompt(x: i32, y: i32, app: &mut App, down: bool) {
    let p = Point::new(x, y);
    let mut buttons = PROMPT_BUTTONS.lock().unwrap();
    if let Some(buttons) = buttons.as_mut() {
        for b in buttons.iter_mut() {
            b.pressed = false;
            if b.rect.contains_point(p) {
                b.pressed = down;
                if down {
                    b.pressed = true;
                } else {
                    // mouse button released
                    match b.name {
                        ButtonType::Ok => click_ok(app),
                        ButtonType::Cancel => click_cancel(app),
                        _ => {} // only Ok and Cancel in the array
                    }
                    return;
                }
            }
        }
    }
}

pub fn draw(
    app: &App,
    canvas: &mut sdl2::render::WindowCanvas,
    button_texture: &Texture,
) -> Result<(), String> {
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

            let mut dst_rect = b.rect;

            // offset the image if the button is pressed
            if b.pressed {
                dst_rect.x += 1;
                dst_rect.y += 1;
            }

            canvas
                .copy(&button_texture, src_rect, dst_rect)
                .map_err(|e| format!("Failed to copy button texture: {}", e))?;
        }
    }
    Ok(())
}

fn click_play(app: &mut App) {
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

fn click_refresh(app: &mut App) {
    app.state = State::Prompt(PromptType::Reset);
}

fn click_hide(app: &mut App, window: &mut Window) {
    app.window_borders = !app.window_borders;
    window.set_bordered(app.window_borders);
}

fn click_mute(app: &mut App) {
    app.muted = !app.muted;
    println!("Mute clicked");
}

fn click_exit(app: &mut App) {
    app.state = State::Prompt(PromptType::Exit);
}

pub fn click_ok(app: &mut App) {
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

pub fn click_cancel(app: &mut App) {
    match app.state {
        State::Prompt(_) => {
            // cancel the prompt
            app.state = State::Paused;
        }
        _ => {}
    }
}
