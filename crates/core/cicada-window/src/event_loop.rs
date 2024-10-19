use crate::{
    backend::event_loop::EventLoop as InnerEventLoop,
    error::Error,
    event::{WindowMove, WindowResize},
};

pub(crate) struct Dispatch {
    pub(crate) resize_callback: Box<dyn WindowResize>,
    pub(crate) move_callback: Box<dyn WindowMove>,
}

impl Default for Dispatch {
    fn default() -> Self {
        Self {
            resize_callback: Box::new(()),
            move_callback: Box::new(()),
        }
    }
}

impl Dispatch {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn set_resize_callback(&mut self, resize_callback: impl WindowResize + 'static) {
        self.resize_callback = Box::new(resize_callback);
    }

    pub(crate) fn set_move_callback(&mut self, move_callback: impl WindowMove + 'static) {
        self.move_callback = Box::new(move_callback);
    }
}

pub struct EventLoopBuilder {
    pub(crate) dispatch: Dispatch,
}

impl EventLoopBuilder {
    pub fn new() -> Self {
        Self {
            dispatch: Dispatch::new(),
        }
    }

    pub fn build(self) -> Result<EventLoop, Error> {
        EventLoop::from_dispatch(self.dispatch)
    }
}

impl EventLoopBuilder {
    pub fn with_resize_callback(mut self, resize_callback: impl WindowResize + 'static) -> Self {
        self.dispatch.set_resize_callback(resize_callback);
        self
    }

    pub fn with_move_callback(mut self, move_callback: impl WindowMove + 'static) -> Self {
        self.dispatch.set_move_callback(move_callback);
        self
    }
}

pub struct EventLoop {
    pub(crate) inner: InnerEventLoop,
}

impl EventLoop {
    pub fn new() -> Result<Self, Error> {
        Self::builder().build()
    }

    pub fn builder() -> EventLoopBuilder {
        EventLoopBuilder::new()
    }

    fn from_dispatch(dispatch: Dispatch) -> Result<Self, Error> {
        Ok(Self {
            inner: InnerEventLoop::new(dispatch)?,
        })
    }

    pub fn set_resize_callback(&mut self, resize_callback: impl WindowResize + 'static) {
        self.inner.dispatch().set_resize_callback(resize_callback);
    }

    pub fn set_move_callback(&mut self, move_callback: impl WindowMove + 'static) {
        self.inner.dispatch().set_move_callback(move_callback);
    }

    pub fn poll_events(&self) {
        self.inner.poll_events();
    }
}
