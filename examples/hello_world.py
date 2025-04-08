from pyegui import *

def update_func(ctx):
  heading("Hello, World!")

if __name__ == "__main__":
  run_native("Hello World App", update_func)
  
