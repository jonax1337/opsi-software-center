"""Refresh the outlined brand lettering from the locally installed Segoe font."""

import json
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont


FONT_PATH = Path("C:/Windows/Fonts/SegUIVar.ttf")
TEXT = {
    "name": ("Software Portal", 700),
    "tagline": ("Alles bereit für deine Arbeit.", 600),
    "opsi": ("für opsi", 600),
    "description": ("Freigegebene Apps. Einfach installiert.", 400),
}


def outline(font, text, weight):
    glyphs = font.getGlyphSet(location={"wght": weight, "opsz": 36})
    cmap = font.getBestCmap()
    path = SVGPathPen(glyphs)
    bounds = BoundsPen(glyphs)
    x = 0
    for character in text:
        glyph = glyphs[cmap[ord(character)]]
        transform = (1, 0, 0, 1, x, 0)
        glyph.draw(TransformPen(path, transform))
        glyph.draw(TransformPen(bounds, transform))
        x += glyph.width
    return {"text": text, "weight": weight, "advance": x,
            "unitsPerEm": font["head"].unitsPerEm,
            "bounds": bounds.bounds, "d": path.getCommands()}


def main():
    with TTFont(FONT_PATH) as font:
        result = {name: outline(font, text, weight)
                  for name, (text, weight) in TEXT.items()}
    output = Path(__file__).with_name("lettering.json")
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n",
                      encoding="utf-8")
    print("Outlined lettering saved. No font files are copied.")


if __name__ == "__main__":
    main()
