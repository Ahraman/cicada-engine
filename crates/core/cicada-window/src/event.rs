use crate::window::{Pos, Size};

pub trait WindowResize {
    fn on_resize(&mut self, size: Size);
}

impl WindowResize for () {
    fn on_resize(&mut self, _: Size) {}
}

impl<T> WindowResize for T
where
    T: FnMut(Size),
{
    fn on_resize(&mut self, size: Size) {
        self(size)
    }
}

pub trait WindowMove {
    fn on_move(&mut self, pos: Pos);
}

impl WindowMove for () {
    fn on_move(&mut self, _: Pos) {}
}

impl<T> WindowMove for T
where
    T: FnMut(Pos),
{
    fn on_move(&mut self, pos: Pos) {
        self(pos)
    }
}
