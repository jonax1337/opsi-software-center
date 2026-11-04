"""Build the Soft Stack SVG family from one traced master and outlined lettering."""

import json
from html import escape
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TYPE = json.loads(Path(__file__).with_name("lettering.json").read_text(encoding="utf-8"))
EXPORTS = []
COLORS = {"blue": "#185ABD", "sky": "#69C6F3", "paper": "#F5F7FB",
          "ink": "#242424", "white": "#FFFFFF", "night": "#14233B"}
BACK = "M156 149C156 134 164 123 178 117L284 76C302 68 323 79 323 101V237C323 254 311 265 294 266L182 268C166 268 156 258 156 243Z"
MIDDLE = "M200 177C200 162 208 151 222 146L326 108C345 101 366 113 366 135V250C366 267 354 280 337 281L227 284C211 285 200 272 200 257Z"
FRONT = "M244 212C244 198 252 186 266 181L367 144C387 136 406 149 406 170V260C406 279 393 292 375 293L272 298C256 299 244 287 244 272Z"
HOLE = '<rect x="335" y="224.5" width="41" height="40" rx="6"/>'


def definitions():
    return f'''<defs>
  <linearGradient id="back" x1="205" y1="95" x2="265" y2="275" gradientUnits="userSpaceOnUse"><stop stop-color="#B6E0FF"/><stop offset="1" stop-color="#9BD4FA"/></linearGradient>
  <linearGradient id="middle" x1="251" y1="127" x2="303" y2="290" gradientUnits="userSpaceOnUse"><stop stop-color="#65C9FB"/><stop offset="1" stop-color="#43B5ED"/></linearGradient>
  <linearGradient id="front" x1="287" y1="171" x2="364" y2="309" gradientUnits="userSpaceOnUse"><stop stop-color="#1B67D0"/><stop offset=".55" stop-color="#155DC5"/><stop offset="1" stop-color="#0B4DB1"/></linearGradient>
  <linearGradient id="edge" x1="200" y1="110" x2="350" y2="280" gradientUnits="userSpaceOnUse"><stop stop-color="#FFFFFF" stop-opacity=".54"/><stop offset=".6" stop-color="#FFFFFF" stop-opacity=".1"/><stop offset="1" stop-color="#FFFFFF" stop-opacity="0"/></linearGradient>
  <linearGradient id="plate" x1="80" y1="50" x2="430" y2="480" gradientUnits="userSpaceOnUse"><stop stop-color="#FFFFFF"/><stop offset=".55" stop-color="#FAFCFF"/><stop offset="1" stop-color="#E9F0FA"/></linearGradient>
  <linearGradient id="dark-plate" x1="80" y1="50" x2="430" y2="480" gradientUnits="userSpaceOnUse"><stop stop-color="#2D405A"/><stop offset="1" stop-color="#15253E"/></linearGradient>
  <mask id="cutout" maskUnits="userSpaceOnUse" x="130" y="50" width="300" height="270"><rect x="130" y="50" width="300" height="270" fill="white"/><g fill="black">{HOLE}</g></mask>
  <mask id="back-gap" maskUnits="userSpaceOnUse" x="130" y="50" width="300" height="270"><rect x="130" y="50" width="300" height="270" fill="white"/><g fill="black" stroke="black" stroke-width="7"><path d="{MIDDLE}"/><path d="{FRONT}"/></g></mask>
  <mask id="middle-gap" maskUnits="userSpaceOnUse" x="130" y="50" width="300" height="270"><rect x="130" y="50" width="300" height="270" fill="white"/><path d="{FRONT}" fill="black" stroke="black" stroke-width="7"/></mask>
  <filter id="plate-shadow" x="-25%" y="-25%" width="150%" height="165%" color-interpolation-filters="sRGB"><feDropShadow dx="0" dy="14" stdDeviation="13" flood-color="#234B80" flood-opacity=".21"/></filter>
  <filter id="card-shadow" x="-35%" y="-25%" width="170%" height="165%" color-interpolation-filters="sRGB"><feDropShadow dx="0" dy="5" stdDeviation="4" flood-color="#164B94" flood-opacity=".16"/></filter>
  <linearGradient id="banner" x2="1" y2="1"><stop stop-color="#FBFDFF"/><stop offset=".6" stop-color="#EDF5FF"/><stop offset="1" stop-color="#D3E9FF"/></linearGradient>
  <linearGradient id="wave" x2=".8" y2="1"><stop stop-color="#8DD7FF" stop-opacity=".26"/><stop offset="1" stop-color="#1768D1" stop-opacity=".55"/></linearGradient>
</defs>'''


