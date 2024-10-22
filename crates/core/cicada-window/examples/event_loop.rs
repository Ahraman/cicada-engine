use cicada_window::{
    error::Error,
    event::EventLoop,
    window::{ShowStyle, Window},
};

struct App {
    _window: Window,
}

impl App {
    pub fn new(event_loop: &EventLoop) -> Result<Self, Error> {
        Ok(Self {
            _window: Self::create_window(event_loop)?,
        })
    }

    fn create_window(event_loop: &EventLoop) -> Result<Window, Error> {
        let mut window = Window::builder()
            .with_title("Test Window")
            .build(event_loop)?;
        window.show(ShowStyle::Visible);

        Ok(window)
    }

    pub fn update(&mut self) {}
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::builder().build()?;
    let mut app = App::new(&event_loop)?;

    loop {
        app.update();

        if !event_loop.process_events() {
            break;
        }
    }

    Ok(())
}
