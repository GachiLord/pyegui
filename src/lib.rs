#![allow(static_mut_refs)]

use chrono::NaiveDate;
use eframe::egui::{FontData, FontDefinitions, FontFamily};
use eframe::{self, egui};
use egui_extras;
use log::debug;
use pyo3::prelude::*;
use pyo3::{
    exceptions::{PyOSError, PyRuntimeError},
    types::{PyAny, PyBool, PyDict, PyInt, PyString},
};
use std::sync::{Arc, Mutex};
use std::{fs, ptr};
use std::time::Duration;

// state

static mut UI: *mut Vec<*mut egui::Ui> = ptr::null_mut();
static mut APP_MUTEX: Mutex<()> = Mutex::new(());

// messages

static APP_MUTEX_ERR: &'static str = "run_simple_native has been called on a separate thread";
static UI_PTR_NULL_ERR: &'static str = "UI ptr is null. This is likely to be a problem with pyegui";
static UI_STACK_ERR: &'static str = "UI stack is empty. This is likely to be a problem with pyegui";
static UI_CALL_OUTSIDE_UPDATE_FUNC: &'static str = "UI functions should be called only within update_fun and on the same thread. update_fun should only be called by run_simple_native";

// classes

#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
#[repr(i32)]
pub enum CursorIcon {
    Default,
    None,
    ContextMenu,
    Help,
    PointingHand,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    AllScroll,
    ResizeHorizontal,
    ResizeNeSw,
    ResizeNwSe,
    ResizeVertical,
    ResizeEast,
    ResizeSouthEast,
    ResizeSouth,
    ResizeSouthWest,
    ResizeWest,
    ResizeNorthWest,
    ResizeNorth,
    ResizeNorthEast,
    ResizeColumn,
    ResizeRow,
    ZoomIn,
    ZoomOut,
}

impl From<&CursorIcon> for egui::CursorIcon {
    fn from(cursor: &CursorIcon) -> Self {
        match cursor {
            CursorIcon::Default => Self::Default,
            CursorIcon::None => Self::None,
            CursorIcon::ContextMenu => Self::ContextMenu,
            CursorIcon::Help => Self::Help,
            CursorIcon::PointingHand => Self::PointingHand,
            CursorIcon::Progress => Self::Progress,
            CursorIcon::Wait => Self::Wait,
            CursorIcon::Cell => Self::Cell,
            CursorIcon::Crosshair => Self::Crosshair,
            CursorIcon::Text => Self::Text,
            CursorIcon::VerticalText => Self::VerticalText,
            CursorIcon::Alias => Self::Alias,
            CursorIcon::Copy => Self::Copy,
            CursorIcon::Move => Self::Move,
            CursorIcon::NoDrop => Self::NoDrop,
            CursorIcon::NotAllowed => Self::NotAllowed,
            CursorIcon::Grab => Self::Grab,
            CursorIcon::Grabbing => Self::Grabbing,
            CursorIcon::AllScroll => Self::AllScroll,
            CursorIcon::ResizeHorizontal => Self::ResizeHorizontal,
            CursorIcon::ResizeNeSw => Self::ResizeNeSw,
            CursorIcon::ResizeNwSe => Self::ResizeNwSe,
            CursorIcon::ResizeVertical => Self::ResizeVertical,
            CursorIcon::ResizeEast => Self::ResizeEast,
            CursorIcon::ResizeSouthEast => Self::ResizeSouthEast,
            CursorIcon::ResizeSouth => Self::ResizeSouth,
            CursorIcon::ResizeSouthWest => Self::ResizeSouthWest,
            CursorIcon::ResizeWest => Self::ResizeWest,
            CursorIcon::ResizeNorthWest => Self::ResizeNorthWest,
            CursorIcon::ResizeNorth => Self::ResizeNorth,
            CursorIcon::ResizeNorthEast => Self::ResizeNorthEast,
            CursorIcon::ResizeColumn => Self::ResizeColumn,
            CursorIcon::ResizeRow => Self::ResizeRow,
            CursorIcon::ZoomIn => Self::ZoomIn,
            CursorIcon::ZoomOut => Self::ZoomOut,
        }
    }
}

#[pyclass]
pub struct Pos2 {
    /// x() -> float
    #[pyo3(get, set)]
    x: f32,
    /// y() -> float
    #[pyo3(get, set)]
    y: f32,
}

/// Context object that controls global aspects of your app
///
/// Usage::
///
///     def update_func(ctx):
///         ctx.set_light_theme
///         heading("Using light theme even if system's is dark")
///
///     run_native("My app", update_func)
#[pyclass]
struct Context(egui::Context);

#[pymethods]
impl Context {
    /// is_light_theme() -> bool
    ///
    /// True when theme is dark
    #[getter]
    fn is_light_theme(&self) -> bool {
        self.0.theme() == egui::Theme::Light
    }

    /// is_dark_theme() -> bool
    ///
    /// True when theme is dark
    #[getter]
    fn is_dark_theme(&self) -> bool {
        self.0.theme() == egui::Theme::Dark
    }

    /// set_light_theme() -> None
    ///
    /// Sets light theme. Default is system's
    #[pyo3(text_signature = "($self)")]
    fn set_light_theme(&self) {
        self.0.set_theme(egui::ThemePreference::Light);
    }

    /// set_dark_theme() -> None
    ///
    /// Sets dark theme. Default is system's
    #[pyo3(text_signature = "($self)")]
    fn set_dark_theme(&self) {
        self.0.set_theme(egui::ThemePreference::Dark);
    }

    /// set_system_theme() -> None
    ///
    /// Sets system's theme if it has been changed.
    #[pyo3(text_signature = "($self)")]
    fn set_system_theme(&self) {
        self.0.set_theme(egui::ThemePreference::System);
    }

