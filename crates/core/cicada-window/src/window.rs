use crate::{backend::window::Window as InnerWindow, error::Error, event_loop::EventLoop};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Windowed(Pos, Size),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowStyle {
    Default,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct WindowAttribs {
    pub title: Option<String>,
    pub display_style: Option<DisplayStyle>,
    pub show_style: Option<ShowStyle>,
}

pub(crate) type WindowPlatformSpecificAttribs =
    crate::backend::window::WindowPlatformSpecificAttribs;

pub struct WindowBuilder {
    pub(crate) attribs: WindowAttribs,
    pub(crate) platform_specific: WindowPlatformSpecificAttribs,
}

impl WindowBuilder {
    fn new() -> Self {
        Self {
            attribs: Default::default(),
            platform_specific: Default::default(),
        }
    }

    pub fn build(&self, event_loop: &EventLoop) -> Result<Window, Error> {
        Window::new(event_loop, &self.attribs, &self.platform_specific)
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.attribs.title = Some(title);
        self
    }

    pub fn with_display_style(mut self, display_style: DisplayStyle) -> Self {
        self.attribs.display_style = Some(display_style);
        self
    }

    pub fn with_show_style(mut self, show_style: ShowStyle) -> Self {
        self.attribs.show_style = Some(show_style);
        self
    }
}

pub struct Window {
    inner: InnerWindow,
}

impl Window {
    fn new(
        event_loop: &EventLoop,
        attribs: &WindowAttribs,
        platform_specific: &WindowPlatformSpecificAttribs,
    ) -> Result<Self, Error> {
        Ok(Self {
            inner: InnerWindow::new(event_loop, attribs, platform_specific)?,
        })
    }

    pub fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }
}

impl Window {
    pub fn show(&mut self, show_style: ShowStyle) {
        self.inner.show(show_style)
    }
}
