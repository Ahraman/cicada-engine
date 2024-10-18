use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND},
        Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, LoadCursorW, LoadIconW, RegisterClassExW, ShowWindow,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, IDI_APPLICATION, SW_SHOWDEFAULT,
            WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{
    error::WindowError,
    event::{WindowMove, WindowResize},
    event_loop::EventLoop,
    util::WideStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Windowed(Pos, Size),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowStyle {
    Default,
}

#[derive(Debug, Clone, Default)]
pub struct WindowAttribs {
    pub title: Option<String>,
    pub display_style: Option<DisplayStyle>,
    pub show_style: Option<ShowStyle>,
}

impl WindowAttribs {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_display_style(mut self, display_style: DisplayStyle) -> Self {
        self.display_style = Some(display_style);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct WindowPlatformSpecificAttribs {
    pub class_name: Option<String>,
}

pub(crate) struct CreateData<'a, T0, T1>
where
    T0: WindowResize,
    T1: WindowMove,
{
    pub(crate) event_loop: &'a mut EventLoop<T0, T1>,
}

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn new<'a, T0>(
        event_loop: &'a mut EventLoop<T0>,
        attribs: WindowAttribs,
        platform_specific: WindowPlatformSpecificAttribs,
    ) -> Result<Self, WindowError>
    where
        T0: WindowResize,
    {
        let create_data = CreateData { event_loop };

        unsafe { Self::create_unchecked(create_data, attribs, platform_specific) }
    }

    pub fn show(&mut self, show_style: ShowStyle) -> Result<(), WindowError> {
        unsafe { Self::show_unchecked(self.hwnd, show_style) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe { Self::destroy_unchecked(self.hwnd) }
    }
}

impl Window {
    unsafe fn create_unchecked<T0, T1>(
        mut create_data: CreateData<T0, T1>,
        attribs: WindowAttribs,
        platform_specific: WindowPlatformSpecificAttribs,
    ) -> Result<Self, WindowError>
    where
        T0: WindowResize,
        T1: WindowMove,
    {
        let hinstance = GetModuleHandleW(None)?.into();
        let class_name = Self::register_class_unchecked::<T0>(
            hinstance,
            platform_specific.class_name.as_deref(),
        )?;

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
            Some(&mut create_data as *mut _ as *mut _),
        )?;

        Ok(Self { hwnd })
    }

    unsafe fn register_class_unchecked<T0>(
        hinstance: HINSTANCE,
        class_name: Option<&str>,
    ) -> Result<WideStr, WindowError>
    where
        T0: WindowResize,
    {
        let class_style = CS_VREDRAW | CS_HREDRAW;

        let class_name = WideStr::new(class_name.unwrap_or("main_window"));

        let window_class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: class_style,
            lpfnWndProc: Some(EventLoop::<T0>::common_window_callback),
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

    unsafe fn show_unchecked(hwnd: HWND, show_style: ShowStyle) -> Result<(), WindowError> {
        let show_mode = match show_style {
            ShowStyle::Default => SW_SHOWDEFAULT,
        };

        Ok(ShowWindow(hwnd, show_mode).ok()?)
    }

    unsafe fn destroy_unchecked(hwnd: HWND) {
        let _ = DestroyWindow(hwnd);
    }
}
