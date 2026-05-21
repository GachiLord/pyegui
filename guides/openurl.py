from pyegui import *


def update_func(ctx):
    hyperlink_to("pyeguion GitHub", "https://github.com/GachiLord/pyegui")

    if button_clicked("Open url"):
        ctx.open_url("https://github.com")

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
