pub trait WindowResize {
    fn on_resize(&mut self);
}

impl WindowResize for () {
    fn on_resize(&mut self) {}
}

impl<T> WindowResize for T
where
    T: FnMut(),
{
    fn on_resize(&mut self) {
        self()
    }
}
