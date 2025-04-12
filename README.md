# pyegui

**pyegui** is a native extenstion for Python that provides bindings for Rust immediate mode GUI library [egui](https://github.com/emilk/egui).

## Example

```python
from pyegui import *

name = Str("Van")
age = Int(24)

def update_func(ctx):
  heading("My egui Application")
  text_edit_singleline(name, hint_text="Your name")
  slider_int(age, 0, 150, "age")

  if button_clicked("Increment"):
    age.value += 1

  heading(f"Hello '{name.value}', age {age.value}")
  image("file://image.png", max_width=350, max_height=250)

run_native("My pyegui Application", update_func)
```
![example 1](https://github.com/GachiLord/pyegui/raw/main/example1.jpeg "Dark theme")
![example 2](https://github.com/GachiLord/pyegui/raw/main/example2.jpeg  "White theme")

## Features

**pyegui** tries to be as close as possible to the original egui API, but with the focus on simplicity and usability. 
Callbacks were removed where possible to accomplish more smooth expirience in Python.

- Light and Dark themes(defaults to the system's)
- Built-in latin and cyrillic alphabets. You can load any font you want with `ctx.set_font` function
- Images(png and jpeg)
- Date picker
- RBG color picker
- Text fields, radio buttons, buttons, code, progress bar etc.
- No dependencies which destroy you project when you distribute it. Just pure giant Rust binary

Full list of implemented features is available [here](https://github.com/GachiLord/pyegui/blob/main/TODO.md)

## Install

Prebuilt binaries are provided for Linux and Windows.
On other platforms pip will build wheel for your OS.
In this case you'll need Rust compiler and [maturin](https://github.com/PyO3/maturin)

Install from pypi
```bash
pip install pyegui
```
Install from source
```bash
git clone https://github.com/gachilord/pyegui
pip install <path to pyegui>
```

## Usage

This is how you write a "hello world" app.
```python
from pyegui import *

def update_func(ctx):
  # draw UI here
  heading("Hello, World!")

run_native("Example app", update_func)
```

You can find more examples in this [folder](https://github.com/GachiLord/pyegui/tree/main/examples). Also Python's `help(pyegui.some_function)` will be quite effective(there are examples for every function).
Read the source code of this binding and the original [library](https://github.com/emilk/egui).

### Update functions

**pyegui** has a notion of update functions which the library calls to draw your UI.
```python
def update_func():
  # you can place here any widget
  heading("I'm a heading")
  # some widgets are interactive
  if button_clicked("I'm a clickable button"):
    # you can update state from here or show another widget
    print("Clicked")
```
The top level update function has the Context object that controls global aspects of your app(e.g fonts and theme).
```python
def update_func(ctx):
  ctx.set_light_theme()
  heading("Using light theme even if system's is dark")
```
Update functions may be nested. Such functions create a new UI scope that can have different styles and befaviour.
```python
def update_func(ctx):
  # define update_func
  def nested():
    label("I'm a label inside nested update function")
    label("New label")
    disable() # this function will disable all further widgets in the scope
    if button_clicked("You can't click me"):
      print("Unreachable")
  # all the widgets inside 'nested' will be centered vertically 
  horizontal_centered(nested)
  # this widget won't be disabled though it goes after 'disable()'
  if button_clicked("You can click me"):
    print("Clicked")
```

### Variables

Many widgets require access to a state via a reference, which can't be done for integers, floats and strings in Python.
That's why such helper classes as Str, Bool, Int, Float, RGB and Date exist. 

They are essentially the following:
```python
# Example for bool type
class Bool:
  value = False
```

These classes can be used to draw UI or to store user input. 
You have to create them outside of update functions.
```python
data = Bool(False)

def update_func():
  heading(f"Value of the data is {data.value}")
  # button will be shown only if the checkbox is checked 
  if data.value and button_clicked("set to False"):
    # hiding the button
    data.value = False
  checkbox(data, "Check me")
```
