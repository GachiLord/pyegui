from pyegui import *

def update_func(ctx):
  # provide absolute or relative path to the file
  ctx.set_font("NotoSansJP-VariableFont_wght.ttf")
  # do cool stuff with Japanese fonts
  heading("天気の子")

if __name__ == "__main__":
  run_native("fonts", update_func)