    /// set_font(source: str) -> None
    ///
    /// Tell egui which fonts to use.
    ///
    /// The default egui fonts only support latin and cyrillic alphabets, but you can call this to install additional fonts that support e.g. Japanese characters.
    ///
    /// The new fonts will become active at the start of the next pass. This will overwrite the existing fonts.
    ///  
    /// Example::
    ///
    ///   def update_func(ctx):
    ///     ctx.set_font("NotoSansJP-VariableFont_wght.ttf")
    ///     heading("天気の子")
    #[pyo3(signature = (source: "str") -> "None")]
    fn set_font(&self, source: String) -> PyResult<()> {
        let buf = fs::read(&source).map_err(|e| {
            eprintln!("Cannot open '{}': {}", source, e.to_string());
            e
        })?;

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            source.clone(),
            Arc::new(
                // .ttf and .otf supported
                FontData::from_owned(buf),
            ),
        );
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push(source.clone());

        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, source);

        self.0.set_fonts(fonts);

        Ok(())
    }

    /// open_url(url: str) -> None
    ///
    /// Open an URL in a browser.
    #[pyo3(text_signature = "($self, url)")]
    fn open_url(&self, url: &str) {
        self.0.open_url(egui::OpenUrl::new_tab(url));
    }

    /// copy_text(text: str) -> None
    ///
    /// Copy the given text to the system clipboard.
    #[pyo3(text_signature = "($self, text)")]
    fn copy_text(&self, text: String) {
        self.0.copy_text(text);
    }

    /// request_repaint() -> None
    ///
    /// Call this if there is need to repaint the UI, i.e. if you are showing an animation.
    /// If this is called at least once in a frame, then there will be another frame right after this. Call as many times as you wish, only one repaint will be issued.
    #[pyo3(text_signature = "($self)")]
    fn request_repaint(&self) {
        self.0.request_repaint();
    }

    /// request_repaint_after(duration: datetime.timedelta) -> None
    ///
    /// Request repaint after at most the specified duration elapses.
    /// 
    /// The backend can chose to repaint sooner, for instance if some other code called this method with a lower duration, or if new events arrived.
    /// 
    /// The function can be multiple times, but only the smallest duration will be considered. So, if the function is called two times with 1 second and 2 seconds, egui will repaint after 1 second
    /// 
    /// This is primarily useful for applications who would like to save battery by avoiding wasted redraws when the app is not in focus. But sometimes the GUI of the app might become stale and outdated if it is not updated for too long.
    /// 
    /// Let’s say, something like a stopwatch widget that displays the time in seconds. You would waste resources repainting multiple times within the same second (when you have no input), just calculate the difference of duration between current time and next second change, and call this function, to make sure that you are displaying the latest updated time, but not wasting resources on needless repaints within the same second.
    /// 
    /// Quirk:
    /// Duration begins at the next frame. Let’s say for example that it’s a very inefficient app and takes 500 milliseconds per frame at 2 fps. The widget / user might want a repaint in next 500 milliseconds. Now, app takes 1000 ms per frame (1 fps) because the backend event timeout takes 500 milliseconds AFTER the vsync swap buffer. So, it’s not that we are requesting repaint within X duration. We are rather timing out during app idle time where we are not receiving any new input events.
    /// 
    /// This repaints the current viewport.
    #[pyo3(text_signature = "($self, duration)")]
    fn request_repaint_after(&self, duration: Duration) {
        self.0.request_repaint_after(duration);
    }

    /// set_cursor_icon(cursor_icon: int) -> None
    ///
    /// Set the cursor icon.
    /// Possible values:
    ///
    /// Default
    ///
    /// None
    ///
    /// ContextMenu
    ///
    /// Help
    ///
    /// PointingHand
    ///
    /// Progress
    ///
    /// Wait
    ///
    /// Cell
    ///
    /// Crosshair
    ///
    /// Text
    ///
    /// VerticalText
    ///
    /// Alias
    ///
    /// Copy
    ///
    /// Move
    ///
    /// NoDrop
    ///
    /// NotAllowed
    ///
    /// Grab
    ///
    /// Grabbing
    ///
    /// AllScroll
    ///
    /// ResizeHorizontal
    ///
    /// ResizeNeSw
    ///
    /// ResizeNwSe
    ///
    /// ResizeVertical
    ///
    /// ResizeEast
    ///
    /// ResizeSouthEast
    ///
    /// ResizeSouth
    ///
    /// ResizeSouthWest
    ///
    /// ResizeWest
    ///
    /// ResizeNorthWest
    ///
    /// ResizeNorth
    ///
    /// ResizeNorthEast
    ///
    /// ResizeColumn
    ///
    /// ResizeRow
    ///
    /// ZoomIn
    ///
    /// ZoomOut
    #[pyo3(text_signature = "($self, cursor_icon)")]
    fn set_cursor_icon(&self, cursor_icon: &CursorIcon) {
        self.0
            .set_cursor_icon(egui::CursorIcon::from(cursor_icon))
    }

    /// pixels_per_point() -> float
    ///
    /// The number of physical pixels for each logical point.
    /// 
    /// This is calculated as zoom_factor * native_pixels_per_point
    #[getter]
    fn pixels_per_point(&self) -> f32 {
        self.0.pixels_per_point()
    }

    /// set_pixels_per_point(pixels_per_point: float) -> None
    ///
    /// Set the number of physical pixels for each logical point. Will become active at the start of the next pass.
    /// 
    /// This will actually translate to a call to set_zoom_factor.
    #[pyo3(text_signature = "($self, pixels_per_point)")]
    fn set_pixels_per_point(&self, pixels_per_point: f32) {
        self.0.set_pixels_per_point(pixels_per_point)
    }

    /// native_pixels_per_point() -> float | None
    ///
    /// The number of physical pixels for each logical point on this monitor.
    #[getter]
    fn native_pixels_per_point(&self) -> Option<f32> {
        self.0.native_pixels_per_point()
    }

    /// zoom_factor() -> float
    ///
    /// Global zoom factor of the UI.
    /// 
    /// This is used to calculate the pixels_per_point for the UI as pixels_per_point = zoom_factor * native_pixels_per_point.
    /// 
    /// The default is 1.0. Make larger to make everything larger.
    #[getter]
    fn zoom_factor(&self) -> f32 {
        self.0.zoom_factor()
    }

    /// set_zoom_factor(zoom_factor: float) -> None
    ///
    /// Sets zoom factor of the UI. Will become active at the start of the next pass.
    /// 
    /// Note that calling this will not update zoom_factor until the end of the pass.
    /// 
    /// This is used to calculate the pixels_per_point for the UI as pixels_per_point = zoom_fator * native_pixels_per_point.
    /// 
    /// The default is 1.0. Make larger to make everything larger.
    fn set_zoom_factor(&self, zoom_factor: f32) {
        self.0.set_zoom_factor(zoom_factor)
    }

    /// is_pointer_over_area() -> bool
    ///
    /// Is the pointer (mouse/touch) over any egui area?
    #[getter]
    fn is_pointer_over_area(&self) -> bool {
        self.0.is_pointer_over_area()
    }

    /// wants_pointer_input() -> bool
    ///
    /// True if egui is currently interested in the pointer (mouse or touch).
    /// 
    /// Could be the pointer is hovering over a crate::Window or the user is dragging a widget. If false, the pointer is outside of any egui area and so you may be interested in what it is doing (e.g. controlling your game). Returns false if a drag started outside of egui and then moved over an egui area.
    #[getter]
    fn wants_pointer_input(&self) -> bool {
        self.0.wants_pointer_input()
    }

    /// is_using_pointer() -> bool
    ///
    /// Is egui currently using the pointer position (e.g. dragging a slider)?
    /// 
    /// NOTE: this will return false if the pointer is just hovering over an egui area.
    #[getter]
    fn is_using_pointer(&self) -> bool {
        self.0.is_using_pointer()
    }

    /// wants_keyboard_input() -> bool
    ///
    /// If true, egui is currently listening on text input.
    #[getter]
    fn wants_keyboard_input(&self) -> bool {
        self.0.wants_keyboard_input()
    }

    /// is_context_menu_open() -> bool
    ///
    /// Is an egui context menu open?
    #[getter]
    fn is_context_menu_open(&self) -> bool {
        self.0.is_context_menu_open()
    }

    /// pointer_latest_pos() -> Pos2 | None
    ///
    /// Latest reported pointer position.
    /// 
    /// When tapping a touch screen, this will be None.
    ///
    /// Usage::
    ///     
    ///     pos = ctx.pointer_latest_pos()
    ///     print(pos.x, pos.y)
    #[getter]
    fn pointer_latest_pos(&self) -> Option<Pos2> {
        let p = self.0.pointer_latest_pos();

        Some(Pos2{
            x: p?.x,
            y: p?.y,
        })
    }


    /// pointer_hover_pos() -> Pos2 | None
    ///
    /// If it is a good idea to show a tooltip, where is pointer?
    ///
    /// Usage::
    ///     
    ///     pos = ctx.pointer_hover_pos()
    ///     print(pos.x, pos.y)
    #[getter]
    fn pointer_hover_pos(&self) -> Option<Pos2> {
        let p = self.0.pointer_latest_pos();

        Some(Pos2{
            x: p?.x,
            y: p?.y,
        })
    }

    /// pointer_interact_pos() -> Pos2 | None
    ///
    /// If you detect a click or drag and wants to know where it happened, use this.
    /// 
    /// Latest position of the mouse, but ignoring any crate::Event::PointerGone if there were interactions this pass. When tapping a touch screen, this will be the location of the touch.
    ///
    /// Usage::
    ///     
    ///     pos = ctx.pointer_interact_pos()
    ///     print(pos.x, pos.y)
    #[getter]
    fn pointer_interact_pos(&self) -> Option<Pos2> {
        let p = self.0.pointer_interact_pos();

        Some(Pos2{
            x: p?.x,
            y: p?.y,
        })
    }
}

