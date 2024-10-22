use crate::{
    backend::{self, BackendWindowAttribs}, error::Error, event::EventLoop, monitor::Monitor, util::{Pos, Size}
};

#[derive(Debug, Clone, Copy, Default)]
pub enum DisplayStyle {
    #[default]
    Default,
    Windowed(Pos, Size),
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ShowStyle {
    #[default]
    Default,
    Visible,
    Hidden,
}

pub struct Window {
    backend: backend::Window,
}

impl Window {
    fn new(
        event_loop: &EventLoop,
        attribs: &WindowAttribs,
        backend_attribs: &BackendWindowAttribs,
    ) -> Result<Self, Error> {
        Ok(Self {
            backend: backend::Window::new(event_loop, attribs, backend_attribs)?,
        })
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn show(&mut self, show_style: ShowStyle) {
        self.backend.show(show_style)
    }

    pub fn monitor(&self) -> Monitor {
        Monitor::from_handle(self.backend.monitor())
    }
}

#[derive(Debug, Default)]
pub(crate) struct WindowAttribs {
    pub title: Option<String>,
    pub display_style: DisplayStyle,
}

#[derive(Debug, Default)]
pub struct Builder {
    pub(crate) attribs: WindowAttribs,
    pub(crate) backend_attribs: BackendWindowAttribs,
}

impl Builder {
    fn new() -> Self {
        Default::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.attribs.title = Some(title.into());
        self
    }

    pub fn with_display_style(mut self, display_style: DisplayStyle) -> Self {
        self.attribs.display_style = display_style;
        self
    }

    pub fn build(&self, event_loop: &EventLoop) -> Result<Window, Error> {
        Window::new(event_loop, &self.attribs, &self.backend_attribs)
    }
}
