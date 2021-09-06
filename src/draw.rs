use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event_loop::EventLoop;

pub fn create_window(
    width: u32,
    height: u32,
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let scale_factor = window.scale_factor();
    let width = width as f64;
    let height = height as f64;

    // Get dimensions
    let (monitor_width, monitor_height) = {
        let size = window.current_monitor().unwrap().size();
        (
            size.width as f64 / scale_factor,
            size.height as f64 / scale_factor,
        )
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round();

    // Resize, center, and display the window
    let min_size = PhysicalSize::new(width, height).to_logical::<f64>(scale_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    // let surface = pixels::wgpu::Surface::create(&window);
    let size = default_size.to_physical::<f64>(scale_factor);

    (
        window,
        // surface,
        size.width.round() as u32,
        size.height.round() as u32,
        scale_factor,
    )
}