def symbol(style="color", dimensional=False):
    if style in ("#242424", "#FFFFFF", "#185ABD"):
        layers = (f'<path d="{BACK}" fill="{style}" mask="url(#back-gap)"/>'
                  f'<path d="{MIDDLE}" fill="{style}" mask="url(#middle-gap)"/>'
                  f'<path d="{FRONT}" fill="{style}"/>')
    else:
        fills = [COLORS["sky"], "#43B5ED", COLORS["blue"]] if style == "flat" else [
            "url(#back)", "url(#middle)", "url(#front)"]
        layers = "".join(f'<path d="{path}" fill="{fill}"/>'
                         for path, fill in zip([BACK, MIDDLE, FRONT], fills))
        if dimensional:
            layers = (f'<g filter="url(#card-shadow)">{layers}</g>'
                      f'<path d="{FRONT}" fill="none" stroke="url(#edge)" stroke-width="1.3"/>')
    return f'<g mask="url(#cutout)">{layers}</g>'


def placed_symbol(x, y, width, style="color", dimensional=False):
    scale = width / 250
    return (f'<g transform="translate({x:g} {y:g}) scale({scale:g}) translate(-156 -73)">'
            f'{symbol(style, dimensional)}</g>')


def type_width(key, size):
    item = TYPE[key]
    return item["advance"] / item["unitsPerEm"] * size


def lettering(key, x, baseline, size, fill, width=None):
    item = TYPE[key]
    scale = size / item["unitsPerEm"]
    scale_x = scale if width is None else width / item["advance"]
    return (f'<path fill="{fill}" transform="translate({x:g} {baseline:g}) '
            f'scale({scale_x:g} {-scale:g})" d="{item["d"]}"/>')


def document(width, height, body, title, view_box=None):
    box = view_box or f"0 0 {width} {height}"
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
            f'viewBox="{box}" fill="none" role="img" aria-labelledby="title">\n'
            f'<title id="title">{escape(title)}</title>\n{definitions()}\n{body}\n</svg>\n')


def save_svg(relative, content, raster_sizes=()):
    path = ROOT / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    for output, width in raster_sizes:
        EXPORTS.append({"input": relative, "output": output, "width": width})


def build_marks():
    variants = {"mark": "color", "mark-flat": "flat", "mark-mono-dark": "#242424",
                "mark-mono-light": "#FFFFFF", "mark-mono-blue": "#185ABD"}
    for name, style in variants.items():
        sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024, 2048] if name == "mark" else [512, 1024]
        exports = [(f"png/{name}/{size}.png", size) for size in sizes]
        save_svg(f"svg/{name}.svg", document(512, 512, placed_symbol(47, 68, 418, style),
                 "Software Center · Soft Stack"), exports)


def build_lockups():
    for suffix, fill in [("", COLORS["ink"]), ("-inverse", COLORS["white"])]:
        name = "logo-horizontal" + suffix
        body = placed_symbol(20, 21, 190) + lettering("name", 252, 150, 103, fill)
        save_svg(f"svg/{name}.svg", document(1100, 224, body, "Software Center"),
                 [(f"png/{name}.png", 1100), (f"png/{name}@2x.png", 2200)])
        name = "logo-stacked" + suffix
        word_width = type_width("name", 94)
        body = placed_symbol(271, 56, 418) + lettering("name", (960-word_width)/2, 521, 94, fill)
        save_svg(f"svg/{name}.svg", document(960, 580, body, "Software Center"),
                 [(f"png/{name}.png", 960), (f"png/{name}@2x.png", 1920)])
        name = "wordmark" + suffix
        body = lettering("name", 16, 98, 112, fill)
        save_svg(f"svg/{name}.svg", document(910, 132, body, "Software Center"),
                 [(f"png/{name}.png", 1820)])


