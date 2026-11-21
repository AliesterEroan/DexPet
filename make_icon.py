#!/usr/bin/env python
"""Draw the exe's icon from the page's own Dex and write src-tauri/icons/icon.{png,ico}.

Dex is CSS - one <template> of spans sized in em - so the honest way to get him into a
Windows icon is to render the real page and photograph the result. This script runs
capture_dex.js for that, crops the mascot out of the frame, squares him on a transparent
field and saves the two files tauri.conf.json names.

    python make_icon.py                # renders the page, crops, writes both files
    python make_icon.py --check        # renders, measures, reports; writes nothing
    python make_icon.py --from art.png # packages an existing render; no browser involved

--from is for art that has been cleaned up by hand after a capture. It still requires a
transparent field, because an opaque rectangle is exactly the mistake it exists to avoid.

Requires Node (for Edge over CDP) and Pillow. No network - the page is already offline.
Change Dex's markup, then re-run this; nothing here redraws him by hand.
"""

import json
import os
import subprocess
import sys
import tempfile

try:
    from PIL import Image, UnidentifiedImageError
except ImportError:
    sys.exit("REFUSED: Pillow is not installed - this script packages the icon with it.")

HERE = os.path.dirname(os.path.abspath(__file__))
OUT_DIR = os.path.join(HERE, "src-tauri", "icons")
SIZES = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
MARGIN = 0.06          # of the mascot's own width, so the duotone shadow is not clipped


def fail(message):
    print("REFUSED: " + message)
    sys.exit(1)


def square_and_size(dex):
    side = max(dex.size)
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.paste(dex, ((side - dex.size[0]) // 2, (side - dex.size[1]) // 2), dex)
    return square.resize((256, 256), Image.LANCZOS)


def from_capture():
    work = tempfile.mkdtemp(prefix="navicon_")
    png = os.path.join(work, "dex.png")
    spec = os.path.join(work, "dex.json")
    try:
        run = subprocess.run(["node", os.path.join(HERE, "capture_dex.js"), png, spec],
                             capture_output=True, text=True)
        if run.returncode != 0:
            fail("capture_dex.js said: " + (run.stderr or run.stdout).strip()[-400:])
        try:
            rect = json.loads(open(spec, encoding="utf-8").read())
        except (ValueError, KeyError) as why:
            fail("could not read the measurement from capture_dex.js: %r" % (why,))

        scale = rect.get("deviceScaleFactor", 2)
        pad = max(4, int(rect["w"] * MARGIN * scale))
        shadow = int(rect["shadow"] * scale)
        box = (int(rect["x"] * scale) - pad, int(rect["y"] * scale) - pad,
               int((rect["x"] + rect["w"]) * scale) + pad + shadow,
               int((rect["y"] + rect["h"]) * scale) + pad + shadow)
        try:
            frame = Image.open(png).convert("RGBA")
        except (UnidentifiedImageError, OSError) as why:
            fail("the screenshot is not a readable PNG: %r" % (why,))
        if frame.width < box[2] or frame.height < box[3]:
            fail("the mascot box %s does not fit the %s frame - raise VIEW in capture_dex.js"
                 % (box, frame.size))
        return (square_and_size(frame.crop(box)),
                "captured %s from %s at %s, mascot %.0f x %.0f css px, crop %s"
                % (png, frame.size, rect.get("em"), rect["w"], rect["h"], box))
    finally:
        for path in (png, spec):
            if os.path.isfile(path):
                os.remove(path)
        os.rmdir(work)


def from_file(source):
    try:
        art = Image.open(source)
    except (UnidentifiedImageError, OSError) as why:
        fail("cannot read %s: %r" % (source, why))
    if art.mode != "RGBA":
        fail("%s is %s, not RGBA - there is no alpha channel for the icon field to show through"
             % (source, art.mode))
    corners = [art.getpixel(p) for p in
               ((0, 0), (art.width - 1, 0), (0, art.height - 1), (art.width - 1, art.height - 1))]
    if any(px[3] for px in corners):
        fail("the corners of %s are %s - a background is still behind Dex" % (source, corners))
    return square_and_size(art), "packaged %s at %s" % (source, art.size)


def main():
    check_only = "--check" in sys.argv
    if "--from" in sys.argv:
        rest = [a for a in sys.argv[1:] if a != "--check"]
        if len(rest) != 2 or not os.path.isfile(rest[1]):
            fail("--from needs the path of an existing PNG: python make_icon.py --from art.png")
        icon, note = from_file(rest[1])
    else:
        icon, note = from_capture()

    hist = icon.split()[3].histogram()
    pct = 100.0 * sum(hist[:8]) / (256 * 256)
    print(note)
    print("packaged  256 x 256 RGBA, %.1f%% transparent" % pct)
    if pct < 5:
        print("WARNING: almost no transparency - the page background was probably photographed.")
    if check_only:
        print("--check: nothing written.")
        return

    os.makedirs(OUT_DIR, exist_ok=True)
    icon.save(os.path.join(OUT_DIR, "icon.png"))
    icon.save(os.path.join(OUT_DIR, "icon.ico"), sizes=SIZES)
    for name in ("icon.png", "icon.ico"):
        target = os.path.join(OUT_DIR, name)
        print("wrote     %s  %s B" % (target, "{:,}".format(os.path.getsize(target))))


if __name__ == "__main__":
    main()
