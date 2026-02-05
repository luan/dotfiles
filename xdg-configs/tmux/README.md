# Luan's Tmux Config

This config aims to make tmux comfortable for vim users. Below are
some highlights and possible workflows.

**Notation**: We'll use `C-spc` to mean "hold control, and tap the spacebar".

## Commands that work "normally"

Most of the time in tmux, you're in normal mode. When you type things, they go
into your shell, and your shell can run them. You can also run the following
commands.

### Managing "windows"
- `C-spc c` -- create a new window
- `C-spc 1` -- switch to window 1
- ...
- `C-spc 9` -- switch to window 9
- `C-spc C-spc` -- switch to most recent other window
- `C-spc ,` -- rename this window
- To close a window, close all the panes in it. Windows start with one pane. See below.

### Managing "panes"
- `C-spc |` -- split window vertically (creates a new "pane")
- `C-spc -` -- split window vertically (creates a new "pane")
- `C-spc h` -- switch keyboard focus to the pane to the left
- `C-spc j` -- switch keyboard focus to the pane below
- `C-spc k` -- switch keyboard focus to the pane above
- `C-spc l` -- switch keyboard focus to the pane to the right
- `C-spc z` -- Zoom!
  + If you can see multiple panes, this will "zoom in" on the current pane.
  + If you're already "zoomed in", this will zoom out, so you can see multiple
    panes again.
- Panes are just regular subshells running your usual shell. If you're running `bash` you can close them with `exit` or `C-d`.

### Clipboard
- `C-spc [` -- enter movement mode
- `C-spc ]` -- paste text that you previously copied in movement mode

## Commands that work in movement mode

Movement mode is like being in vim. You can move around using vim movement keys.
You can highlight things. You can copy things to a clipboard, for pasting later
(in normal mode).

### Moving around
- `h` -- go left
- `j` -- go down
- `k` -- go up
- `l` -- go right
- `0` -- go to the beginning of the line
- `$` -- go to the end of the line
- `w` -- go foreword by one Word
- `b` -- go Backward by one word
- `fx` -- go Foreword until you hit the next `x` character. Also works for any
          other character instead of `x`.
- `Fx` -- go backward until you hit the next `x` character. Also works for any
          other character instead of `x`.
- `tx` -- go foreword unTil just before you hit the next `x` character. Also
          works for any other character instead of `x`.
- `Tx` -- go backward unTil just before you hit the next `x` character. Also
          works for any other character instead of `x`.

### Searching
- `/` -- search forwards
- `?` -- search backwards
- `n` -- jump to the next thing that matches your last search (in whatever
         direction you were already searching)
- `N` -- jump to the previous thing that matches your last search (in whatever
         direction you were already searching)

### Clipboard
- `v` -- start highlighting character-by-character (you can continue to highlight by moving around)
- `V` -- start highlighting line-by-line (you can continue to highlight by moving around)
- `y` -- copy whatever is highlighted to the clipboard (so you can paste later
         in normal mode). This also puts you immediately back into normal mode.

### Stopping
- `ESC` stop highlighting or searching
- `q` stop being in movement mode -- go back to normal mode.

## Common workflows

### The copy-paste flow

I'm in tmux, and I have a single terminal in a single pane in a single window.

I have a list of handy commands in a file called `handy-bash-commands.txt`. I
want to use one of them. I'm going to cat the file, copy the appropriate command
to clipboard, paste it into my shell, and see the results of my cool command.
Let's break that into steps:

First I cat the file:

```
$ cat handy-bash-commands.txt

handy-command 1
handy-command 2
super cool command
sudo super cool command
another command
more stuff

$
```

Now my cursor is at the shell prompt as I would expect. I hit `C-spc [` to get
into movement mode, then hit `kkkk0` to move my cursor to the beginning of the
line that reads `sudo super cool command`. To highlight the whole line, I hit
`V`. To copy it to clipboard, I hit `y`. This also kicks me back into normal
mode, with my cursor at the shell prompt again. To paste and run the command, I
hit `C-spc ]`. Note that the reason the command runs as soon as I paste is is
because I copied a newline to clipboard when I highlighted and copied the whole
line earlier.

Here's the result:


```
$ cat handy-bash-commands.txt

handy-command 1
handy-command 2
super cool command
sudo super cool command
another command
more stuff

$ sudo super cool command

SUPER COOL OUTPUT!

$
```

Now I want to run something similar to `handy-command 2`, but with a slight
difference.

My cursor is at the shell prompt as I would expect. I hit `C-spc [` to get
into movement mode, then hit `?handy<ENTER>` to move my cursor to the beginning of the
line that reads `handy-command 2`. I'm only interested in the beginning of this
command, so I hit `v` to start highlighting character-by-character. I hit `ww`
to highlight the words `handy-command`. I hit `y` to copy those words to the
clipboard. This also kicks me back into normal mode, with my cursor at the shell
prompt again. To paste the command, I hit `C-spc ]`. Because I didn't copy any
newlines, the command doesn't run immediately, and I can edit it.

Here's what my terminal looks like now:

```
$ cat handy-bash-commands.txt

handy-command 1
handy-command 2
super cool command
sudo super cool command
another command
more stuff

$ sudo super cool command

SUPER COOL OUTPUT!

$ handy-command 
```

Now I'm free to complete the command as I wish, and run it as normal:

```
$ cat handy-bash-commands.txt

handy-command 1
handy-command 2
super cool command
sudo super cool command
another command
more stuff

$ sudo super cool command

SUPER COOL OUTPUT!

$ handy-command 65537

Wow, that's a really cool number. Are you a big fan of regular polygons?

$
```