def build_app_icons():
    for suffix in ["", "-dark"]:
        plate = "dark-plate" if suffix else "plate"
        body = (f'<rect x="48" y="38" width="416" height="416" rx="104" '
                f'fill="url(#{plate})" filter="url(#plate-shadow)"/>'
                '<rect x="48.5" y="38.5" width="415" height="415" rx="103.5" '
                'stroke="white" stroke-opacity=".64"/>' + placed_symbol(82, 88, 348, dimensional=True))
        sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024, 2048]
        name = "app-icon" + suffix
        save_svg(f"svg/{name}.svg", document(512, 512, body, "Software Center App-Icon"),
                 [(f"png/{name}/{size}.png", size) for size in sizes])


def waves(dark=False):
    opacity = ".28" if dark else ".7"
    return (f'<g opacity="{opacity}"><path d="M620 520C860 320 1120 420 1600 105V520Z" fill="url(#wave)"/>'
            '<path d="M820 520C1015 380 1260 450 1600 260V520Z" fill="url(#wave)"/>'
            '<path d="M1030 520C1180 415 1380 470 1600 363V520Z" fill="url(#wave)"/></g>')


def build_banners():
    for suffix, dark in [("", False), ("-dark", True)]:
        ink = COLORS["white"] if dark else COLORS["ink"]
        secondary = "#C5D8EE" if dark else "#52657D"
        background = COLORS["night"] if dark else "url(#banner)"
        body = (f'<rect width="1600" height="520" rx="24" fill="{background}"/>'
                + waves(dark) + lettering("opsi", 91, 124, 25, secondary)
                + lettering("name", 87, 239, 101, ink)
                + lettering("tagline", 91, 319, 38, secondary)
                + placed_symbol(1072, 93, 346, dimensional=True))
        name = "readme-banner" + suffix
        save_svg(f"banners/{name}.svg", document(1600, 520, body, "Software Center · Alles bereit für deine Arbeit."),
                 [(f"banners/{name}.png", 1600), (f"banners/{name}@2x.png", 3200)])
    body = ('<rect width="1200" height="630" fill="url(#banner)"/>'
            '<g transform="translate(-400 90)">' + waves() + '</g>'
            + placed_symbol(475, 61, 250, dimensional=True)
            + lettering("name", (1200-type_width("name", 78))/2, 425, 78, COLORS["ink"])
            + lettering("tagline", (1200-type_width("tagline", 30))/2, 493, 30, "#52657D"))
    save_svg("social/social-preview.svg", document(1200, 630, body, "Software Center für opsi"),
             [("social/social-preview.png", 1200)])


def build_manifest():
    (ROOT / "source/exports.json").write_text(json.dumps(EXPORTS, indent=2) + "\n", encoding="utf-8")
    tokens = {"name": "Software Center", "direction": "02 Soft Stack", "colors": COLORS,
              "fontFamily": ["Segoe UI Variable", "Segoe UI", "system-ui", "sans-serif"],
              "fontWeights": {"body": 400, "heading": 600, "wordmark": 700},
              "tagline": TYPE["tagline"]["text"], "clearSpace": "mindestens Breite des Ausschnitts",
              "minimumMarkSize": 16, "controlRadius": 4, "cardRadius": 8}
    (ROOT / "tokens.json").write_text(json.dumps(tokens, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    css = ":root {\n" + "".join(f"  --brand-{key}: {value};\n" for key, value in COLORS.items())
    css += '  --brand-font: "Segoe UI Variable", "Segoe UI", system-ui, sans-serif;\n}\n'
    (ROOT / "tokens.css").write_text(css, encoding="utf-8")


def main():
    build_marks()
    build_lockups()
    build_app_icons()
    build_banners()
    build_manifest()
    print(f"Built SVG family and {len(EXPORTS)} PNG export specifications.")


if __name__ == "__main__":
    main()
