use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND},
        Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH},
        UI::WindowsAndMessaging::{
            CreateWindowExW, LoadCursorW, LoadIconW, RegisterClassExW, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::window::{Attribs, Builder, DisplayStyle};

use super::{error::Error, event::EventLoop, util::WideStr};

pub(crate) struct Window {
    pub(super) hwnd: HWND,
}

impl Window {
    unsafe fn create_unchecked(
        event_loop: &EventLoop,
        attribs: &Attribs,
        backend_attribs: &BackendAttribs,
    ) -> Result<Self, Error> {
        let hinstance = event_loop.hinstance;
        let class_name =
            Self::register_class_unchecked(hinstance, backend_attribs.class_name.as_deref())?;

        struct CreateAttribs {
            x: i32,
            y: i32,
            width: i32,
            height: i32,

            style: WINDOW_STYLE,
            style_ex: WINDOW_EX_STYLE,
        }

        let create_attribs = match attribs.display_style {
            Some(DisplayStyle::Windowed(pos, size)) => CreateAttribs {
                x: pos.x,
                y: pos.y,
                width: size.width as i32,
                height: size.height as i32,
                style: WS_OVERLAPPEDWINDOW,
                style_ex: Default::default(),
            },
            None => CreateAttribs {
                x: CW_USEDEFAULT,
                y: CW_USEDEFAULT,
                width: CW_USEDEFAULT,
                height: CW_USEDEFAULT,
                style: WS_OVERLAPPEDWINDOW,
                style_ex: Default::default(),
            },
        };

        let window_title = WideStr::new(attribs.title.as_deref().unwrap_or("Window"));

        //let mut create_data = CreateData { event_loop };

        let hwnd = CreateWindowExW(
            create_attribs.style_ex,
            class_name.as_pcwstr(),
            window_title.as_pcwstr(),
            create_attribs.style,
            create_attribs.x,
            create_attribs.y,
            create_attribs.width,
            create_attribs.height,
            None,
            None,
            hinstance,
            None, //Some(&mut create_data as *mut _ as *mut _),
        )?;

        let mut window = Self { hwnd };
        match attribs.show_style {
            Some(show_style) => {} //window.show(show_style),
            None => {}
        };

        Ok(window)
    }

    unsafe fn register_class_unchecked(
        hinstance: HINSTANCE,
        class_name: Option<&str>,
    ) -> Result<WideStr, Error> {
        let class_style = CS_VREDRAW | CS_HREDRAW;

        let class_name = WideStr::new(class_name.unwrap_or("main_window"));

        let window_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: class_style,
            lpfnWndProc: None, //Some(common_window_callback),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: class_name.as_pcwstr(),
            hIconSm: LoadIconW(None, IDI_APPLICATION)?,
        };

        if RegisterClassExW(&window_class) == 0 {
            let err = GetLastError();
            if err != ERROR_CLASS_ALREADY_EXISTS {
                err.ok()?;
            }
        }

        Ok(class_name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BackendAttribs {
    pub class_name: Option<String>,
}

pub trait BackendBuilderExt {
    fn with_class_name(self, class_name: String) -> Self;
}

impl BackendBuilderExt for Builder {
    fn with_class_name(mut self, class_name: String) -> Self {
        self.backend_attribs.class_name = Some(class_name);
        self
    }
}
