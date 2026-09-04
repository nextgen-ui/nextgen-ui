#!/usr/bin/env python3
"""Screenshot the app window without focusing it or moving the pointer.

    python3 tools/grab_window.py out.png

Use this rather than `gnome-screenshot -w`, which captures whatever is focused
and will silently grab the user's own window instead. Selection is by WM_CLASS,
not title: a file manager showing the project directory is also titled
"nextgen-ui" and will otherwise be picked up.

Requires python3-xlib and Pillow, both already present on this machine.
"""
import subprocess, sys
from Xlib import display, X
from PIL import Image

WM_CLASS_PREFIX = "nextgen-ui."


def find_window():
    out = subprocess.run(["wmctrl", "-lx"], capture_output=True, text=True).stdout
    for line in out.splitlines():
        cols = line.split(None, 4)
        if len(cols) >= 3 and cols[2].startswith(WM_CLASS_PREFIX):
            return int(cols[0], 16)
    raise SystemExit("app window not found — is it running?")


def grab(wid, path):
    d = display.Display()
    w = d.create_resource_object("window", wid)
    g = w.get_geometry()
    raw = w.get_image(0, 0, g.width, g.height, X.ZPixmap, 0xFFFFFFFF)
    Image.frombytes("RGB", (g.width, g.height), raw.data, "raw", "BGRX").save(path)
    return g.width, g.height


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "window.png"
    size = grab(find_window(), out)
    print(f"wrote {out} {size[0]}x{size[1]}")
