from pyegui import *

DARK = 0
LIGHT = 1

theme = Int(DARK)

def get_theme_index(ctx):
  if ctx.is_dark_theme:
    return DARK
  return LIGHT
  
def set_theme(ctx, idx):
  if idx == DARK:
    ctx.set_dark_theme()
  else:
    ctx.set_light_theme()

def update_func(ctx):
  theme.value = get_theme_index(ctx)
  
  radio_value(theme, DARK, "dark")  
  radio_value(theme, LIGHT, "light")

  set_theme(ctx, theme.value)

if __name__ == "__main__":
  run_native("themes", update_func)
