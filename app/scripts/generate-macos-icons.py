#!/usr/bin/env python3
from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw
from PIL import ImageChops


def _root_dir() -> Path:
    return Path(__file__).resolve().parent.parent


def _assets_dir() -> Path:
    return _root_dir() / "assets"


def _rounded_rect(draw: ImageDraw.ImageDraw, xy, radius: int, fill):
    draw.rounded_rectangle(xy, radius=radius, fill=fill)


def _draw_logo(size: int, *, primary=(0x16, 0x77, 0xFF, 0xFF), fg=(0xFF, 0xFF, 0xFF, 0xFF)) -> Image.Image:
    s = size
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    radius = round(s * 0.22)
    inset = round(s * 0.06)

    # Base background
    _rounded_rect(draw, (inset, inset, s - inset, s - inset), radius=radius, fill=primary)

    # Subtle top highlight gradient (masked to avoid white halo on transparent edges)
    mask = Image.new("L", (s, s), 0)
    md = ImageDraw.Draw(mask)
    _rounded_rect(md, (inset, inset, s - inset, s - inset), radius=radius, fill=255)

    grad = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    gd = ImageDraw.Draw(grad)
    for y in range(s):
        # More highlight at top; fade to transparent
        t = max(0.0, 1.0 - (y / (s * 0.95)))
        a = int(40 * (t**2))
        gd.line([(0, y), (s, y)], fill=(255, 255, 255, a))

    r, g, b, a = grad.split()
    a = ImageChops.multiply(a, mask)
    grad = Image.merge("RGBA", (r, g, b, a))

    img = Image.alpha_composite(img, grad)
    draw = ImageDraw.Draw(img)

    stroke = max(2, round(s * 0.10))
    pad = round(s * 0.22)
    top_y = pad
    mid_y = pad + round(s * 0.18)
    base_y = pad + round(s * 0.52)
    left_x = pad
    bar_w = stroke
    bar_h = base_y - top_y + stroke

    def rect(x0: int, y0: int, w: int, h: int):
        draw.rectangle([x0, y0, x0 + w, y0 + h], fill=fg)

    # F vertical + top + mid
    rect(left_x, top_y, bar_w, bar_h)
    rect(left_x, top_y, round(s * 0.44), stroke)
    rect(left_x, mid_y, round(s * 0.34), stroke)

    # "1" "1"
    one_x1 = round(s * 0.62)
    one_x2 = round(s * 0.76)
    rect(one_x1, top_y, bar_w, bar_h)
    rect(one_x2, top_y, bar_w, bar_h)

    # Small sync dot accent
    dot_r = round(s * 0.055)
    cx = round(s * 0.78)
    cy = round(s * 0.77)
    draw.ellipse([cx - dot_r, cy - dot_r, cx + dot_r, cy + dot_r], fill=(0x40, 0x93, 0xFF, 0xFF))

    return img


def _write_iconset(base_png: Path, iconset_dir: Path):
    iconset_dir.mkdir(parents=True, exist_ok=True)
    base = Image.open(base_png).convert("RGBA")

    # Apple iconset filenames (spotlight, app, etc.)
    sizes = [16, 32, 128, 256, 512]
    for sz in sizes:
        base.resize((sz, sz), Image.Resampling.LANCZOS).save(iconset_dir / f"icon_{sz}x{sz}.png")
        base.resize((sz * 2, sz * 2), Image.Resampling.LANCZOS).save(
            iconset_dir / f"icon_{sz}x{sz}@2x.png"
        )

    # 1024 = 512@2x
    base.resize((1024, 1024), Image.Resampling.LANCZOS).save(iconset_dir / "icon_512x512@2x.png")


def main() -> int:
    assets = _assets_dir()
    assets.mkdir(parents=True, exist_ok=True)

    icon_png = assets / "icon.png"
    iconset = assets / "icon.iconset"
    icon_icns = assets / "icon.icns"
    icon_ico = assets / "icon.ico"

    tray_png = assets / "trayTemplate.png"
    tray_png_2x = assets / "trayTemplate@2x.png"

    # App/Dock icon source
    _draw_logo(1024).save(icon_png)

    if iconset.exists():
        shutil.rmtree(iconset)
    _write_iconset(icon_png, iconset)

    subprocess.check_call(["iconutil", "-c", "icns", str(iconset), "-o", str(icon_icns)])

    # Windows .ico (multi-size)
    base = Image.open(icon_png).convert("RGBA")
    base.save(icon_ico, format="ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])

    # Tray template icons (monochrome; system can tint if used as template)
    _draw_logo(36, primary=(0, 0, 0, 0), fg=(0, 0, 0, 255)).save(tray_png_2x)
    Image.open(tray_png_2x).resize((18, 18), Image.Resampling.LANCZOS).save(tray_png)

    shutil.rmtree(iconset, ignore_errors=True)
    print(f"OK: {icon_icns}")
    print(f"OK: {icon_ico}")
    print(f"OK: {tray_png}")
    print(f"OK: {tray_png_2x}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
