from pyegui import *

text = Str("many lines of text\nmany lines of text\nmany lines of text")
textoneline = Str("one line of text")

def update_func(ctx):
    text_edit_multiline(text, hint_text="hint")
    text_edit_singleline(textoneline, hint_text="hint")

if __name__ == "__main__":
    run_native("Hello World App", update_func)
  
