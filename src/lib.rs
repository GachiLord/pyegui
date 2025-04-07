#![allow(static_mut_refs)]

use pyo3::prelude::*;
use pyo3::{exceptions::PyRuntimeError, types::PyFunction};
use eframe::{egui, self};
use egui_extras;
use std::sync::Mutex;
use std::ptr;
use chrono::NaiveDate;

// state

static mut UI: *mut Vec<*mut egui::Ui> = ptr::null_mut();
static mut APP_MUTEX: Mutex<()> = Mutex::new(());
static mut UPDATE_FUNC: *const Py<PyFunction> = ptr::null();

// messages

static APP_MUTEX_ERR: &'static str = "run_simple_native has been called on a separate thread";
static UPDATE_FUNC_PTR_NULL_ERR: &'static str = "UPDATE_FUNC ptr is null. This is likely to be a problem with pyegui";
static UI_PTR_NULL_ERR: &'static str = "UI ptr is null. This is likely to be a problem with pyegui";
static UI_STACK_ERR: &'static str = "UI stack is empty. This is likely to be a problem with pyegui";
static UI_CALL_OUTSIDE_UPDATE_FUNC: &'static str = "UI functions should be called only within update_fun and on the same thread. update_fun should only be called by run_simple_native";

// classes

#[pyclass]
struct Context(egui::Context);

#[pyclass]
struct Str {
	#[pyo3(get, set)]
	value: String
}

#[pymethods]
impl Str {
    #[new]
    fn new(value: String) -> Self {
        Str { value }
    }
}

#[pyclass]
struct Bool {
	#[pyo3(get, set)]
	value: bool
}

#[pymethods]
impl Bool {
    #[new]
    fn new(value: bool) -> Self {
        Bool { value }
    }
}

#[pyclass]
struct Int {
	#[pyo3(get, set)]
	value: i32
}

#[pymethods]
impl Int {
    #[new]
    fn new(value: i32) -> Self {
        Int { value }
    }
}

#[pyclass]
struct Float {
	#[pyo3(get, set)]
	value: f32
}

#[pymethods]
impl Float {
    #[new]
    fn new(value: f32) -> Self {
        Float { value }
    }
}


#[pyclass]
struct RGB {
	#[pyo3(get, set)]
  r: f32,
	#[pyo3(get, set)]
  g: f32,
	#[pyo3(get, set)]
  b: f32,
}

#[pymethods]
impl RGB {
    #[new]
    fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
}

#[pyclass]
struct Date {
	#[pyo3(get, set)]
	value: NaiveDate
}

#[pymethods]
impl Date {
    #[new]
    fn new(value: NaiveDate) -> Self {
        Date { value }
    }
}

// Start function

#[derive(Default)]
struct PyeguiApp;

impl eframe::App for PyeguiApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

		let ctx_r = Context(ctx.clone());

    unsafe {

      egui::CentralPanel::default().show(ctx, |ui| {

        let ui_stack = UI.as_mut().expect(UI_PTR_NULL_ERR);
        ui_stack.push(&raw mut *ui);

        if let Err(err) = Python::with_gil(|py| {
          UPDATE_FUNC.as_ref().expect(UPDATE_FUNC_PTR_NULL_ERR).call1(py, (ctx_r,))
        }) {
          println!("update_fun threw an error: {}", err.to_string());
        }

        ui_stack.pop().expect(UI_STACK_ERR);
      });

    }
  }
}