/// __init__(value: str) -> Str
/// Str stores string value that can be referenced
///
/// Usage::
///
///     data = Str("")
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Add :)"):
///             data.value += ":) "
#[pyclass]
struct Str {
    /// value() -> str
    #[pyo3(get, set)]
    value: String,
}

#[pymethods]
impl Str {

    #[new]
    fn new(value: String) -> Self {
        Str { value }
    }
}

/// __init__(value: bool) -> Bool
/// Bool stores a boolean value that can be referenced
///
/// Usage::
///
///     data = Bool(False)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         # button will be shown only if the checkbox is checked
///         if data.value and button_clicked("set to False"):
///             # hiding the button
///             data.value = False
///         checkbox(data, "Check me")
#[pyclass]
struct Bool {
    /// value() -> bool
    #[pyo3(get, set)]
    value: bool,
}

#[pymethods]
impl Bool {
    #[new]
    fn new(value: bool) -> Self {
        Bool { value }
    }
}

/// __init__() -> int
/// Int stores integer value that can be referenced
///
/// Usage::
///
///     data = Int(69)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Increment"):
///             data.value += 1
#[pyclass]
struct Int {
    /// value() -> int
    #[pyo3(get, set)]
    value: i32,
}

#[pymethods]
impl Int {
    #[new]
    fn new(value: i32) -> Self {
        Int { value }
    }
}

/// __init__() -> Float
/// Float stores float value that can be referenced
///
/// Usage::
///
///     data = Float(69.0)
///     
///     def update_func():
///         heading(f"Value of the data is {data.value}")
///         if button_clicked("Increment"):
///             # what can go wrong?
///             data.value += 0.1
#[pyclass]
struct Float {
    /// value() -> float
    #[pyo3(get, set)]
    value: f32,
}

#[pymethods]
impl Float {
    #[new]
    fn new(value: f32) -> Self {
        Float { value }
    }
}

/// __init__(r: float, g: float, b: float) -> RGB
/// Rgb color picker
///
/// Usage::
///
///     color_rgb = RGB(69, 69, 69)
///     color_edit_button_rgb(color_rgb)
#[pyclass]
struct RGB {
    /// r() -> float
    #[pyo3(get, set)]
    r: f32,
    /// g() -> float
    #[pyo3(get, set)]
    g: f32,
    #[pyo3(get, set)]
    /// b() -> float
    b: f32,
}

#[pymethods]
impl RGB {
    #[new]
    fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
}

/// __init__() -> datetime
/// Date picker
///
/// Usage::
///
///     date = Date(datetime.datetime.now())
///     date_picker_button(date)
#[pyclass]
struct Date {
    /// value() -> datetime
    #[pyo3(get, set)]
    value: NaiveDate,
}

#[pymethods]
impl Date {
    #[new]
    fn new(value: NaiveDate) -> Self {
        Date { value }
    }
}

/// Values of this enum are used in Layout
#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
#[repr(i32)]
pub enum LayoutType {
    Horizontal,
    HorizontalCentered,
    HorizontalTop,
    HorizontalWrapped,
    Vertical,
    VerticalCentered,
    VerticalCenteredJustified,
    CenteredAndJustified,
}

/// __init__(layout: LayoutType) -> Layout
/// Layout class can be used to specify layout of widgets that go after with statement.
///
/// Usage::
///
///     with Layout(LayoutType.VerticalCentered):
///        heading("This widget will be horizontally centered")
///
/// Layout types:
///
/// **Horizontal**
///
/// Start a ui with horizontal layout.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// If you don’t want the contents to be centered, use HorizontalTop instead.
///
/// **HorizontalCentered**
///
/// Like Horizontal, but allocates the full vertical height and then centers elements vertically.
///
/// **HorizontalTop**
///
/// Like Horizontal, but aligns content with top.
///
/// **HorizontalWrapped**
///
/// Start a ui with horizontal layout that wraps to a new row when it reaches the right edge.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// **Vertical**
///
/// Start a ui with vertical layout. Widgets will be left-justified.
///
/// **VerticalCentered**
///
/// Start a ui with vertical layout. Widgets will be horizontally centered.
///
/// **VerticalCenteredJustified**
///
/// Start a ui with vertical layout. Widgets will be horizontally centered and justified (fill full width).
///
/// **CenteredAndJustified**
///
/// This will make the next added widget centered and justified in the available space.
///
/// Only one child widget is allowed!
#[pyclass]
struct Layout {
    ui: egui::Ui,
}

