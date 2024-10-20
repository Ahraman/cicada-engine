use std::ops::Deref;

use crate::{
    error::Error,
    util::{Pos, Size},
};

#[derive(Default)]
pub struct Dispatcher<'a> {
    pub window_resize: Option<Box<dyn FnMut(Size) + 'a>>,
    pub window_move: Option<Box<dyn FnMut(Pos) + 'a>>,
}

pub struct EventLoop<'a> {
    dispatcher: Dispatcher<'a>,
}

impl<'a> EventLoop<'a> {
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    pub fn builder() -> Builder<'a> {
        Builder::new()
    }

    pub fn set_window_move<F>(&mut self, window_move: F)
    where
        F: FnMut(Pos) + 'a,
    {
        self.dispatcher.window_move = Some(Box::new(window_move));
    }

    pub fn set_window_resize<F>(&mut self, window_resize: F)
    where
        F: FnMut(Size) + 'a,
    {
        self.dispatcher.window_resize = Some(Box::new(window_resize));
    }

    pub fn run<A, F>(mut self, app: A, mut callback: F)
    where
        A: Deref,
        F: FnMut(&A::Target, &mut Self) -> bool,
    {
        loop {
            self.poll_events();

            if !callback(app.deref(), &mut self) {
                break;
            }
        }
    }

    pub fn poll_events(&mut self) {
        if let Some(window_resize) = self.dispatcher.window_resize.as_deref_mut() {
            window_resize(Default::default());
        }
    }
}

pub struct Builder<'a> {
    dispatcher: Dispatcher<'a>,
}

impl<'a> Builder<'a> {
    fn new() -> Self {
        Self {
            dispatcher: Default::default(),
        }
    }

    pub fn build(self) -> Result<EventLoop<'a>, Error> {
        Ok(EventLoop {
            dispatcher: self.dispatcher,
        })
    }

    pub fn with_window_move<F>(mut self, window_move: F) -> Self
    where
        F: FnMut(Pos) + 'a,
    {
        self.dispatcher.window_move = Some(Box::new(window_move));
        self
    }

    pub fn with_window_resize<F>(mut self, window_resize: F) -> Self
    where
        F: FnMut(Size) + 'a,
    {
        self.dispatcher.window_resize = Some(Box::new(window_resize));
        self
    }
}
