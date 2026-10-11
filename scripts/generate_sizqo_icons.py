#!/usr/bin/env python3
"""Generate Sizqo's independent app icon assets with offline vector drawing."""

from pathlib import Path
import subprocess
import tempfile

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parents[1]
ICON_DIR = ROOT / "src-tauri" / "icons"
SIZE = 1024
SCALE = 4
CANVAS = SIZE * SCALE


def scaled_point(point: tuple[int, int]) -> tuple[int, int]:
    return (point[0] * SCALE, point[1] * SCALE)


def draw_icon() -> Image.Image:
    image = Image.new("RGB", (CANVAS, CANVAS), "#063E46")
    draw = ImageDraw.Draw(image)

    # Two inward brackets form a compression frame. Their ends sit well inside
    # the system mask safe area; the OS supplies the final rounded icon shape.
    mint = "#85E4C5"
    bracket_width = 29 * SCALE
    for x, direction in ((205, 1), (819, -1)):
        outer = x
        inner = x + direction * 72
        for y in (350, 674):
            draw.line(
                [scaled_point((outer, y)), scaled_point((inner, y))],
                fill=mint,
                width=bracket_width,
            )
        draw.line(
            [scaled_point((outer, 350)), scaled_point((outer, 674))],
            fill=mint,
            width=bracket_width,
        )

    # A compact Q keeps the mark legible at Dock and Finder sizes.
    q_color = "#F6F5EC"
    stroke = 62 * SCALE
    draw.ellipse(
        [scaled_point((350, 252)), scaled_point((674, 576))],
        outline=q_color,
        width=stroke,
    )
    draw.line(
        [scaled_point((570, 552)), scaled_point((720, 702))],
        fill=q_color,
        width=stroke,
    )

    # A small contrasting core makes the Q distinctive without a second badge.
    draw.ellipse(
        [scaled_point((473, 375)), scaled_point((551, 453))],
        fill="#F0B78B",
    )
    return image.resize((SIZE, SIZE), Image.Resampling.LANCZOS).convert("RGBA")


def write_iconset(image: Image.Image, iconset: Path) -> None:
    iconset.mkdir(parents=True, exist_ok=True)
    for points, suffix in (
        (16, "icon_16x16.png"),
        (32, "icon_16x16@2x.png"),
        (32, "icon_32x32.png"),
        (64, "icon_32x32@2x.png"),
        (128, "icon_128x128.png"),
        (256, "icon_128x128@2x.png"),
        (256, "icon_256x256.png"),
        (512, "icon_256x256@2x.png"),
        (512, "icon_512x512.png"),
        (1024, "icon_512x512@2x.png"),
    ):
        image.resize((points, points), Image.Resampling.LANCZOS).save(iconset / suffix)


def main() -> None:
    ICON_DIR.mkdir(parents=True, exist_ok=True)
    icon = draw_icon()
    icon.save(ICON_DIR / "icon.png")
    with tempfile.TemporaryDirectory(prefix="sizqo-iconset-") as temporary:
        iconset = Path(temporary) / "Sizqo.iconset"
        write_iconset(icon, iconset)
        subprocess.run(
            ["iconutil", "-c", "icns", "-o", str(ICON_DIR / "icon.icns"), str(iconset)],
            check=True,
        )
    icon.save(
        ICON_DIR / "icon.ico",
        format="ICO",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )
    for filename, size in (
        ("32x32.png", 32),
        ("64x64.png", 64),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44),
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),
    ):
        icon.resize((size, size), Image.Resampling.LANCZOS).save(ICON_DIR / filename)
    print(f"Generated Sizqo icon assets in {ICON_DIR}")


if __name__ == "__main__":
    main()
