use cicada_window::{
    error::Error,
    event_loop::EventLoop,
    window::{ShowStyle, Window},
};

fn main() -> Result<(), Error> {
    let mut event_loop = EventLoop::builder()
        .with_resize_callback(|size| println!("{size:?}"))
        .build()?;

    let mut window = Window::builder().build(&event_loop)?;
    window.show(ShowStyle::Default);
    event_loop.set_move_callback(|pos| println!("{pos:?}"));

    loop {
        event_loop.poll_events();
    }
}
