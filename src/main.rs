use hachi_core::*;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

const SCALE: u32 = 20;
const WINDOW_WIDTH: u32 = (hachi_core::DISPLAY_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (hachi_core::DISPLAY_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("usage hachi path/to/game");
        return;
    }

    let rom_name = &args[1];
    let window_name = format!("hachi - {}", rom_name);

    // setup sdl
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsys = sdl_ctx.video().unwrap();
    let audio_subsys = sdl_ctx.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let audio_device = audio_subsys
        .open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })
        .unwrap();
    let window = video_subsys
        .window(&window_name, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut hachi = Hachi::new();

    let mut rom = File::open(&args[1]).expect("unable to oper rom");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    hachi.load(&buffer);

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        hachi.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        hachi.keypress(k, false);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            hachi.tick();
        }

        hachi.tick_timers();
        if hachi.get_audio() == 1 {
            audio_device.resume();
            std::thread::sleep(Duration::from_millis(50));
        } else {
            audio_device.pause();
        }

        draw_screen(&hachi, &mut canvas);
    }
}

fn draw_screen(h: &Hachi, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(1, 39, 40));
    canvas.clear();

    let screen_buffer = h.get_display();
    canvas.set_draw_color(Color::RGB(201, 235, 32));
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % hachi_core::DISPLAY_WIDTH) as u32;
            let y = (i / hachi_core::DISPLAY_WIDTH) as u32;
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }

    canvas.present();
}

fn key_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        //generate a sq wave
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
