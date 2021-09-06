extern crate pixels;
extern crate winit;

use crate::world::{ArrayWorld, ConstructableWorld};
use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use std::env;
use std::time::Instant;

mod draw;
mod game;
mod thread_pool;
mod world;
use game::Game;

const BLACK_COLOR: [u8; 4] = [0, 0, 0, 0];
const LIVE_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

fn get_args() -> Result<(u32, u32, u32, u32), &'static str> {
    let args: Vec<String> = env::args().collect();
    let args = &args.as_slice()[1..];

    if args.len() > 4 {
        return Err("Wrong arguments");
    }

    let width = args
        .get(0)
        .map(|x| x.parse::<u32>().ok())
        .flatten()
        .unwrap_or(128);
    let height = args
        .get(1)
        .map(|x| x.parse::<u32>().ok())
        .flatten()
        .unwrap_or(64);
    let num_threads = args
        .get(2)
        .map(|x| x.parse::<u32>().ok())
        .flatten()
        .unwrap_or(4);
    let ups = args
        .get(3)
        .map(|x| x.parse::<u32>().ok())
        .flatten()
        .unwrap_or(15);

    Ok((width, height, num_threads, ups))
}
fn main() {
    let (width, height, num_threads, ups) = get_args().unwrap();
    let initial_population = (width * height) / 2;

    let event_loop = EventLoop::new();

    let (window, p_width, p_height, mut hidpi_factor) =
        draw::create_window(width, height, "Conway's Game of Life", &event_loop);

    let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
    let mut pixels =
        Pixels::new(width, height, surface_texture).expect("Error creating pixels buffer");

    let world: ArrayWorld = ConstructableWorld::new(width, height);

    let mut game = Game::new(world, ups as i64, num_threads as usize);
    game.populate(initial_population);

    let mut previous = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                game.draw(LIVE_COLOR, BLACK_COLOR, pixels.get_frame());
                pixels.render().expect("Rendering error");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => {
                let size = size.to_logical::<f64>(hidpi_factor);
                pixels.resize_surface(size.width.round() as u32, size.height.round() as u32);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size: _,
                    },
                window_id,
            } if window_id == window.id() => {
                hidpi_factor = scale_factor;
            }
            _ => (),
        }
        let elapsed = now.duration_since(previous);
        game.update(elapsed.as_secs_f64());
        window.request_redraw();
        previous = now;
    });
}
