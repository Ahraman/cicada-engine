use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_CLASS_ALREADY_EXISTS, HWND},
        Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, LoadCursorW, LoadIconW, RegisterClassExW, ShowWindow,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, SW_HIDE, SW_SHOW,
            SW_SHOWDEFAULT, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{
    backend::{
        self,
        inner::{CreateData, WideStr},
    },
    event::EventLoop,
    window::{Builder, DisplayStyle, ShowStyle, WindowAttribs},
};

use super::{generic_window_callback, Error};

pub(crate) struct Window {
    pub(super) hwnd: HWND,
}

impl Window {
    pub(crate) fn new(
        event_loop: &EventLoop,
        attribs: &WindowAttribs,
        backend_attribs: &BackendWindowAttribs,
    ) -> Result<Self, Error> {
        Self::create_unchecked(&event_loop.backend, attribs, backend_attribs)
    }

    pub(crate) fn show(&mut self, show_style: ShowStyle) {
        Self::show_unchecked(self.hwnd, show_style)
    }
    
    pub(crate) fn monitor(&self) -> backend::Monitor {
        backend::Monitor::from_window(self)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        Self::destroy_unchecked(self.hwnd);
        self.hwnd = Default::default();
    }
}

impl Window {
    fn create_unchecked(
        event_loop: &backend::EventLoop,
        attribs: &WindowAttribs,
        backend_attribs: &BackendWindowAttribs,
    ) -> Result<Self, Error> {
        let class_name = Self::register_class_unchecked(
            &event_loop,
            backend_attribs
                .class_name
                .as_deref()
                .unwrap_or("window_class"),
        )?;

        let title = WideStr::from_os_str(attribs.title.as_deref().unwrap_or("Window"));

        struct Create {
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            style: WINDOW_STYLE,
            style_ex: WINDOW_EX_STYLE,
        }

        let create_info = match attribs.display_style {
            DisplayStyle::Windowed(pos, size) => Create {
                x: pos.x,
                y: pos.y,
                width: size.width as i32,
                height: size.height as i32,
                style: WS_OVERLAPPEDWINDOW,
                style_ex: Default::default(),
            },
            DisplayStyle::Default => Create {
                x: CW_USEDEFAULT,
                y: CW_USEDEFAULT,
                width: CW_USEDEFAULT,
                height: CW_USEDEFAULT,
                style: WS_OVERLAPPEDWINDOW,
                style_ex: Default::default(),
            },
        };

        let mut create_data = CreateData::new();
        let hwnd = unsafe {
            CreateWindowExW(
                create_info.style_ex,
                class_name.as_pcswtr(),
                title.as_pcswtr(),
                create_info.style,
                create_info.x,
                create_info.y,
                create_info.width,
                create_info.height,
                None,
                None,
                event_loop.hinstance,
                Some(&mut create_data as *mut _ as *mut _),
            )
        }?;

        Ok(Self { hwnd })
    }

    fn register_class_unchecked(
        event_loop: &backend::EventLoop,
        class_name: &str,
    ) -> Result<WideStr, Error> {
        let class_name = WideStr::from_os_str(class_name);
        let style = CS_VREDRAW | CS_HREDRAW;

        let window_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: style,
            lpfnWndProc: Some(generic_window_callback),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: event_loop.hinstance,
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }?,
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
            hbrBackground: HBRUSH(unsafe { GetStockObject(BLACK_BRUSH) }.0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name.as_pcswtr(),
            hIconSm: unsafe { LoadIconW(None, IDI_APPLICATION) }?,
        };

        if unsafe { RegisterClassExW(&window_class) } == 0 {
            let e = unsafe { GetLastError() };
            if e != ERROR_CLASS_ALREADY_EXISTS {
                e.ok()?;
            }
        }

        Ok(class_name)
    }

    fn show_unchecked(hwnd: HWND, show_style: ShowStyle) {
        let show_cmd = match show_style {
            ShowStyle::Default => SW_SHOWDEFAULT,
            ShowStyle::Visible => SW_SHOW,
            ShowStyle::Hidden => SW_HIDE,
        };

        _ = unsafe { ShowWindow(hwnd, show_cmd) };
    }

    fn destroy_unchecked(hwnd: HWND) {
        _ = unsafe { DestroyWindow(hwnd) };
    }
}

#[derive(Debug, Default)]
pub(crate) struct BackendWindowAttribs {
    pub(crate) class_name: Option<String>,
}

pub trait WindowBuilderWindowsExt {
    fn with_class_name(self, class_name: String) -> Self;
}

impl WindowBuilderWindowsExt for Builder {
    fn with_class_name(mut self, class_name: String) -> Self {
        self.backend_attribs.class_name = Some(class_name);
        self
    }
}
