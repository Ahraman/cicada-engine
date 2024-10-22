use crate::{backend, error::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlFlow {
    #[default]
    Poll,
    Block,
    Exit,
}

pub struct EventLoop {
    pub(crate) backend: backend::EventLoop,
}

impl EventLoop {
    fn new(attribs: &Attribs) -> Result<Self, Error> {
        Ok(Self {
            backend: backend::EventLoop::new(attribs)?,
        })
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn process_events(&self) -> bool {
        self.backend.process_events()
    }

    pub fn control_flow(&self) -> ControlFlow {
        todo!()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Attribs {
    pub control_flow: ControlFlow,
}

pub struct Builder {
    pub(crate) attribs: Attribs,
}

impl Builder {
    fn new() -> Self {
        Self {
            attribs: Default::default(),
        }
    }

    pub fn with_control_flow(mut self, control_flow: ControlFlow) -> Self {
        self.attribs.control_flow = control_flow;
        self
    }

    pub fn build(&self) -> Result<EventLoop, Error> {
        EventLoop::new(&self.attribs)
    }
}
