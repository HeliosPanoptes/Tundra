Once a web browser has downloaded a web page, it has to show that web page to the user. Since we are not savages,[1] browsers use a graphical user interface to draw web pages in a mix of fonts, colors, and styles. How does it do that? Well, in short, it talks to the operating system to create a *window*, walks through the page HTML, and draws text on that window while changing styles as the HTML tags dictate.

## Creating windows

On desktop and laptop computer, users run operating systems that provide *desktop environments* which contain windows, icons, menus, and a pointer.[2] This desktop environment provided by some component of the operating system, which handles important jobs like keeping track of the pointer and where the windows are, talking to applications to get windows contents and tell them about clicks, and pushing pixels to the screen.

In order to draw anything on the screen, a program has to communicate with this operating system component. This communication usually involves:

- Asking the OS to allocate space for a new window and track it
- Keeping track of some kind of identifier for this window
- Acting on messages from the OS about keyboard and mouse event
- Redrawing the window contents periodically[3]

Since doing all of this by hand is a bit of a drag, this is usually wrapped up in libraries called *graphical toolkits*. We'll be using a rust wrapper of the gtk library, specifically focusing on Cairo, their drawing package. To install the base library on Ubuntu: run `sudo apt-get install libgtk-3-dev`. On macOS: run `brew install gtk+3`.

Add this to your Cargo.toml:

```
[dependencies]
gtk = "0.7.0"
```

When building, if it spouts an error saying that libffi could not be found, it might be a homebrew issue that can be worked around with

```shell
PKG_CONFIG_PATH="/usr/local/opt/libffi/lib/pkgconfig" cargo build
```

If you're using an IDE like the Rust plugin for CLion, it's easiest to build the whole project once using the command line, then the IDE's build will work properly.

```
extern crate cairo; // Drawing
extern crate gio;   // IO
extern crate gtk;   // Application/Window


use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;


import tkinter
window = tkinter.Tk()
tkinter.mainloop()
```

Here, after importing the library, we call `tkinter.Tk()` to communicate with the OS in order to create a window on the screen. The OS responds with an identifier for that window that we can use in future communication with the OS. That identifier is stored inside the `Tk` object that we assign to `window`.

Then, the final line starts the *main loop*. This is an important and general pattern for all graphical applications, from web browsers to video games. The main loop internally looks like this:[5]

```
while True:
    for evt in pendingEvents():
        handleEvent(evt)
    drawScreen()
```

Our simple window above does not need to do a lot of event handling (it ignores all events) and it does not need to do a lot of drawing (on my computer it is a uniform gray). But when graphical applications get more complex having a main loop is a good way to make sure that all events are eventually handled and the screen is eventually updated, which is essential to a good user experience.

## Drawing to the window

A graphical application extends the `handleEvent` and `drawScreen` functions to draw interesting stuff on the screen and react when the user clicks on that stuff. Let's start by drawing some text on the screen.

We are going to draw text on the screen using a *canvas*,[6] a rectangular region of the window that we can draw circles, lines, and text in. Tk also has higher-level abstractions like buttons and dialog boxes. While these abstractions are useful for many application, we won't be using them: web pages have a lot of control over how they should look, control Tk's higher-level abstractions don't provide. (This is why desktop applications look much more uniform than web pages do—desktop applications are generally written using the abstractions provided by an operating system's most common graphical toolkit, which limit their creative possibilities.)

To create a canvas in Tk, we insert the following code between the `tkinter.Tk()` call and the `tkinter.mainloop()` call:

```
canvas = tkinter.Canvas(window, width=800, height=600)
canvas.pack()
```

The first line creates a `Canvas` object inside the `window` we already created. We pass it some arguments that define its size; I chose 800×600 because that was a common old-timey monitor size.[7] The second line is something particular to Tk, which requires us to call `pack` on all *widgets*like canvases to position them inside their parent (the `window`).

Adding these two lines won't yet change how the window appears, since we haven't drawn anything to the canvas. To do that, you can call methods on the `canvas` whose names begin with `create_`:

```
canvas.create_rectangle(10, 20, 400, 300)
canvas.create_oval(100, 100, 150, 150)
canvas.create_text(200, 150, text="Hi!")
```