/// Creates window and runs update_func.
/// This is an entrypoint for your GUI application.
///
/// Example:
/// name = Str("")
///
/// def update_func(ctx):
/// 	heading(f"Hello, {name.value}!")
///		text_edit_singleline(name)
/// 
/// 	if button_clicked("click me"):
///			print("clicked")
///
/// run_simple_native("My app", update_func)
#[pyfunction]
unsafe fn run_native(
    app_name: &str,
    // native_options: eframe::NativeOptions,
    update_fun: Bound<'_, PyFunction>,
) -> PyResult<()> {
	// ensure thread safety 
	let _lock = APP_MUTEX.try_lock().expect(APP_MUTEX_ERR);
	// init UI stack
	let mut ui_stack = Vec::with_capacity(32);
	UI = &raw mut *&mut ui_stack;
	// set update_func
	UPDATE_FUNC = &raw const *&update_fun.unbind();
	// create a window
	let result = eframe::run_native(
        app_name,
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<PyeguiApp>::default())
        }),
  );

	match result {
		Ok(_) => Ok(()),
		Err(err) => return Err(PyRuntimeError::new_err(format!("Cannot create a window: {}", err.to_string())))
	}
}

// helpers

unsafe fn ui_stack(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut Vec<*mut egui::Ui>> {
    ui.as_mut().ok_or(PyRuntimeError::new_err(UI_CALL_OUTSIDE_UPDATE_FUNC))
}

unsafe fn last_ui(ui_stack: &mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
  let last_ui = ui_stack.last_mut().ok_or(PyRuntimeError::new_err(UI_STACK_ERR))?;

  last_ui.as_mut().ok_or(PyRuntimeError::new_err(UI_PTR_NULL_ERR))
}

unsafe fn current_ui(ui: &*mut Vec<*mut egui::Ui>) -> PyResult<&mut egui::Ui> {
  last_ui(ui_stack(ui)?)  
}

// UI functions

/// Show large text
///
/// Example:
/// heading("hello") 
#[pyfunction]
unsafe fn heading(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.heading(text);
  Ok(())
}

/// Show monospace (fixed width) text.
///
/// Example:
/// monospace("hello") 
#[pyfunction]
unsafe fn monospace(text: &str) -> PyResult<()>  {
	let ui = current_ui(&UI)?;

	ui.monospace(text);
  Ok(())
}

/// Show small text.
///
/// Example:
/// small("hello") 
#[pyfunction]
unsafe fn small(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.small(text);
  Ok(())
}

/// Show text that stand out a bit (e.g. slightly brighter).
///
/// Example:
/// strong("hello") 
#[pyfunction]
unsafe fn strong(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.strong(text);
  Ok(())
}

/// Show text that is weaker (fainter color).
///
/// Example:
/// weak("hello") 
#[pyfunction]
unsafe fn weak(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.weak(text);
  Ok(())
}

/// Show some text.
///
/// Example:
/// label("some text") 
#[pyfunction]
unsafe fn label(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.label(text);
  Ok(())
}

/// Show text as monospace with a gray background.
///
/// Example:
/// code("print(42 + 27)") 
#[pyfunction]
unsafe fn code(text: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.code(text);
  Ok(())
}

/// Show singleline text field and update the text
///
/// Example:
/// text = Str("print(42 + 27)")
/// # inside update func
/// code_editor(text)
#[pyfunction]
unsafe fn code_editor(text: &mut Str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.code_editor(&mut text.value);
  Ok(())
}

/// Show singleline text field and update the text
///
/// Example:
/// text = Str("editable")
/// # inside update func
/// text_edit_singleline(text)
#[pyfunction]
unsafe fn text_edit_singleline(text: &mut Str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.text_edit_singleline(&mut text.value);
  Ok(())
}

/// Show multiline text field and update the text
/// 
/// Example:
/// text = Str("editable")
/// # inside update func
/// text_edit_multiline(text)
#[pyfunction]
unsafe fn text_edit_multiline(text: &mut Str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

	ui.text_edit_multiline(&mut text.value);
  Ok(())
}

/// Returns true if the button was clicked this frame
/// 
/// if button_clicked("click me"):
///		print("click me, my friend")
#[pyfunction]
unsafe fn button_clicked(text: &str) -> PyResult<bool> {
	let ui = current_ui(&UI)?;

	Ok(ui.button(text).clicked())
}

/// Returns true if the small button was clicked this frame
/// 
/// if small_button_clicked("click me"):
///		print("click me, my friend")
#[pyfunction]
unsafe fn small_button_clicked(text: &str) -> PyResult<bool> {
	let ui = current_ui(&UI)?;

	Ok(ui.small_button(text).clicked())
}

/// Start a ui with horizontal layout. After you have called this, the function registers the contents as any other widget.
/// 
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
/// 
/// If you don’t want the contents to be centered, use horizontal_top instead.
/// 
/// Example:
/// def horizontal_update_func():
/// 	heading("I'm horizontal")
/// 
/// horizontal(horizontal_update_func)
#[pyfunction]
unsafe fn horizontal(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.horizontal(|ui| {
	  let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Like horizontal, but allocates the full vertical height and then centers elements vertically.
#[pyfunction]
unsafe fn horizontal_centered(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.horizontal_centered(|ui| {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}
/// Like horizontal, but aligns content with top.
#[pyfunction]
unsafe fn horizontal_top(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.horizontal_top(|ui| {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Start a ui with horizontal layout that wraps to a new row when it reaches the right edge of the max_size. After you have called this, the function registers the contents as any other widget.
/// 
/// Elements will be centered on the Y axis, i.e. adjusted up and down to lie in the center of the horizontal layout. The initial height is style.spacing.interact_size.y. Centering is almost always what you want if you are planning to mix widgets or use different types of text.
#[pyfunction]
unsafe fn horizontal_wrapped(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.horizontal_wrapped(|ui| {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}


/// A CollapsingHeader that starts out collapsed.
///
/// Example:
/// def update_func():
///   heading("hi")
/// collapsing("collapsed", update_func)
#[pyfunction]
unsafe fn collapsing(heading: &str, update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.collapsing(heading, |ui| {

    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).body_returned.unwrap() {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Create a child ui which is indented to the right.
/// Example:
/// def update_func():
///   heading("I'm indented")
/// indent(update_func)
#[pyfunction]
unsafe fn indent(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.indent("your mom", |ui| {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Visually groups the contents together.
///
/// Example
/// def update_func():
///   heading("hi")
///   heading("there")
/// 
/// group(update_func)
#[pyfunction]
unsafe fn group(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.group(|ui| {
    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Create a scoped child ui.
/// 
/// You can use this to temporarily change the Style of a sub-region.
///
/// Example
/// def update_func():
///   heading("0.5 opacity")
///   set_opacity(0.5)
/// 
/// heading("normal opacity")
/// scope(update_func)
#[pyfunction]
unsafe fn scope(update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.scope(|ui| {

    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Control float with a slider.
///
/// Example:
/// data = Float(5) 
/// # inside update_func 
/// slider_float(data, 0, 50, "slide me")
#[pyfunction]
unsafe fn slider_float(value: &mut Float, min: f32, max: f32, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.add(egui::Slider::new(&mut value.value, min..=max).text(text));
  Ok(())
}

/// Control int with a slider.
/// 
/// Example:
/// data = Int(5) 
/// # inside update_func 
/// slider_float(data, 0, 50, "slide me")
#[pyfunction]
unsafe fn slider_int(value: &mut Int, min: i32, max: i32, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.add(egui::Slider::new(&mut value.value, min..=max).text(text).integer());
  Ok(())
}


/// Control float by dragging the number.
///
/// Example:
/// data = Float(5) 
/// # inside update_func 
/// drag_float(data, 0, 50, 1.5)
#[pyfunction]
unsafe fn drag_float(value: &mut Float, min: f32, max: f32, speed: f32) -> PyResult<()> {
  let ui = current_ui(&UI)?;
 
  ui.add(egui::DragValue::new(&mut value.value).speed(speed).range(min..=max));
  Ok(())
}

/// Control int by dragging the number.
///
/// Example:
/// data = Int(5) 
/// # inside update_func 
/// drag_int(data, 0, 50, 1)
#[pyfunction]
unsafe fn drag_int(value: &mut Int, min: i32, max: i32, speed: f32) -> PyResult<()> {
  let ui = current_ui(&UI)?;
 
  ui.add(egui::DragValue::new(&mut value.value).speed(speed).range(min..=max));
  Ok(())
}

/// A clickable hyperlink
/// 
/// Example:
/// hyperlink("https://github.com/emilk/egui")
#[pyfunction]
unsafe fn hyperlink(url: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.hyperlink(url);
  Ok(())
}

/// A clickable hyperlink with label
/// 
/// Example:
/// hyperlink_to("egui on GitHub", "https://www.github.com/emilk/egui/")
#[pyfunction]
unsafe fn hyperlink_to(label: &str, url: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.hyperlink_to(label, url);
  Ok(())
}


/// Clickable text, that looks like a hyperlink.
/// To link to a web page, use hyperlink or hyperlink_to.
/// 
/// Example:
/// if link_clicked("egui on GitHub"):
///   print("clicked on a fake link")
#[pyfunction]
unsafe fn link_clicked(label: &str) -> PyResult<bool> {
  let ui = current_ui(&UI)?;
  
  Ok(ui.link(label).clicked())
}

/// Show a checkbox.
/// 
/// Example:
/// data = Bool(false)
/// # inside update_func
/// checkbox(data, "check me")
#[pyfunction]
unsafe fn checkbox(checked: &mut Bool, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.checkbox(&mut checked.value, text);
  Ok(())
}

/// Acts like a checkbox, but looks like a selectable label.
/// 
/// Example:
/// data = Bool(false)
/// # inside update_func
/// toggle_value(data, "check me")
#[pyfunction]
unsafe fn toggle_value(selected: &mut Bool, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.toggle_value(&mut selected.value, text);
  Ok(())
}


/// Show a radio button. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
/// 
/// Example:
/// RED = 0
/// GREEN = 1
/// BLUE = 2
/// 
/// c = Int(RED)
/// 
/// radio_value(c, RED, "red")
/// radio_value(c, GREEN, "green")
/// radio_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn radio_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.radio_value(&mut current_value.value, alternative, text);
  Ok(())
}


/// Show selectable text. It is selected if current_value == selected_value. If clicked, selected_value is assigned to current_value.
/// 
/// Example:
/// RED = 0
/// GREEN = 1
/// BLUE = 2
/// 
/// c = Int(RED)
/// 
/// selectable_value(c, RED, "red")
/// selectable_value(c, GREEN, "green")
/// selectable_value(c, BLUE, "blue")
#[pyfunction]
unsafe fn selectable_value(current_value: &mut Int, alternative: i32, text: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.selectable_value(&mut current_value.value, alternative, text);
  Ok(())
}

/// Shows a combo box with values defined in "alternatives" and their corresponding names
/// defined in "names"
/// 
/// Example:
/// RED = 0
/// GREEN = 1
/// BLUE = 2
///
/// data = Int(RED)
///
/// def update_func(a):
///     combo_box(data, [RED, GREEN, BLUE], ["red", "green", "blue"], "choose your fate")
#[pyfunction]
unsafe fn combo_box(current_value: &mut Int, alternatives: Vec<i32>, names: Vec<String>, label: &str) -> PyResult<()> {
	let ui = current_ui(&UI)?;

  egui::ComboBox::from_label(label)
    .selected_text(names.get(current_value.value.try_into().unwrap_or(0)).unwrap_or(&"Unknown".to_string()))
    .show_ui(ui, |ui| {
      for i in 0..alternatives.len() {
        ui.selectable_value(
          &mut current_value.value, 
          alternatives[i], 
          names.get(i).unwrap_or(&"Unknown".to_string())
        );
      }
    }
	);
  Ok(())
}

/// A simple progress bar.
/// value in the [0, 1] range, where 1 means “completed”.
///
/// Example:
///
/// progress(0.5)
#[pyfunction]
unsafe fn progress(value: f32) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.add(egui::widgets::ProgressBar::new(value).show_percentage());
  Ok(())
}


/// A spinner widget used to indicate loading.
///
/// Example:
///
/// spinner()
#[pyfunction]
unsafe fn spinner() -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.spinner();
  Ok(())
}

/// Shows a button with the given color. If the user clicks the button, a full color picker is shown.
/// 
/// Example:
///
/// color = RGB(69, 69, 69)
/// # inside udpate_func
/// color_edit_button_rgb(color)
/// heading(f"r:{color.r} g:{color.g} b:{color.b}")
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


/// Show an image available at the given uri.
///
/// Example:
///
/// image("https://picsum.photos/480");
/// image("file://assets/ferris.png");
#[pyfunction]
unsafe fn image(source: &str) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.image(source);
  Ok(())
}

/// Creates a button with an image to the left of the text 
///
/// Example:
///
/// if image_and_text_clicked("https://picsum.photos/480", "click me"):
///   print("clicked")
#[pyfunction]
unsafe fn image_and_text_clicked(source: &str, text: &str) -> PyResult<bool> {
  let ui = current_ui(&UI)?;
  
  Ok(ui.add(egui::Button::image_and_text(source, text)).clicked())
}

/// A visual separator. A horizontal or vertical line on layout.
///
/// Example:
/// separator();
#[pyfunction]
unsafe fn separator() -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.separator();
  Ok(())
}


/// Calling set_invisible() will cause all further widgets to be invisible, yet still allocate space.
/// 
/// The widgets will not be interactive (set_invisible() implies disable()).
/// 
/// Once invisible, there is no way to make the Ui visible again.
///
/// Example:
/// set_invisible();
/// heading("this will not be visible")
#[pyfunction]
unsafe fn set_invisible() -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.set_invisible();
  Ok(())
}

/// Calling disable() will cause the Ui to deny all future interaction and all the widgets will draw with a gray look.
/// 
/// Usually it is more convenient to use add_enabled.
/// 
/// Note that once disabled, there is no way to re-enable the Ui.
///
/// Example:
///
/// disable()
/// if button_clicked("you can't click me"):
///   pass
#[pyfunction]
unsafe fn disable() -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.disable();
  Ok(())
}

/// Add a section that is possibly disabled, i.e. greyed out and non-interactive.
/// 
/// If you call add_enabled from within an already disabled Ui, the result will always be disabled, even if the enabled argument is true.
/// 
/// Example:
/// add_enabled(False, lambda: button_clicked("you can't click me"))
/// button_clicked("but you can click me")
#[pyfunction]
unsafe fn add_enabled(enabled: bool, update_fun: Bound<'_, PyFunction>) -> PyResult<()> {

	match current_ui(&UI)?.add_enabled_ui(enabled, |ui| {

    let ui_stack = ui_stack(&UI).unwrap_unchecked();

		ui_stack.push(&raw mut *ui);

		if let Err(err) = update_fun.call0() {
			println!("update_fun threw an error: {}", err.to_string());
		}

		ui_stack.pop()

	}).inner {
    Some(_) => Ok(()),
    None => Err(PyRuntimeError::new_err(UI_STACK_ERR))
  }
}

/// Make the widget in this Ui semi-transparent.
/// 
/// opacity must be between 0.0 and 1.0, where 0.0 means fully transparent (i.e., invisible) and 1.0 means fully opaque.
/// Example:
///
/// set_opacity(0.5)
#[pyfunction]
unsafe fn set_opacity(opacity: f32) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.set_opacity(opacity);
  Ok(())
}


/// Shows a date, and will open a date picker popup when clicked.
/// 
/// Example:
/// date = Date(datetime.datetime.now())
/// # inside update_func
/// date_picker_button(date)
#[pyfunction]
unsafe fn date_picker_button(selection: &mut Date) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.add(egui_extras::DatePickerButton::new(&mut selection.value));
  Ok(())
}

/// Add extra space before the next widget.
/// 
/// The direction is dependent on the layout.
/// Example:
///
/// add_space(5)
/// heading("I'm so spaced now")
#[pyfunction]
unsafe fn add_space(amount: f32) -> PyResult<()> {
  let ui = current_ui(&UI)?;
  
  ui.add_space(amount);
  Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyegui(m: &Bound<'_, PyModule>) -> PyResult<()> {
	// classes
	m.add_class::<Str>()?;
	m.add_class::<Bool>()?;
	m.add_class::<Int>()?;
	m.add_class::<Float>()?;
	m.add_class::<RGB>()?;
	m.add_class::<Date>()?;
	// functions
	m.add_function(wrap_pyfunction!(run_native, m)?)?;
	m.add_function(wrap_pyfunction!(heading, m)?)?;
	m.add_function(wrap_pyfunction!(monospace, m)?)?;
	m.add_function(wrap_pyfunction!(small, m)?)?;
	m.add_function(wrap_pyfunction!(strong, m)?)?;
	m.add_function(wrap_pyfunction!(weak, m)?)?;
	m.add_function(wrap_pyfunction!(label, m)?)?;
	m.add_function(wrap_pyfunction!(code, m)?)?;
	m.add_function(wrap_pyfunction!(code_editor, m)?)?;
	m.add_function(wrap_pyfunction!(text_edit_singleline, m)?)?;
	m.add_function(wrap_pyfunction!(text_edit_multiline, m)?)?;
	m.add_function(wrap_pyfunction!(button_clicked, m)?)?;
	m.add_function(wrap_pyfunction!(small_button_clicked, m)?)?;
	m.add_function(wrap_pyfunction!(horizontal, m)?)?;
	m.add_function(wrap_pyfunction!(horizontal_centered, m)?)?;
	m.add_function(wrap_pyfunction!(horizontal_top, m)?)?;
	m.add_function(wrap_pyfunction!(horizontal_wrapped, m)?)?;
	m.add_function(wrap_pyfunction!(collapsing, m)?)?;
	m.add_function(wrap_pyfunction!(indent, m)?)?;
	m.add_function(wrap_pyfunction!(group, m)?)?;
	m.add_function(wrap_pyfunction!(scope, m)?)?;
	m.add_function(wrap_pyfunction!(slider_float, m)?)?;
	m.add_function(wrap_pyfunction!(slider_int, m)?)?;
	m.add_function(wrap_pyfunction!(drag_int, m)?)?;
	m.add_function(wrap_pyfunction!(drag_float, m)?)?;
	m.add_function(wrap_pyfunction!(hyperlink, m)?)?;
	m.add_function(wrap_pyfunction!(hyperlink_to, m)?)?;
	m.add_function(wrap_pyfunction!(link_clicked, m)?)?;
	m.add_function(wrap_pyfunction!(checkbox, m)?)?;
	m.add_function(wrap_pyfunction!(radio_value, m)?)?;
	m.add_function(wrap_pyfunction!(toggle_value, m)?)?;
	m.add_function(wrap_pyfunction!(selectable_value, m)?)?;
	m.add_function(wrap_pyfunction!(combo_box, m)?)?;
	m.add_function(wrap_pyfunction!(progress, m)?)?;
	m.add_function(wrap_pyfunction!(spinner, m)?)?;
	m.add_function(wrap_pyfunction!(color_edit_button_rgb, m)?)?;
	m.add_function(wrap_pyfunction!(image, m)?)?;
	m.add_function(wrap_pyfunction!(image_and_text_clicked, m)?)?;
	m.add_function(wrap_pyfunction!(separator, m)?)?;
	m.add_function(wrap_pyfunction!(set_invisible, m)?)?;
	m.add_function(wrap_pyfunction!(disable, m)?)?;
	m.add_function(wrap_pyfunction!(add_enabled, m)?)?;
	m.add_function(wrap_pyfunction!(set_opacity, m)?)?;
	m.add_function(wrap_pyfunction!(date_picker_button, m)?)?;
	m.add_function(wrap_pyfunction!(add_space, m)?)?;
	Ok(())
}