#[pymethods]
impl Layout {
    #[new]
    fn __new__(layout_type: &LayoutType) -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;

            // build layout based on scope type
            let layout = match layout_type {
                LayoutType::HorizontalCentered => egui::Layout::left_to_right(egui::Align::Center)
                    .with_cross_align(egui::Align::Center),
                LayoutType::Horizontal => egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_wrap(false)
                    .with_cross_align(egui::Align::Min),
                LayoutType::HorizontalTop => egui::Layout::left_to_right(egui::Align::Center)
                    .with_cross_align(egui::Align::Min),
                LayoutType::HorizontalWrapped => egui::Layout::left_to_right(egui::Align::Center)
                    .with_main_wrap(false)
                    .with_cross_align(egui::Align::Min),
                LayoutType::Vertical => egui::Layout::top_down(egui::Align::Min),
                LayoutType::VerticalCentered => egui::Layout::top_down(egui::Align::Center),
                LayoutType::VerticalCenteredJustified => {
                    egui::Layout::top_down(egui::Align::Center).with_cross_justify(true)
                }
                LayoutType::CenteredAndJustified => {
                    egui::Layout::centered_and_justified(egui::Direction::TopDown)
                }
            };

            let ui = parent_ui.new_child(egui::UiBuilder::new().layout(layout));

            Ok(Self { ui })
        }
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let child_ui = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            let parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

/// __init__() -> Scope
/// Create a scope for the contents.
/// You can use this to temporarily change the Style of a sub-region
///
/// Usage::
///
///     with Scope():
///        set_opacity(0.5)
///        heading("hi")
///        heading("there")
///
///     heading("normal opacity")
#[pyclass]
struct Scope {
    ui: egui::Ui,
}

#[pymethods]
impl Scope {
    #[new]
    fn __new__() -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;
            let ui = parent_ui.new_child(egui::UiBuilder::new());
            Ok(Self { ui })
        }
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let child_ui = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            let parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

/// __init__() -> Group
/// Visually groups the contents together.
///
/// Usage::
///
///     with Group():
///        heading("hi")
///        heading("there")
#[pyclass]
struct Group {
    prepared: egui::frame::Prepared,
}

#[pymethods]
impl Group {
    #[new]
    fn __new__() -> PyResult<Self> {
        unsafe {
            let parent_ui = current_ui(&UI)?;

            let frame = egui::Frame::group(&parent_ui.style());
            let prepared = frame.begin(parent_ui);

            Ok(Self { prepared })
        }
    }

    fn __enter__(&mut self) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            ui_stack.push(&raw mut self.prepared.content_ui);
            Ok(())
        }
    }

    fn __exit__(
        &self,
        _exception_type: Bound<'_, PyAny>,
        _exception_value: Bound<'_, PyAny>,
        _exception_traceback: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        unsafe {
            let ui_stack = ui_stack(&UI)?;

            let _ = ui_stack
                .pop()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?;

            let mut parent_ui = ui_stack
                .last()
                .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))?
                .as_mut()
                .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))?;

            self.prepared.paint(&mut parent_ui);
            self.prepared.allocate_space(&mut parent_ui);
            // parent_ui.advance_cursor_after_rect(child_ui.min_rect());

            Ok(())
        }
    }
}

// Start function

struct PyeguiApp<'py> {
    update_func: Bound<'py, PyAny>,
}

impl eframe::App for PyeguiApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let ctx_r = Context(ctx.clone());

        unsafe {
            egui::CentralPanel::default().show(ctx, |ui| {
                debug!("Getting ui stack pointer");
                let ui_stack = UI.as_mut().expect(UI_PTR_NULL_ERR);

                debug!("Push UI");
                ui_stack.push(&raw mut *ui);

                debug!("Execute update_func");

                Python::attach(|py| {
                    if let Err(err) = self.update_func.call1((ctx_r,)) {
                        err.display(py);
                    }
                });

                debug!("Executed update_func");

                ui_stack.pop().expect(UI_STACK_ERR);

                debug!("Pop UI");
            });
        }
    }
}

/// run_native(app_name: str, update_func: Callable[[Context], None], **kwargs) -> None
///
/// Creates a window and runs update_func.
/// This is an entrypoint for your GUI application.
///
/// Args:
///     app_name (str): name displayed at the header bar
///
///     update_func (Callable[[Context], None]): your function that draws UI
///
///     inner_height (float): the desired height of the window
///
///     inner_width (float): the desired width of the window
///
///     min_inner_height (float): min height of the window
///
///     min_inner_width (float): min width of the window
///
///     max_inner_height (float): max height of the window
///
///     max_inner_width (float): max width of the window
///
///     fullscreen (bool): whether to open app in fullscreen
///
///     maximized (bool): whether to open app maximized
///
///     resizable (bool): whether our app is resizable
///
///     transparent (bool): whether our app is transparent
///
///     icon_path (str): path to icon in rgba format
///
/// Examples::
///
///     name = Str("")
///     
///     def update_func(ctx):
///         heading(f"Hello, {name.value}!")
///         text_edit_singleline(name)
///         
///         if button_clicked("click me"):
///             print("clicked")
///     
///     run_native("My app", update_func)
///
#[pyfunction]
#[pyo3(signature = (app_name: "str", update_func: "Callable[[Context], None]", **kwargs))]
unsafe fn run_native(
    app_name: &str,
    update_func: Bound<'_, PyAny>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<()> {
    pyo3_log::init();
    debug!("Trying to get the app lock");
    // ensure thread safety
    let _lock = APP_MUTEX
        .try_lock()
        .map_err(|_| PyRuntimeError::new_err(APP_MUTEX_ERR))?;
    // init UI stack
    debug!("Initialzing UI stack");
    let mut ui_stack = Vec::with_capacity(32);
    UI = &raw mut *&mut ui_stack;
    // parse kwargs
    let mut viewport = egui::viewport::ViewportBuilder::default();

    if let Some(kwargs) = kwargs {
        if let (Some(height), Some(width)) = (
            kwargs.get_item("inner_height")?,
            kwargs.get_item("inner_width")?,
        ) {
            viewport = viewport.with_inner_size([
                width.cast::<PyInt>()?.extract()?,
                height.cast::<PyInt>()?.extract()?,
            ]);
        }

        if let (Some(height), Some(width)) = (
            kwargs.get_item("min_inner_height")?,
            kwargs.get_item("min_inner_width")?,
        ) {
            viewport = viewport.with_min_inner_size([
                width.cast::<PyInt>()?.extract()?,
                height.cast::<PyInt>()?.extract()?,
            ]);
        }

        if let (Some(height), Some(width)) = (
            kwargs.get_item("max_inner_height")?,
            kwargs.get_item("max_inner_width")?,
        ) {
            viewport = viewport.with_max_inner_size([
                width.cast::<PyInt>()?.extract()?,
                height.cast::<PyInt>()?.extract()?,
            ]);
        }

        if let Some(fullscreen) = kwargs.get_item("fullscreen")? {
            viewport = viewport.with_fullscreen(fullscreen.cast::<PyBool>()?.extract()?);
        }

        if let Some(maximized) = kwargs.get_item("maximized")? {
            viewport = viewport.with_maximized(maximized.cast::<PyBool>()?.extract()?);
        }

        if let Some(resizable) = kwargs.get_item("resizable")? {
            viewport = viewport.with_resizable(resizable.cast::<PyBool>()?.extract()?);
        }

        if let Some(transparent) = kwargs.get_item("transparent")? {
            viewport = viewport.with_transparent(transparent.cast::<PyBool>()?.extract()?);
        }

        if let Some(icon_path) = kwargs.get_item("icon_path")? {
            let path = icon_path.cast::<PyString>()?.extract::<String>()?;
            let buf = fs::read(path)?;

            let icon_data = eframe::icon_data::from_png_bytes(&buf)
                .map_err(|e| PyOSError::new_err(format!("Failed to decode png file: {}", e)))?;
            viewport = viewport.with_icon(icon_data);
        }
    }

    let options = eframe::NativeOptions {
        viewport,
        ..eframe::NativeOptions::default()
    };
    debug!("Creating a window");
    // create a window
    let result = eframe::run_native(
        app_name,
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(PyeguiApp {
                update_func: update_func,
            }))
        }),
    );

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(PyRuntimeError::new_err(format!(
            "Cannot create a window: {}",
            err.to_string()
        ))),
    }
}

