use crate::{
    backend,
    util::{Pos, Rect},
};

pub struct Monitor {
    backend: backend::Monitor,
}

impl Default for Monitor {
    fn default() -> Self {
        Self::primary()
    }
}

impl Monitor {
    pub fn primary() -> Self {
        Self {
            backend: backend::Monitor::primary(),
        }
    }

    pub fn from_pos(pos: Pos) -> Self {
        Self {
            backend: backend::Monitor::from_pos(pos),
        }
    }

    pub(crate) fn from_handle(backend: backend::Monitor) -> Self {
        Self { backend }
    }

    pub fn display_area(&self) -> Rect {
        self.backend.display_area()
    }

    pub fn working_area(&self) -> Rect {
        self.backend.working_area()
    }
}
