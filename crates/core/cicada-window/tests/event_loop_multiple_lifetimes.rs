use std::{cell::RefCell, rc::Rc};

use cicada_window::{
    error::Error,
    event::EventLoop,
    util::{Pos, Size},
};

struct App {
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self { running: true }
    }

    pub fn on_window_resize(&mut self, size: Size) {
        println!("{size:?}")
    }

    pub fn on_window_move(&mut self, _: Pos) {}
}

#[test]
fn event_loop_validity() -> Result<(), Error> {
    let app = Rc::new(RefCell::new(App::new()));
    event_loop_validity_inner(app.clone())?;
    println!("{}", app.borrow().running);

    Ok(())
}

fn event_loop_validity_inner(app: Rc<RefCell<App>>) -> Result<(), Error> {
    let event_loop = EventLoop::builder()
        .with_window_move(|pos| app.clone().borrow_mut().on_window_move(pos))
        .with_window_resize(|size| app.clone().borrow_mut().on_window_resize(size))
        .build()?;
    event_loop.run(&app, |_app, event_loop| {
        let mut x = 0;
        event_loop_validity_inner2(event_loop, &mut x);
        false
    });

    Ok(())
}

fn event_loop_validity_inner2(event_loop: &mut EventLoop, x: &mut u32) -> bool {
    // The following line of code is currently an error, because it does not live for long enough.
    //event_loop.set_window_resize(|_| *x += 1);

    event_loop.set_window_resize(|_| {});
    false
}
