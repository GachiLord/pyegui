from pyegui import *

def app():
  heading("I'm horizontal")

def update_func(ctx):
    horizontal_centered(app)

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
