from pyegui import *
from datetime import datetime, timedelta

def update_func(ctx):
    ctx.request_repaint_after(timedelta(seconds=1))

    now = datetime.now().strftime("%H:%M:%S")
    heading(now)

if __name__ == "__main__":
  run_native("Hello World App", update_func)