You ought to see a rectangle, starting near the top-left corner of the canvas and ending at its center, then a circle inside that rectangle, and then the text “Hi!” next to the circle.

Play with some of the arguments to those methods—which coordinate does each number refer to? Check that you got it right against [online documentation](http://infohost.nmt.edu/tcc/help/pubs/tkinter/web/canvas.html). It is important to remember that coordinates in Tk, like (10, 20), refer first to X position from left to right and then to Y position from top to bottom. This means that lower on the screen has a *larger*Y value, the opposite of what you might be used to from math.

## Laying out text

Now that we've got a basic GUI window and can draw into it, let's start laying out a simple web page.

Remember that in the last post, we implemented a simple function that stepped through the web page source code character by character and printed the text (but not the tags) to the console window. We now want to do the same thing, but to print the characters to our GUI instead.

To start, let's change the `show`function from the previous post into a function that I'll call `lex`[8](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.8)which just *returns*the text-not-tags content of an HTML document:

```
def lex(body):
  text = ""
  # ...
  for c in body:
      # ...
      elif not in_angle:
          text += c
    return text
```

Now, let's refactor `show`to output to our window instead. `show`will have to start by creating the window and canvas, like above, and then stepping through the text character by character, drawing it to the screen:

```
def show(text):
    # set up window, canvas
    for c in text:
        canvas.create_text(100, 100, text=c)
    tkinter.mainloop()
```

Let's apply this code to a real webpage, and for reasons that might seem inscrutible[9](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.9), we're going to start with a web page in Chinese: the [beginning of 西游记 or "Journey to the West"](http://www.zggdwx.com/xiyou/1.html), a classic Chinese novel about a monkey.[10](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.10)To handle this web page, you're going to need to go back to the `request`function you wrote last time and change it to encode and decode the text not using the `ascii`codec but using the `utf8`codec (which handles Chinese text).[11](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.11)You'll also find that this website response in HTTP 1.1 even when you make an HTTP 1.0 request.[12](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.12)You should change your `request`function to compensate—either ignore the error, or make the request with HTTP 1.1, passing the `Connection: close`header to close the connection once you're done.

If you run the URL above through `parse`, `request`,[13](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.13)`lex`, and `show`, you should see a window with a big rectangle of black ink roughly 100 pixels from the top left corner of the window. Why is it a blob of ink? Well, of course, because we are drawing every letter to the same part of the screen! Let's fix that:

```
x, y = 13, 13
for c in text:
    canvas.create_text(x, y, text=c)
    x += 13
```

Now, the characters should form a nice line from left to right, so that you can see each individual character. But now the problem is that with an 800 pixel wide canvas and 13 pixels per character, you can only fit about 60 characters. You'll need more than that to read a novel!

The solution is to *wrap*the text once we reach the edge of the screen:[14](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.14)

```
x, y = 13, 13
for c in text:
    canvas.create_text(x, y, text=c)
    x += 13
    if x >= 787:
        y += 18
        x = 13
```

Here, when we get past pixel 787 to the right[15](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.15)we increase *y*and reset *x*to the left hand side again. This moves us down a line and makes it possible to see all of the text. Also, note that I've got some magic numbers here: 13 and 18. I'll reveal where they come from in the next post.

## Scrolling text

Now we can read several paragraphs of text, but there's still a problem. But if there's enough text, all of the lines of text don't fit on the screen, and there's still content you can't read. Every browser solves this problem by allowing the user to *scroll*the page and look at different parts of it.

Scrolling introduces a layer of indirection between page coordinates (this text is 132 pixels from the top of the *page*) and screen coordinates (this text is 72 pixels from the top of the *screen*). Generally speaking, a browser *lays out*the page in terms of page coordinates—determines where everything on the page goes—and then *renders*the page in terms of screen coordinates.

Let's introduce the same split in our browser. Right now we have a `show`function that takes in text and then creates a graphical window, computes the position of each character, and then draws that character. Let's split it into a `layout`function that just computes the position of each bit of text, and a `render`function that creates the window and draws each bit of text on the screen. Only `render`needs to think about screen coordinates, while `layout`can operate on page coordinates alone.

What should the interface between these two functions be? Well, `render`only needs to know which character to place where, so what about having `layout`just returning a list of tuples: the character to draw and its `x`and `y`coordinates? Then `render`could loop through that list and draw each:

```
display_list = layout(text)
for x, y, c in display_list:
  canvas.create_text(x, y, text=c)
```

I am calling this list that `layout`returns a *display list*, since it is a list of things to display; the term is standard. Creating that list is easy. Right now, we loop over each token, and loop over every word in that token, and call `canvas.create_text`. Now instead of calling `canvas.create_text`, we add it to a list:

```
display_list = []
for c in text:
    display_list.append((x, y, c))
    # ...
return display_list
```

Now if we want to scroll the whole page by, say, 100 pixels, we can change the `create_text`parameter from `y`to `y - 100`. More generally, let's add a `scrolly`state variable and subtract that from the `y`position when we render text:

```
scrolly = 0
for x, y, c in display_list:
  canvas.create_text(x, y - scrolly, text=c)
```

If you change the value of `scrolly`the page will scroll up and down. So how do we change the value of `scrolly`?

## Reacting to keyboard input

Most browsers scroll the page when you press the up and down keys, rotate the scroll wheel, or drag the scroll bar. Let's keep things simple and implement the first of those.

Tk allows you to *bind*certain keyboard buttons, and call a specific function when then that key is pressed. For example, to call the `scrolldown`function when the "Down" button is pressed, we write:

```
window.bind("<Down>", scrolldown)
```

Note that I wrote `scrolldown`, not `scrolldown()`: I'm not calling the function, I'm just writing its name. Tk will call the function, when the user presses the "Down" button. To implement `scrolldown`, we need to increment `y`and then re-draw the canvas:

```
SCROLL_STEP = 100
scrolly = 0

def render():
    for x, y, c in display_list:
        canvas.create_text(x, y - scrolly, text=c)

def scrolldown(e):
    nonlocal scrolly
    scrolly += SCROLL_STEP
    render()

render()
```

There are some pretty big changes here. First, I've moved the loop that draws all the text into a function, `render`. That function is called immediately when the page is first rendered (last line above). But it is also called when you scroll down, so that the page can be redrawn.[16](http://pavpanchekha.com/blog/emberfox/graphics.html#fn.16)

If you try this out, you'll find that scrolling causes all the text to be drawn twice. That's because we didn't erase the old text when we started drawing the new text. To do that, we call `canvas.delete`:

```
canvas.delete('all')
```

## Summary

The last post build a simple, purely-command-line browser. Now we've significantly upgraded it by introducing a rudimentary graphical user interface, which can:

- Create a graphical window
- Lay out text in lines
- Scroll so you can see all of the text

The code in this post describes modifications atop the code from the last post. Implement those modifications. You should be able to call your browser from the command line with a URL, and see the first line of text from that page in a graphical window. If that line contains any bold or italic text, you should see that displayed correctly.

So far, I've asked you to try this out on a Chinese web page. If you try it out on an English web page, you'll find that there are some strange bits. For one, the text looks really weird, with all of the characters spaced far apart. For another, lines break in the middle of words. And, there's no special handling for paragraphs, links, or any kind of formatting. We'll fix these problems in the next post.

## Exercises

- Look through [the options](http://effbot.org/tkinterbook/canvas.htm#Tkinter.Canvas.config-method)you can pass to the `Canvas`constructor. Change the canvas to have a white background and give it a red border (which Tk calls a highlight). (This will help you see where the edge of the canvas is.)
- Add support for scrolling up as well as down (when you hit the up arrow). Remember that you shouldn't be able to scroll above the original top of the page.
- Change `layout`so that it handles newlines by ending the line and starting a new one. Better yet, increment *y*by more than when you just break lines, to give the illusion of paragraph breaks. Line breaks will make it easy to read the poems in "Journey to the West".
- Change the `render`function to not render characters outside the 800×600 window. This avoids wasting time doing work that the user can't see. Make sure you use screen, not page, coordinates to determine which caracters to draw! Depending on your OS, this should make a difference for "Journey to the West".
- Make browser resizable. To do so, pass the `fill`and `expand`arguments to `canvas.pack`call and bind to the `<Configure>`event to run code when the window is resized. You can get the new window width and height with `e.width`and `e.height`. When the window is resized, the line breaking behavior will have to change, so you will need to call `layout`again.