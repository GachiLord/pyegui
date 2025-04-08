from pyegui import *
import time

buf = Str("")
output = Str("")

def update_func(ctx):
  code_editor(buf)  
  separator()
  label(output.value)
  if button_clicked("Run"):
    output.value = ""
    try:
      t1 = time.time_ns()
      exec(buf.value)
      t2 = time.time_ns()
      output.value = f"Run {t2-t1}ns"
    except Exception as e:
      output.value = str(e)

if __name__ == "__main__":
  run_native("Python IDE", update_func)
