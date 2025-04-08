import datetime
from pyegui import *

RED = 0                       
GREEN = 1
BLUE = 2

text_singline = Str("")
text_multiline = Str("")
checkbox_value = Bool(False)
color = Int(RED)
color_rgb = RGB(69, 69, 69)
date = Date(datetime.datetime.now())
is_visible = Bool(True)
is_interactive = Bool(True)
opacity_value = Float(1)

def update_func(ctx):
  label("Welcome to the widget gallery!")

  hyperlink_to("pyegui on GitHub", "https://github.com/GachiLord/pyegui")

  text_edit_singleline(text_singline, hint_text="hint text")
  text_edit_multiline(text_multiline, hint_text="hint text")
  code("print(69)")
  code_editor(text_multiline)

  if button_clicked("clear text"):
    text_singline.value = ""

  if link_clicked("I'm a fake link"):
    text_singline.value = "new text"

  checkbox(checkbox_value, "check me")

  def radios():
    radio_value(color, RED, "red")
    radio_value(color, GREEN, "green")
    radio_value(color, BLUE, "blue")
  horizontal(radios)

  def selectables():
    selectable_value(color, RED, "red")
    selectable_value(color, GREEN, "green")
    selectable_value(color, BLUE, "blue")
  horizontal(selectables)

  combo_box(color, [RED, GREEN, BLUE], ["red", "green", "blue"], "Choose your fate") 

  slider_int(color, 0, 2, "slide me")

  drag_int(color, 0, 2, 1)

  progress(1 / (color.value + 1))

  color_edit_button_rgb(color_rgb)

  image("https://github.githubassets.com/favicons/favicon.svg", max_width=240, max_height=320)

  if image_and_text_clicked("https://github.githubassets.com/favicons/favicon.svg", "button with image"):
    print("clicked on image with text")

  date_picker_button(date) 

  collapsing("collapsed", lambda: heading("I'm so collapsed now"))

  separator()

  heading("Configure block in the frame")

  def hideable():
    set_opacity(opacity_value.value)
    if not is_visible.value:
      set_invisible()
    if not is_interactive.value:
      disable()
      
    heading("This block may be hidden")
    button_clicked("Try to click me")

  group(hideable)

  def footer():
    checkbox(is_visible, "Visible")
    checkbox(is_interactive, "Interactive")
    slider_float(opacity_value, 0.0, 1.0, "Opacity")
  horizontal(footer)


if __name__ == "__main__":
  run_native("Widget Gallery", update_func, inner_height=900, inner_width=500)

