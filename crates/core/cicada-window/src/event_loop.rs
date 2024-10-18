use std::mem::MaybeUninit;

use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, PeekMessageW, SetWindowLongPtrW,
        TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, PM_REMOVE, WM_NCCREATE, WM_NCDESTROY,
        WM_SIZE,
    },
};

use crate::{error::WindowError, event::WindowResize, window::CreateData};

pub(crate) struct WindowData<'a, T0>
where
    T0: WindowResize,
{
    resize_callback: &'a mut T0,
}

pub struct EventLoop<T0 = ()>
where
    T0: WindowResize,
{
    hinstance: HINSTANCE,

    pub(crate) resize_callback: T0,
}

impl EventLoop<()> {
    pub fn new() -> Result<Self, WindowError> {
        Ok(Self {
            hinstance: unsafe { GetModuleHandleW(None) }?.into(),
            resize_callback: (),
        })
    }
}

impl EventLoop<()> {
    pub fn with_resize_callback<T>(self, resize_callback: T) -> EventLoop<T>
    where
        T: WindowResize,
    {
        EventLoop {
            hinstance: self.hinstance,
            resize_callback,
        }
    }
}

impl<T0> EventLoop<T0>
where
    T0: WindowResize,
{
    pub fn poll_events(&mut self) {
        unsafe { self.poll_events_unchecked() };
    }
}

impl<T0> EventLoop<T0>
where
    T0: WindowResize,
{
    unsafe fn poll_events_unchecked(&mut self) {
        let mut msg = MaybeUninit::uninit();
        while PeekMessageW(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE).as_bool() {
            let msg = msg.assume_init();

            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }

    pub(crate) unsafe extern "system" fn common_window_callback(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let data = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        let data = match (msg, data) {
            (WM_NCCREATE, 0) => {
                let create_struct = &mut *(lparam.0 as *mut CREATESTRUCTW);
                let create_data = &mut *(create_struct.lpCreateParams as *mut CreateData<T0>);
                let data = Box::into_raw(Box::new(WindowData {
                    resize_callback: &mut create_data.event_loop.resize_callback,
                }));

                SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
                &mut *data
            }
            (_, 0) => return DefWindowProcW(hwnd, msg, wparam, lparam),
            (WM_NCDESTROY, data) => {
                drop(Box::from_raw(data as *mut WindowData<T0>));
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            (_, data) => &mut *(data as *mut WindowData<T0>),
        };

        Self::inner_common_window_callback(hwnd, data, msg, wparam, lparam)
    }

    unsafe fn inner_common_window_callback(
        hwnd: HWND,
        data: &mut WindowData<T0>,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT
    where
        T0: WindowResize,
    {
        match msg {
            WM_SIZE => data.resize_callback.on_resize(),
            _ => {}
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
