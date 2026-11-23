from pyegui import *

FACTOR = 4.0

def update_func(ctx):
    ctx.set_zoom_factor(FACTOR)
    if ctx.zoom_factor >= FACTOR:
        heading("I'm big")

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
