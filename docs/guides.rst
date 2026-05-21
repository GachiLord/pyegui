How to
===========

Copy text to clipboard
------------------------

All text elements support copy operation:

.. literalinclude:: ../guides/copytext.py
   :language: python
   :linenos:

.. image:: _static/copytext.png
   :alt: text fields screenshot

However you can copy text programmatically:

.. literalinclude:: ../guides/copytextprog.py
   :language: python
   :linenos:

.. image:: _static/copytextprog.png
   :alt: copy text button screenshot

Open URL in a browser
--------------------------

Links are opened when clicked on. Also you can open them using ``ctx.open_url()``

.. literalinclude:: ../guides/openurl.py
   :language: python
   :linenos:

.. image:: _static/openurl.png
   :alt: copy text button screenshot

Display non-latin and non-cyrillic characters
-------------------------------------------------------

Download ``ttf`` font file and distribute it with your app

.. literalinclude:: ../guides/fonts.py
   :language: python
   :linenos:

.. image:: _static/fonts.png
   :alt: fonts screenshot


Set theme
-------------

By default system's theme is used.
Theme is changed using these methods:

.. code-block:: python

  ctx.set_light_theme()
  ctx.set_dark_theme()
  ctx.set_system_theme()

Example app:

.. literalinclude:: ../guides/themes.py
   :language: python
   :linenos:

.. image:: _static/themes.png
   :alt: themes screenshot

Center elements vertically
-----------------------------

.. literalinclude:: ../guides/centerhor.py
   :language: python
   :linenos:

.. image:: _static/centerhor.png
   :alt: vertically centered label screenshot

Center elements horizontally 
-----------------------------

At this moment you can't. Sorry
