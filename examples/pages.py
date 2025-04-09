from pyegui import *
import logging

PAGE_1 = 1
PAGE_2 = 2
PAGE_3 = 3

page = Int(PAGE_1)

def update_func(ctx):
  label("You can emulate pages in your app by using some variable and a few conditions")
  label(f"For example, this app has 3 pages. Current page is {page.value}")

  separator()

  def hor():
    selectable_value(page, PAGE_1, "Go to page 1")
    selectable_value(page, PAGE_2, "Go to page 2")
    selectable_value(page, PAGE_3, "Go to page 3")

  horizontal(hor)

  separator()

  if page.value == PAGE_1:
    heading("Page 1")
    label("Distinctio ut assumenda officia necessitatibus consequatur. Quia laborum illo aliquid delectus. Molestiae maiores unde aut quas saepe laboriosam et voluptatum. Provident beatae suscipit id. Quam odio corrupti quia voluptate fugiat.")
  if page.value == PAGE_2:
    heading("Page 2")
    label("Lorem ipsum is a dummy or placeholder text commonly used in graphic design, publishing, testing, and web development.")
  if page.value == PAGE_3:
    heading("Page 3")
    label("Its purpose is to permit a page layout to be designed, independently of the copy that will subsequently populate it, or to demonstrate various fonts of a typeface without meaningful text that could be distracting. Lorem ipsum is typically a corrupted version of De finibus bonorum et malorum, a 1st-century BC text by the Roman statesman and philosopher Cicero, with words altered, added, and removed to make it nonsensical and improper Latin.")

if __name__ == "__main__":
  run_native("pages", update_func)
