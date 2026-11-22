from pyegui import *
from datetime import datetime, timedelta

factor = 1
indent_value = 0

def update_func(ctx):
    global indent_value, factor
    indent_value += 1 * factor

    if indent_value > 500:
        factor = -1
    elif indent_value <= 0:
        factor = 1

    add_space(indent_value)
    image("https://github.githubassets.com/favicons/favicon.svg", max_width=240, max_height=320)

    ctx.request_repaint()

if __name__ == "__main__":
  run_native("Hello World App", update_func)