// helpers

unsafe fn ui_stack(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut Vec<*mut egui::Ui>> {
    ui.as_mut()
        .ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))
}

unsafe fn last_ui(ui_stack: &mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
    let last_ui = ui_stack
        .last_mut()
        .ok_or(PyRuntimeError::new_err(UI_STACK_ERR))?;

    last_ui
        .as_mut()
        .ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))
}

unsafe fn current_ui(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
    last_ui(ui_stack(ui)?)
}

unsafe fn run_nested_update_func(ui: &mut egui::Ui, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

    ui_stack.push(&raw mut *ui);

    if let Err(err) = update_fun.call0() {
        Python::attach(|py| {
            err.display(py);
        });
    }

    match ui_stack.pop() {
        Some(_) => Ok(()),
        None => Err(PyRuntimeError::new_err(UI_STACK_ERR)),
    }
}

// UI functions

/// heading(text: str) -> None
///
/// Show large text
///
/// Example::
///
///     heading("hello")
#[pyfunction]
unsafe fn heading(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.heading(text);
    Ok(())
}

/// monospace(text: str) -> None
///
/// Show monospace (fixed width) text.
///
/// Example::
///
///     monospace("hello")
#[pyfunction]
unsafe fn monospace(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.monospace(text);
    Ok(())
}

/// small(text: str) -> None
///
/// Show small text.
///
/// Example::
///
///     small("hello")
#[pyfunction]
unsafe fn small(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.small(text);
    Ok(())
}

/// strong(text: str) -> None
///
/// Show text that stand out a bit (e.g. slightly brighter).
///
/// Example::
///
///     strong("hello")
#[pyfunction]
unsafe fn strong(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.strong(text);
    Ok(())
}

/// weak(text: str) -> None
///
/// Show text that is weaker (fainter color).
///
/// Example::
///
///     weak("hello")
#[pyfunction]
unsafe fn weak(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.weak(text);
    Ok(())
}

/// label(text: str) -> None
///
/// Show some text.
///
/// Example::
///
///     label("some text")
#[pyfunction]
unsafe fn label(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.label(text);
    Ok(())
}

/// code(text: str) -> None
///
/// Show text as monospace with a gray background.
///
/// Example::
///
///     code("print(42 + 27)")
#[pyfunction]
unsafe fn code(text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.code(text);
    Ok(())
}

/// code_editor(text: Str) -> None
///
/// Show singleline text field and update the text
///
/// Example::
///
///     text = Str("print(42 + 27)")
///     # inside update func
///     code_editor(text)
#[pyfunction]
unsafe fn code_editor(text: &mut Str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.code_editor(&mut text.value);
    Ok(())
}

/// text_edit_singleline(text: Str, **kwargs) -> None
///
/// Show singleline text field and update the text
///
/// Example::
///
///     text = Str("editable")
///     # inside update func
///     text_edit_singleline(text, hint_text="hint me bro")
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_singleline(text: &mut Str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    let mut w = egui::TextEdit::singleline(&mut text.value);

    if let Some(kwargs) = kwargs {
        if let Some(hint_text) = kwargs.get_item("hint_text")? {
            w = w.hint_text(hint_text.cast::<PyString>()?.extract::<String>()?);
        }
    }

    ui.add(w);
    Ok(())
}

/// text_edit_multiline(text: Str, **kwargs) -> None
///
/// Show multiline text field and update the text
///
/// Example::
///
///     text = Str("editable")
///     # inside update func
///     text_edit_multiline(text, hint_text="hint")
#[pyfunction]
#[pyo3(signature = (text, **kwargs))]
unsafe fn text_edit_multiline(text: &mut Str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    let mut w = egui::TextEdit::multiline(&mut text.value);

    if let Some(kwargs) = kwargs {
        if let Some(hint_text) = kwargs.get_item("hint_text")? {
            w = w.hint_text(hint_text.cast::<PyString>()?.extract::<String>()?);
        }
    }

    ui.add(w);
    Ok(())
}

/// button_clicked(text: str) -> bool
///
/// Returns true if the button was clicked this frame
///
/// Example::
///
///     if button_clicked("click me"):
///       print("click me, my friend")
#[pyfunction]
unsafe fn button_clicked(text: &str) -> PyResult<bool> {
    let ui = current_ui(&UI)?;

    Ok(ui.button(text).clicked())
}

/// small_button_clicked(text: str) -> bool
///
/// Returns true if the small button was clicked this frame
///
/// Example::
///
///     if small_button_clicked("click me"):
///       print("click me, my friend")
#[pyfunction]
unsafe fn small_button_clicked(text: &str) -> PyResult<bool> {
    let ui = current_ui(&UI)?;

    Ok(ui.small_button(text).clicked())
}

