from pyegui import *

# This item is to big to be inserted into clipboard
# text_to_copy = "your mom" 

text_to_copy = "smol item"

def update_func(ctx):
    if button_clicked("Copy"):
        ctx.copy_text(text_to_copy)

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
