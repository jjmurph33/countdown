extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::image::LoadTexture;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let timer_subsystem = sdl_context.timer().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 200,100)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let char_texture = texture_creator.load_texture("res/chars.png").unwrap();

    let mut ticks = timer_subsystem.ticks64();
    let mut elapsed_ms = 0;
    let mut timer = 500;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

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

        canvas.set_draw_color(Color::RGB(200, 200, 255));
        canvas.clear();

        //draw(&mut canvas,&char_texture);
        let mut x = 8;
        let y = 8;
        for c in timer_str.chars() {
            draw_char(&mut canvas,&char_texture,c,x,y);
            x += 32;
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

//fn draw(canvas: &mut WindowCanvas,char_texture: &Texture) {
//    let x = 8;
//    let y = 8;
//    draw_char(canvas,char_texture,'A',x,y);
//}

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