/// horizontal(update_fun: Callable[[Context], None]) -> None
///
/// Start a ui with horizontal layout. After you have called this, the function registers the contents as any other widget.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
///
/// If you don’t want the contents to be centered, use horizontal_top instead.
///
/// Example::
///
///     def horizontal_update_func():
///       heading("I'm horizontal")
///     
///     horizontal(horizontal_update_func)
#[pyfunction]
unsafe fn horizontal(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// horizontal_centered(update_fun: Callable[[Context], None]) -> None
///
/// Like horizontal, but allocates the full vertical height and then centers elements vertically.
#[pyfunction]
unsafe fn horizontal_centered(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_centered(|ui| run_nested_update_func(ui, update_fun))
        .inner
}
/// horizontal_top(update_fun: Callable[[Context], None]) -> None
///
/// Like horizontal, but aligns content with top.
#[pyfunction]
unsafe fn horizontal_top(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_top(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// horizontal_wrapped(update_fun: Callable[[Context], None]) -> None
///
/// Start a ui with horizontal layout that wraps to a new row when it reaches the right edge of the max_size. After you have called this, the function registers the contents as any other widget.
///
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
#[pyfunction]
unsafe fn horizontal_wrapped(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .horizontal_wrapped(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// vertical(update_fun: Callable[[Context], None]) -> None
///
/// Start a ui with vertical layout. Widgets will be left-justified.
#[pyfunction]
unsafe fn vertical(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// vertical_centered(update_fun: Callable[[Context], None]) -> None
///
/// Start a ui with vertical layout. Widgets will be horizontally centered.
#[pyfunction]
unsafe fn vertical_centered(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical_centered(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// vertical_centered_justified(update_fun: Callable[[Context], None]) -> None
///
/// Start a ui with vertical layout. Widgets will be horizontally centered and justified (fill full width).
#[pyfunction]
unsafe fn vertical_centered_justified(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .vertical_centered_justified(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// centered_and_justified(update_fun: Callable[[Context], None]) -> None
///
/// This will make the next added widget centered and justified in the available space.
///
/// Only one widget may be added inside update_func!
#[pyfunction]
unsafe fn centered_and_justified(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .centered_and_justified(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// collapsing(heading: str, update_fun: Callable[[Context], None]) -> None
///
/// A CollapsingHeader that starts out collapsed.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///     collapsing("collapsed", update_func)
#[pyfunction]
unsafe fn collapsing(heading: &str, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?.collapsing(heading, |ui| run_nested_update_func(ui, update_fun));
    Ok(())
}

/// indent(update_fun: Callable[[Context], None]) -> None
///
/// Create a child ui which is indented to the right.
/// Example::
///
///     def update_func():
///       heading("I'm indented")
///     indent(update_func)
#[pyfunction]
unsafe fn indent(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .indent("your mom", |ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// group(update_fun: Callable[[Context], None]) -> None
///
/// Visually groups the contents together.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///     
///     group(update_func)
#[pyfunction]
unsafe fn group(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .group(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// scroll_area_vertical(update_fun: Callable[[Context], None]) -> None
///
/// Create a vertical scroll area.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///       # a lot of elements
///     
///     scroll_area_vertical(update_func)
#[pyfunction]
unsafe fn scroll_area_vertical(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    egui::ScrollArea::vertical().show(current_ui(&UI)?, |ui| run_nested_update_func(ui, update_fun)).inner
}


/// scroll_area_horizontal(update_fun: Callable[[Context], None]) -> None
///
/// Create a horizontal scroll area.
///
/// Example::
///
///     def update_func():
///       heading("hi")
///       heading("there")
///       # a lot of elements
///     
///     scroll_area_horizontal(update_func)
#[pyfunction]
unsafe fn scroll_area_horizontal(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    egui::ScrollArea::horizontal().show(current_ui(&UI)?, |ui| run_nested_update_func(ui, update_fun)).inner
}

/// scope(update_fun: Callable[[Context], None]) -> None
///
/// Create a scoped child ui.
///
/// You can use this to temporarily change the Style of a sub-region.
///
/// Example::
///
///     def update_func():
///       heading("0.5 opacity")
///       set_opacity(0.5)
///     
///     heading("normal opacity")
///     scope(update_func)
#[pyfunction]
unsafe fn scope(update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .scope(|ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// slider_float(value: Float, min: float, max: float, text: str) -> None
///
/// Control float with a slider.
///
/// Example::
///
///     data = Float(5)
///     # inside update_func
///     slider_float(data, 0, 50, "slide me")
#[pyfunction]
unsafe fn slider_float(value: &mut Float, min: f32, max: f32, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(egui::Slider::new(&mut value.value, min..=max).text(text));
    Ok(())
}

/// slider_int(value: Int, min: int, max: int, text: str) -> None
///
/// Control int with a slider.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     slider_int(data, 0, 50, "slide me")
#[pyfunction]
unsafe fn slider_int(value: &mut Int, min: i32, max: i32, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(
        egui::Slider::new(&mut value.value, min..=max)
            .text(text)
            .integer(),
    );
    Ok(())
}

/// drag_float(value: Float, min: float, max: float, speed: float) -> None
///
/// Control float by dragging the number.
///
/// Example::
///
///     data = Float(5)
///     # inside update_func
///     drag_float(data, 0, 50, 1.5)
#[pyfunction]
unsafe fn drag_float(value: &mut Float, min: f32, max: f32, speed: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(
        egui::DragValue::new(&mut value.value)
            .speed(speed)
            .range(min..=max),
    );
    Ok(())
}

/// drag_int(value: Int, min: int, max: int, speed: int) -> None
///
/// Control int by dragging the number.
///
/// Example::
///
///     data = Int(5)
///     # inside update_func
///     drag_int(data, 0, 50, 1)
#[pyfunction]
unsafe fn drag_int(value: &mut Int, min: i32, max: i32, speed: i32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(
        egui::DragValue::new(&mut value.value)
            .speed(speed)
            .range(min..=max),
    );
    Ok(())
}

/// hyperlink(url: str) -> None
///
/// A clickable hyperlink
///
/// Example::
///
///     hyperlink("https://github.com/emilk/egui")
#[pyfunction]
unsafe fn hyperlink(url: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.hyperlink(url);
    Ok(())
}

/// hyperlink_to(label: str, url: str) -> None
///
/// A clickable hyperlink with label
///
/// Example::
///
///     hyperlink_to("egui on GitHub", "https://www.github.com/emilk/egui/")
#[pyfunction]
unsafe fn hyperlink_to(label: &str, url: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.hyperlink_to(label, url);
    Ok(())
}

/// link_clicked(label: str) -> bool
///
/// Clickable text, that looks like a hyperlink.
/// To link to a web page, use hyperlink or hyperlink_to.
///
/// Example::
///
///     if link_clicked("egui on GitHub"):
///       print("clicked on a fake link")
#[pyfunction]
unsafe fn link_clicked(label: &str) -> PyResult<bool> {
    let ui = current_ui(&UI)?;

    Ok(ui.link(label).clicked())
}

/// checkbox(checked: Bool, text: str) -> None
///
/// Show a checkbox.
///
/// Example::
///
///     data = Bool(false)
///     # inside update_func
///     checkbox(data, "check me")
#[pyfunction]
unsafe fn checkbox(checked: &mut Bool, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.checkbox(&mut checked.value, text);
    Ok(())
}

/// toggle_value(selected: Bool, text: str) -> None
///
/// Acts like a checkbox, but looks like a selectable label.
///
/// Example::
///
///     data = Bool(false)
///     # inside update_func
///     toggle_value(data, "check me")
#[pyfunction]
unsafe fn toggle_value(selected: &mut Bool, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.toggle_value(&mut selected.value, text);
    Ok(())
}

/// radio_value(current_value: Int, alternative: int, text: str) -> None
///
/// Show a radio button. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///     
///     c = Int(RED)
///     
///     radio_value(c, RED, "red")
///     radio_value(c, GREEN, "green")
///     radio_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn radio_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.radio_value(&mut current_value.value, alternative, text);
    Ok(())
}

/// selectable_value(current_value: Int, alternative: int, text: str) -> None
///
/// Show selectable text. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///     
///     c = Int(RED)
///     
///     selectable_value(c, RED, "red")
///     selectable_value(c, GREEN, "green")
///     selectable_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn selectable_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.selectable_value(&mut current_value.value, alternative, text);
    Ok(())
}

/// combo_box(current_value: Int, alternatives: list[int], names: list[str], label: str) -> None
///
/// Shows a combo box with values defined in "alternatives" and their corresponding names
/// defined in "names"
///
/// Example::
///
///     RED = 0
///     GREEN = 1
///     BLUE = 2
///
///     data = Int(RED)
///
///     def update_func(a):
///         combo_box(data, [RED, GREEN, BLUE], ["red", "green", "blue"], "choose your fate")
#[pyfunction]
unsafe fn combo_box(
    current_value: &mut Int,
    alternatives: Vec<i32>,
    names: Vec<String>,
    label: &str,
) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    egui::ComboBox::from_label(label)
        .selected_text(
            names
                .get(current_value.value.try_into().unwrap_or(0))
                .unwrap_or(&"Unknown".to_string()),
        )
        .show_ui(ui, |ui| {
            for i in 0..alternatives.len() {
                ui.selectable_value(
                    &mut current_value.value,
                    alternatives[i],
                    names.get(i).unwrap_or(&"Unknown".to_string()),
                );
            }
        });
    Ok(())
}

/// progress(value: float) -> None
///
/// A simple progress bar.
/// value in the [0, 1] range, where 1 means “completed”.
///
/// Example::
///
///     progress(0.5)
#[pyfunction]
unsafe fn progress(value: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(egui::widgets::ProgressBar::new(value).show_percentage());
    Ok(())
}

/// spinner() -> None
///
/// A spinner widget used to indicate loading.
///
/// Example::
///
///     spinner()
#[pyfunction]
unsafe fn spinner() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.spinner();
    Ok(())
}

/// color_edit_button_rgb(rgb: RGB) -> None
///
/// Shows a button with the given color. If the user clicks the button, a full color picker is shown.
///
/// Example::
///
///     color = RGB(69, 69, 69)
///     # inside udpate_func
///     color_edit_button_rgb(color)
///     heading(f"r:{color.r} g:{color.g} b:{color.b}")
#[pyfunction]
unsafe fn color_edit_button_rgb(rgb: &mut RGB) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    let mut tmp: [f32; 3] = [rgb.r, rgb.g, rgb.b];

    ui.color_edit_button_rgb(&mut tmp);

    rgb.r = tmp[0];
    rgb.g = tmp[1];
    rgb.b = tmp[2];

    Ok(())
}

/// image(source: str, **kwargs) -> None
///
/// Show an image available at the given uri.
///
/// Example::
///
///     image("https://picsum.photos/480")
///     image("file://assets/ferris.png", max_height = 50, max_width = 50)
#[pyfunction]
#[pyo3(signature = (source, **kwargs))]
unsafe fn image(source: &str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    let mut img = egui::Image::new(source);

    if let Some(kwargs) = kwargs {
        if let Some(height) = kwargs.get_item("max_height")? {
            img = img.max_height(height.cast::<PyInt>()?.extract()?);
        }
        if let Some(width) = kwargs.get_item("max_width")? {
            img = img.max_width(width.cast::<PyInt>()?.extract()?);
        }
    }
    ui.add(img);
    Ok(())
}

/// image_and_text_clicked(source: str, text: str) -> bool
///
/// Creates a button with an image to the left of the text
///
/// Example::
///
///     if image_and_text_clicked("https://picsum.photos/480", "click me"):
///       print("clicked")
#[pyfunction]
unsafe fn image_and_text_clicked(source: &str, text: &str) -> PyResult<bool> {
    let ui = current_ui(&UI)?;

    Ok(ui.add(egui::Button::image_and_text(source, text)).clicked())
}

/// separator() -> None
///
/// A visual separator. A horizontal or vertical line on layout.
///
/// Example::
///
///     separator()
#[pyfunction]
unsafe fn separator() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.separator();
    Ok(())
}

/// set_invisible() -> None
///
/// Calling set_invisible() will cause all further widgets to be invisible, yet still allocate space.
///
/// The widgets will not be interactive (set_invisible() implies disable()).
///
/// Once invisible, there is no way to make the Ui visible again.
///
/// Example::
///
///     set_invisible()
///     heading("this will not be visible")
#[pyfunction]
unsafe fn set_invisible() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.set_invisible();
    Ok(())
}

/// disable() -> None
///
/// Calling disable() will cause the Ui to deny all future interaction and all the widgets will draw with a gray look.
///
/// Usually it is more convenient to use add_enabled.
///
/// Note that once disabled, there is no way to re-enable the Ui.
///
/// Example::
///
///     disable()
///     if button_clicked("you can't click me"):
///       pass
#[pyfunction]
unsafe fn disable() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.disable();
    Ok(())
}

/// add_enabled(enabled: bool, update_fun: Callable[[Context], None]) -> None
///
/// Add a section that is possibly disabled, i.e. greyed out and non-interactive.
///
/// If you call add_enabled from within an already disabled Ui, the result will always be disabled, even if the enabled argument is true.
///
/// Example::
///
///     add_enabled(False, lambda: button_clicked("you can't click me"))
///     button_clicked("but you can click me")
#[pyfunction]
unsafe fn add_enabled(enabled: bool, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .add_enabled_ui(enabled, |ui| run_nested_update_func(ui, update_fun))
        .inner
}

/// set_opacity(opacity: float) -> None
///
/// Make the widget in this Ui semi-transparent.
///
/// opacity must be between 0.0 and 1.0, where 0.0 means fully transparent (i.e., invisible) and 1.0 means fully opaque.
/// Example::
///
///     set_opacity(0.5)
#[pyfunction]
unsafe fn set_opacity(opacity: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.set_opacity(opacity);
    Ok(())
}

/// date_picker_button(selection: Date) -> None
///
/// Shows a date, and will open a date picker popup when clicked.
///
/// Example::
///
///     date = Date(datetime.datetime.now())
///     # inside update_func
///     date_picker_button(date)
#[pyfunction]
unsafe fn date_picker_button(selection: &mut Date) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add(egui_extras::DatePickerButton::new(&mut selection.value));
    Ok(())
}

/// add_space(amount: float) -> None
///
/// Add extra space before the next widget.
///
/// The direction is dependent on the layout.
/// Example::
///
///     add_space(5)
///     heading("I'm so spaced now")
#[pyfunction]
unsafe fn add_space(amount: f32) -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.add_space(amount);
    Ok(())
}


/// close_menu() -> None
///
/// Close the menu we are in (including submenus), if any.
///
/// Example::
///
///    def nested():
///        if button_clicked("submenu button"):
///            close_menu()
///    menu_image_button("https://github.githubassets.com/favicons/favicon.svg", nested)
#[pyfunction]
unsafe fn close_menu() -> PyResult<()> {
    let ui = current_ui(&UI)?;

    ui.close_menu();
    Ok(())
}


/// menu_button(title: str, update_fun: Callable[[Context], None]) -> None
///
/// Create a menu button that when clicked will show the given menu.
/// 
/// If called from within a menu this will instead create a button for a sub-menu.
///
/// Example::
///
///     menu_button("Submenu", lambda: button_clicked("submenu button"))
#[pyfunction]
unsafe fn menu_button(title: &str, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    current_ui(&UI)?
        .menu_button(title, |ui| run_nested_update_func(ui, update_fun))
        .inner;

    Ok(())
}


/// menu_image_button(image_path: str, update_fun: Callable[[Context], None]) -> None
///
///
/// Example::
///
///     menu_image_button("https://github.githubassets.com/favicons/favicon.svg", lambda: button_clicked("submenu button"))
#[pyfunction]
unsafe fn menu_image_button(image_path: &str, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    let icon_data = egui::Image::new(image_path);

    current_ui(&UI)?
        .menu_image_button(icon_data, |ui| run_nested_update_func(ui, update_fun))
        .inner;

    Ok(())
}

/// menu_image_text_button(image_path: str, title: str, update_fun: Callable[[Context], None]) -> None
///
///
/// Example::
///
///     menu_image_text_button("https://github.githubassets.com/favicons/favicon.svg", "title", lambda: button_clicked("submenu button"))
#[pyfunction]
unsafe fn menu_image_text_button(image_path: &str, title: &str, update_fun: Bound<'_, PyAny>) -> PyResult<()> {
    let icon_data = egui::Image::new(image_path);

    current_ui(&UI)?
        .menu_image_text_button(icon_data, title, |ui| run_nested_update_func(ui, update_fun))
        .inner;

    Ok(())
}

#[pymodule]
mod pyegui {
    #[pymodule_export]
    use super::Str;
    #[pymodule_export]
    use super::Bool;
    #[pymodule_export]
    use super::Int;
    #[pymodule_export]
    use super::Float;
    #[pymodule_export]
    use super::RGB;
    #[pymodule_export]
    use super::Date;
    #[pymodule_export]
    use super::Context;
    #[pymodule_export]
    use super::Layout;
    #[pymodule_export]
    use super::LayoutType;
    #[pymodule_export]
    use super::Scope;
    #[pymodule_export]
    use super::Group;
    #[pymodule_export]
    use super::CursorIcon;
    #[pymodule_export]
    use super::Pos2;

    #[pymodule_export]
    use super::run_native;
    #[pymodule_export]
    use super::heading;
    #[pymodule_export]
    use super::monospace;
    #[pymodule_export]
    use super::small;
    #[pymodule_export]
    use super::strong;
    #[pymodule_export]
    use super::weak;
    #[pymodule_export]
    use super::label;
    #[pymodule_export]
    use super::code;
    #[pymodule_export]
    use super::code_editor;
    #[pymodule_export]
    use super::text_edit_singleline;
    #[pymodule_export]
    use super::text_edit_multiline;
    #[pymodule_export]
    use super::button_clicked;
    #[pymodule_export]
    use super::small_button_clicked;
    #[pymodule_export]
    use super::horizontal;
    #[pymodule_export]
    use super::horizontal_centered;
    #[pymodule_export]
    use super::horizontal_top;
    #[pymodule_export]
    use super::horizontal_wrapped;
    #[pymodule_export]
    use super::vertical;
    #[pymodule_export]
    use super::vertical_centered;
    #[pymodule_export]
    use super::vertical_centered_justified;
    #[pymodule_export]
    use super::centered_and_justified;
    #[pymodule_export]
    use super::collapsing;
    #[pymodule_export]
    use super::indent;
    #[pymodule_export]
    use super::group;
    #[pymodule_export]
    use super::scroll_area_vertical;
    #[pymodule_export]
    use super::scroll_area_horizontal;
    #[pymodule_export]
    use super::scope;
    #[pymodule_export]
    use super::slider_float;
    #[pymodule_export]
    use super::slider_int;
    #[pymodule_export]
    use super::drag_int;
    #[pymodule_export]
    use super::drag_float;
    #[pymodule_export]
    use super::hyperlink;
    #[pymodule_export]
    use super::hyperlink_to;
    #[pymodule_export]
    use super::link_clicked;
    #[pymodule_export]
    use super::checkbox;
    #[pymodule_export]
    use super::radio_value;
    #[pymodule_export]
    use super::toggle_value;
    #[pymodule_export]
    use super::selectable_value;
    #[pymodule_export]
    use super::combo_box;
    #[pymodule_export]
    use super::progress;
    #[pymodule_export]
    use super::spinner;
    #[pymodule_export]
    use super::color_edit_button_rgb;
    #[pymodule_export]
    use crate::image;
    #[pymodule_export]
    use super::image_and_text_clicked;
    #[pymodule_export]
    use super::separator;
    #[pymodule_export]
    use super::set_invisible;
    #[pymodule_export]
    use super::disable;
    #[pymodule_export]
    use super::add_enabled;
    #[pymodule_export]
    use super::set_opacity;
    #[pymodule_export]
    use super::date_picker_button;
    #[pymodule_export]
    use super::add_space;
    #[pymodule_export]
    use super::close_menu;
    #[pymodule_export]
    use super::menu_button;
    #[pymodule_export]
    use super::menu_image_button;
    #[pymodule_export]
    use super::menu_image_text_button;
}
