#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use evo_grid::world::{GridCell, WorldGrid};
use log::{/* debug, */ error};
use pixels::{Error, Pixels, PixelsBuilder, SurfaceTexture};
use pixels::wgpu::Color;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::WindowBuilder,
};
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 300;

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = build_window(&event_loop);
    let mut pixels = build_pixels(&window)?;

    let mut grid = WorldGrid::new(WIDTH as usize, HEIGHT as usize);

    let mut input = WinitInputHelper::new();
    let mut paused = false;

    let res = event_loop.run(|event, elwt| {
        // The one and only event that winit_input_helper doesn't have for us...
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            draw_grid_cells(&grid, pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }
            if input.key_pressed(KeyCode::KeyP) {
                paused = !paused;
            }
            if input.key_pressed_os(KeyCode::Space) {
                // Space is frame-step, so ensure we're paused
                paused = true;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }
            if !paused || input.key_pressed_os(KeyCode::Space) {
                grid.update();
            }
            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn build_window(event_loop: &EventLoop<()>) -> Window {
    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let scaled_size = LogicalSize::new(WIDTH as f64 * 3.0, HEIGHT as f64 * 3.0);
    WindowBuilder::new()
        .with_title("Conway's Game of Life")
        .with_inner_size(scaled_size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap()
}

fn build_pixels(window: &Window) -> Result<Pixels, Error> {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
        .clear_color(Color::WHITE)
        .build()
}

fn draw_grid_cells(grid: &WorldGrid, screen: &mut [u8]) {
    debug_assert_eq!(screen.len(), 4 * grid.num_cells());
    for (cell, pixel) in grid.cells_iter().zip(screen.chunks_exact_mut(4)) {
        let color_rgba = render_cell(cell);
        pixel.copy_from_slice(&color_rgba);
    }
}

fn render_cell(cell: &GridCell) -> [u8; 4] {
    let color_rgba =
        if let Some(substance) = cell.substance {
            let color_rgb = substance.color;
            let color_alpha = (substance.amount * 0xff as f32) as u8;
            [color_rgb[0], color_rgb[1], color_rgb[2], color_alpha]
        } else {
            [0, 0, 0, 0]
        };
    color_rgba
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
