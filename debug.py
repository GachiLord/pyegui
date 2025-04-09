from pyegui import *
import logging

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

def update_func(ctx):
  heading("Debugging your mom")
  heading("I'm pretty good at that")


run_native("Debug", update_func)

