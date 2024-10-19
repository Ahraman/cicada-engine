use std::{
    cell::{RefCell, RefMut},
    mem::MaybeUninit,
    rc::{Rc, Weak},
};

use windows::Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetWindowLongPtrW, PeekMessageW, SetWindowLongPtrW,
        TranslateMessage, CREATESTRUCTW, GWLP_USERDATA, PM_REMOVE, WM_MOVE, WM_NCCREATE,
        WM_NCDESTROY, WM_SIZE,
    },
};

use crate::{
    event_loop::Dispatch,
    window::{Pos, Size},
};

use super::{error::BackendError, window::CreateData};

pub(crate) struct WindowData {
    pub(crate) dispatch: Weak<RefCell<Dispatch>>,
}

pub struct EventLoop {
    pub(crate) hinstance: HINSTANCE,

    pub(crate) dispatch: Rc<RefCell<Dispatch>>,
}

impl EventLoop {
    pub(crate) fn new(dispatch: Dispatch) -> Result<Self, BackendError> {
        Ok(Self {
            hinstance: unsafe { GetModuleHandleW(None) }?.into(),
            dispatch: Rc::new(RefCell::new(dispatch)),
        })
    }

    pub(crate) fn dispatch(&self) -> RefMut<Dispatch> {
        self.dispatch.borrow_mut()
    }

    pub(crate) fn poll_events(&self) {
        unsafe { self.poll_events_unchecked() };
    }
}

impl EventLoop {
    unsafe fn poll_events_unchecked(&self) {
        let mut msg = MaybeUninit::uninit();
        while PeekMessageW(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE).as_bool() {
            let msg = msg.assume_init();

            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }
}

pub(super) unsafe extern "system" fn common_window_callback(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let data = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    let data = match (msg, data) {
        (WM_NCCREATE, 0) => {
            let create_struct = &mut *(lparam.0 as *mut CREATESTRUCTW);
            let create_data = &mut *(create_struct.lpCreateParams as *mut CreateData);
            let data = Box::into_raw(Box::new(WindowData {
                dispatch: Rc::downgrade(&create_data.event_loop.dispatch),
            }));

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, data as isize);
            &mut *data
        }
        (_, 0) => return DefWindowProcW(hwnd, msg, wparam, lparam),
        (WM_NCDESTROY, data) => {
            drop(Box::from_raw(data as *mut WindowData));
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        (_, data) => &mut *(data as *mut WindowData),
    };

    inner_common_window_callback(hwnd, data, msg, wparam, lparam)
}

unsafe fn inner_common_window_callback(
    hwnd: HWND,
    data: &mut WindowData,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(dispatch) = data.dispatch.upgrade() {
        let mut dispatch = dispatch.borrow_mut();
        match msg {
            WM_MOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                dispatch.move_callback.on_move(Pos::new(x, y))
            }
            WM_SIZE => {
                let width = (lparam.0 & 0xFFFF) as u32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as u32;
                dispatch.resize_callback.on_resize(Size::new(width, height))
            }
            _ => {}
        };
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
