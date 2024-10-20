use crate::{
    backend::window::BackendAttribs,
    error::Error,
    event::EventLoop,
    util::{Pos, Size},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Windowed(Pos, Size),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowStyle {
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct Attribs {
    pub title: Option<String>,
    pub display_style: Option<DisplayStyle>,
    pub show_style: Option<ShowStyle>,
}

pub struct Window {}

impl Window {
    fn new(
        _event_loop: &mut EventLoop,
        _attribs: &Attribs,
        _backend_attribs: &BackendAttribs,
    ) -> Result<Self, Error> {
        todo!()
    }

    pub fn builder() -> Builder {
        Builder::new()
    }
}

pub struct Builder {
    pub(crate) attribs: Attribs,
    pub(crate) backend_attribs: BackendAttribs,
}

impl Builder {
    fn new() -> Self {
        Self {
            attribs: Default::default(),
            backend_attribs: Default::default(),
        }
    }

    pub fn build(&self, event_loop: &mut EventLoop) -> Result<Window, Error> {
        Window::new(event_loop, &self.attribs, &self.backend_attribs)
    }
}
