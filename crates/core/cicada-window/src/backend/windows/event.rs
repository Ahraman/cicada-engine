use std::mem::MaybeUninit;

use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW, PeekMessageW,
        SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, PM_REMOVE, WM_NCCREATE,
        WM_NCDESTROY,
    },
};

use crate::event::{Attribs, ControlFlow};

use super::Error;

pub struct EventLoop {
    pub(super) hinstance: HINSTANCE,

    pub(super) control_flow: ControlFlow,
}

impl EventLoop {
    pub(crate) fn new(attribs: &Attribs) -> Result<Self, Error> {
        Self::create_unchecked(attribs)
    }

    pub(crate) fn process_events(&self) -> bool {
        match self.control_flow {
            ControlFlow::Poll => self.poll_events(),
            ControlFlow::Block => self.next_event(),
            ControlFlow::Exit => false,
        }
    }

    pub(crate) fn poll_events(&self) -> bool {
        Self::poll_events_unchecked()
    }

    pub(crate) fn next_event(&self) -> bool {
        Self::next_event_unchecked()
    }
}

impl EventLoop {
    fn create_unchecked(attribs: &Attribs) -> Result<Self, Error> {
        Ok(Self {
            hinstance: unsafe { GetModuleHandleW(None) }?.into(),
            control_flow: attribs.control_flow,
        })
    }

    fn poll_events_unchecked() -> bool {
        let mut msg = MaybeUninit::uninit();
        while unsafe { PeekMessageW(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE) }.as_bool() {
            let msg = unsafe { msg.assume_init() };

            _ = unsafe { TranslateMessage(&msg) };
            _ = unsafe { DispatchMessageW(&msg) };
        }

        true
    }

    fn next_event_unchecked() -> bool {
        let mut msg = MaybeUninit::uninit();
        if unsafe { GetMessageW(msg.as_mut_ptr(), None, 0, 0) }.as_bool() {
            let msg = unsafe { msg.assume_init() };

            _ = unsafe { TranslateMessage(&msg) };
            _ = unsafe { DispatchMessageW(&msg) };
        }

        true
    }
}

pub(super) struct CreateData {}

impl CreateData {
    pub(super) fn new() -> Self {
        Self {}
    }
}

pub(super) struct WindowData {}

impl WindowData {
    pub(super) fn new(_create_data: &mut CreateData) -> Self {
        Self {}
    }
}

pub(super) extern "system" fn generic_window_callback(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let user_data = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
    let window_data = match (msg, user_data) {
        (WM_NCCREATE, 0) => {
            let create_struct = unsafe { &*(lparam.0 as *const CREATESTRUCTW) };
            let create_data = unsafe { &mut *(create_struct.lpCreateParams as *mut CreateData) };

            let mut window_data = Box::new(WindowData::new(create_data));
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, window_data.as_mut() as *mut _ as _) };

            Box::leak(window_data)
        }
        (_, 0) => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        (WM_NCDESTROY, user_data) => {
            drop(unsafe { Box::from_raw(user_data as *mut WindowData) });
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
        (_, user_data) => unsafe { &mut *(user_data as *mut WindowData) },
    };

    inner_generic_window_callback(hwnd, window_data, msg, wparam, lparam)
}

fn inner_generic_window_callback(
    hwnd: HWND,
    _window_data: &mut WindowData,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
