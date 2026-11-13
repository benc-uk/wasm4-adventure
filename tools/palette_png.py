#!/usr/bin/env python3
"""Batch-export opaque, four-color, 2-bit indexed PNGs."""

import argparse
from pathlib import Path
import re
import sys
from tempfile import TemporaryDirectory
from typing import Literal

from PIL import Image


def background_color(value: str) -> tuple[int, int, int] | Literal["lightest"]:
    if value.lower() == "lightest":
        return "lightest"
    if not re.fullmatch(r"#?[0-9a-fA-F]{6}", value):
        raise argparse.ArgumentTypeError(
            "background must be 'lightest' or a hex color, e.g. '#20283d'"
        )
    value = value.removeprefix("#")
    return tuple(int(value[index:index + 2], 16) for index in (0, 2, 4))


def extract_colors(image: Image.Image, count: int) -> list[tuple[int, int, int]]:
    indexed = image.quantize(colors=count, method=Image.Quantize.MEDIANCUT)
    palette = indexed.getpalette()
    used = indexed.getcolors(maxcolors=count)
    if palette is None or used is None:
        raise ValueError("could not extract the image palette")
    return [
        tuple(palette[index * 3:index * 3 + 3])
        for _, index in sorted(used, key=lambda item: item[1])
    ]


def convert_png(
    source: Path, background: tuple[int, int, int] | Literal["lightest"]
) -> Image.Image:
    with Image.open(source) as image:
        if image.format != "PNG":
            raise ValueError("input must be a PNG")
        if getattr(image, "n_frames", 1) != 1:
            raise ValueError("animated PNGs are not supported")
        rgba = image.convert("RGBA")

    if background == "lightest":
        pixels = rgba.tobytes()
        visible_colors = (
            (pixels[index], pixels[index + 1], pixels[index + 2])
            for index in range(0, len(pixels), 4)
            if pixels[index + 3] != 0
        )
        lightest = max(
            visible_colors,
            key=lambda color: 299 * color[0] + 587 * color[1] + 114 * color[2],
            default=None,
        )
        if lightest is None:
            raise ValueError(
                "cannot select the lightest color from a fully transparent image; "
                "supply a hex --background"
            )
        background = lightest

    alpha = rgba.getchannel("A")
    has_transparency = alpha.getextrema()[0] < 255
    flattened = Image.alpha_composite(
        Image.new("RGBA", rgba.size, (*background, 255)), rgba
    ).convert("RGB")

    if has_transparency:
        # Hidden RGB values must not consume any of the four palette slots.
        rgb = flattened.tobytes()
        visible = bytearray()
        for index, opacity in enumerate(alpha.tobytes()):
            if opacity:
                visible.extend(rgb[index * 3:index * 3 + 3])
        if visible:
            sample = Image.frombytes("RGB", (len(visible) // 3, 1), bytes(visible))
            colors = extract_colors(sample, 4)
            if background not in colors and len(colors) == 4:
                colors = extract_colors(sample, 3)
            colors = [background] + [color for color in colors if color != background]
        else:
            colors = [background]
    else:
        colors = extract_colors(flattened, 4)

    # Pillow quantizes against 256 entries; padding must not introduce new colors.
    padded = colors + [colors[-1]] * (256 - len(colors))
    palette_image = Image.new("P", (1, 1))
    palette_image.putpalette([component for color in padded for component in color])
    result = flattened.quantize(palette=palette_image, dither=Image.Dither.NONE)
    result = result.point([colors.index(color) for color in padded])
    if has_transparency:
        result.paste(0, mask=alpha.point(lambda opacity: 255 if opacity == 0 else 0))
    output_palette = colors + [colors[-1]] * (4 - len(colors))
    result.putpalette([component for color in output_palette for component in color])
    result.info.clear()
    return result


def collect_jobs(
    inputs: list[Path], output_dir: Path, overwrite: bool
) -> list[tuple[Path, Path]]:
    jobs = {}
    for source in inputs:
        if source.is_dir():
            sources = sorted(
                path for path in source.iterdir()
                if path.is_file() and path.suffix.lower() == ".png"
            )
            if not sources:
                raise ValueError(f"no PNG files in {source}")
        elif source.is_file() and source.suffix.lower() == ".png":
            sources = [source]
        else:
            raise ValueError(f"not a PNG file or directory: {source}")

        for path in sources:
            destination = output_dir / f"{path.stem}.png"
            resolved = destination.resolve()
            if resolved == path.resolve():
                raise ValueError(f"output would overwrite the source: {path}")
            if resolved in jobs:
                if jobs[resolved][0].resolve() == path.resolve():
                    continue
                raise ValueError(f"multiple inputs have the output name {destination.name}")
            if destination.exists() and not overwrite:
                raise ValueError(f"output exists: {destination}; use --overwrite to replace it")
            jobs[resolved] = (path, destination)
    return list(jobs.values())


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", nargs="+", type=Path, help="PNG files or directories")
    parser.add_argument(
        "-o", "--output-dir", required=True, type=Path, help="directory for converted PNGs"
    )
    parser.add_argument(
        "--background", type=background_color, default="#000000",
        help="'lightest' visible source color, or RRGGBB / #RRGGBB (default: #000000)"
    )
    parser.add_argument("--overwrite", action="store_true", help="replace existing outputs")
    args = parser.parse_args(argv)

    try:
        jobs = collect_jobs(args.inputs, args.output_dir, args.overwrite)
        args.output_dir.mkdir(parents=True, exist_ok=True)
    except (OSError, ValueError) as error:
        parser.error(str(error))

    failures = 0
    for source, destination in jobs:
        try:
            result = convert_png(source, args.background)
            with TemporaryDirectory(prefix=".palette-", dir=args.output_dir) as temporary:
                staged = Path(temporary) / destination.name
                result.save(staged, format="PNG", bits=2, optimize=True)
                if destination.exists() and not args.overwrite:
                    raise FileExistsError(f"output exists: {destination}")
                staged.replace(destination)
            print(f"{source} -> {destination}")
        except (OSError, ValueError, Image.DecompressionBombError) as error:
            print(f"{source}: {error}", file=sys.stderr)
            failures += 1
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
