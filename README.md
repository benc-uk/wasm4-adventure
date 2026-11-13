# hello-world

A game written in Rust for the [WASM-4](https://wasm4.org) fantasy console.

## Building

Build the cart by running:

```shell
cargo build --release
```

The cartridge imports WASM-4's host functions and fixed 64 KiB memory from the
`env` module, so it needs a WASM-4 runtime. Run it with:

```shell
w4 run target/wasm32-unknown-unknown/release/cart.wasm
```

For development, use `cargo build` and run
`w4 run target/wasm32-unknown-unknown/debug/cart.wasm`.
The development profile disables debug symbols to keep the cartridge below
WASM-4's 65,536-byte size limit. If the warning returns as the game grows, use the
optimized release build above.

For more info about setting up WASM-4, see the [quickstart guide](https://wasm4.org/docs/getting-started/setup?code-lang=rust#quickstart).

## Custom 6x8 font

[src/font.rs](./src/font.rs) contains the Rust conversion of
[src/font.h](./src/font.h). Its 128 ASCII glyphs are packed at compile time into
a 768-byte 1bpp atlas with 128 bytes of character advances and rendered with
`blit_sub`, following the
[WASM-4 custom text guide](https://wasm4.org/docs/guides/text).

```rust
unsafe { *wasm4::DRAW_COLORS = 0x40; }
font::text("Hello, adventurer!\nA new journey begins.", 2, 2);
```

Blank side columns are trimmed so visible glyphs have exactly one pixel of
horizontal spacing. Character advances depend on the glyph width; spaces advance
5 pixels and other blank glyphs retain a 6-pixel advance. Each newline advances 8 pixels.
Add `\n` where you want line breaks; text does not wrap automatically. CRLF line
endings are supported, and non-ASCII characters are visibly replaced with `?`.

Unlike the built-in `wasm4::text`, this uses the usual 1bpp sprite color mapping:
the low nibble of `DRAW_COLORS` is the background, and the next nibble is the
foreground. For example, `0x40` draws palette color 4 on a transparent background;
`0x41` draws palette color 4 on palette color 1. Drawing text leaves `DRAW_COLORS`
unchanged.

## Four-color PNG converter

The Python batch converter automatically extracts a palette for each image and
exports a **2-bit indexed PNG with four palette entries and at most four used
colors**. Transparency is replaced with an opaque background color (black by
default). Partially transparent pixels are blended onto that color before
quantization. The background is kept exactly when the source has transparency.
Dimensions are preserved; there is no cropping, resizing, or dithering.

For sprites that already use four visible colors, `--background lightest` reuses
the lightest visible source color as the background, preserving all four colors
when the foreground is opaque. This avoids merging a highlight to make room for
a fifth color. Brightness is measured with weighted RGB luma; fully transparent
pixels are ignored. A fully transparent image requires an explicit hex
background because it has no visible color to select.

Install with Python 3.10 or later:

```shell
python3 -m venv .venv
.venv/bin/python -m pip install -r tools/requirements.txt
```

If creating the virtual environment reports that `ensurepip` is missing, install
your operating system's matching `python3-venv` package first.

Convert a directory of PNGs (not including subdirectories):

```shell
.venv/bin/python tools/palette_png.py assets/art -o assets/palettized --background lightest
```

Or pass individual files or a shell wildcard:

```shell
.venv/bin/python tools/palette_png.py assets/art/*.png -o assets/palettized
```

For a specific background instead, use `--background "#20283d"`. If that color
is not already in the sprite, it occupies one of the four palette slots, leaving
only three for the foreground.

On Windows, use `.venv\Scripts\python.exe` instead of `.venv/bin/python`.
Quote hex colors starting with `#` in the shell. Originals are never modified;
existing outputs are rejected unless `--overwrite` is supplied. Animated PNGs
are rejected rather than silently exporting only one frame. Invalid files are
reported on stderr and cause a nonzero exit status.

## Links

- [Documentation](https://wasm4.org/docs): Learn more about WASM-4.
- [Snake Tutorial](https://wasm4.org/docs/tutorials/snake/goal): Learn how to build a complete game
  with a step-by-step tutorial.
- [GitHub](https://github.com/aduros/wasm4): Submit an issue or PR. Contributions are welcome!
