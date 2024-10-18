use cicada_window::{
    error::WindowError,
    event_loop::EventLoop,
    window::{ShowStyle, Window, WindowAttribs, WindowPlatformSpecificAttribs},
};

fn main() -> Result<(), WindowError> {
    let mut event_loop = EventLoop::new()?.with_resize_callback(|size| println!("{size:?}"));

    let window_attribs = WindowAttribs::default();
    let platform_specific = WindowPlatformSpecificAttribs::default();

    let mut window = Window::new(&mut event_loop, window_attribs, platform_specific).unwrap();
    _ = window.show(ShowStyle::Default);

    loop {
        event_loop.poll_events();
    }
}
