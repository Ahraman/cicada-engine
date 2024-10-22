use std::mem::MaybeUninit;

use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, HMONITOR, MONITOR_DEFAULTTONEAREST,
        MONITOR_DEFAULTTOPRIMARY,
    },
};

use crate::{
    backend,
    util::{Pos, Rect, Size},
};

pub(crate) struct Monitor {
    hmonitor: HMONITOR,
}
impl Monitor {
    pub(crate) fn primary() -> Self {
        Self::from_pos(Default::default())
    }

    pub(crate) fn from_pos(pos: Pos) -> Self {
        Self::from_pos_unchecked(pos)
    }

    pub(crate) fn from_window(window: &backend::Window) -> Monitor {
        Self::from_window_unchecked(window.hwnd)
    }

    pub(crate) fn display_area(&self) -> Rect {
        Self::display_area_unchecked(self.hmonitor)
    }

    pub(crate) fn working_area(&self) -> Rect {
        Self::working_area_unchecked(self.hmonitor)
    }
}

impl Monitor {
    fn from_pos_unchecked(pos: Pos) -> Self {
        let point = POINT { x: pos.x, y: pos.y };
        Self {
            hmonitor: unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTOPRIMARY) },
        }
    }

    fn from_window_unchecked(hwnd: HWND) -> Self {
        Self {
            hmonitor: unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) },
        }
    }

    fn display_area_unchecked(hmonitor: HMONITOR) -> Rect {
        let mut monitor_info = MaybeUninit::uninit();
        _ = unsafe { GetMonitorInfoW(hmonitor, monitor_info.as_mut_ptr()) };
        let monitor_info = unsafe { monitor_info.assume_init() };

        let RECT {
            left,
            top,
            right,
            bottom,
        } = monitor_info.rcMonitor;

        Rect::new(
            Pos::new(left, top),
            Size::new((right - left) as u32, (bottom - top) as u32),
        )
    }

    fn working_area_unchecked(hmonitor: HMONITOR) -> Rect {
        let mut monitor_info = MaybeUninit::uninit();
        _ = unsafe { GetMonitorInfoW(hmonitor, monitor_info.as_mut_ptr()) };
        let monitor_info = unsafe { monitor_info.assume_init() };

        let RECT {
            left,
            top,
            right,
            bottom,
        } = monitor_info.rcWork;

        Rect::new(
            Pos::new(left, top),
            Size::new((right - left) as u32, (bottom - top) as u32),
        )
    }
}
