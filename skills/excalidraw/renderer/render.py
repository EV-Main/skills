#!/usr/bin/env python3
"""Render an .excalidraw scene to PNG, to look at it.

usage: render.py <scene.excalidraw> <out.png> [--crop x0,y0,x1,y1] [--scale S] [--ids] [--trace]
       render.py <scene.excalidraw> --sections [N]
  --crop      scene coordinates; default is the whole scene
  --scale     output pixels per scene unit
  --ids       label every element but strokes with its type and position
  --trace     colour each freehand stroke on its own, its start dotted green and its end red
  --sections  print the N largest rectangles/frames and the text in each, as crop boxes; renders nothing
"""
import argparse, html, json, math, subprocess, sys, tempfile

FONTS = {1: "Virgil, 'Comic Sans MS', cursive", 2: "Helvetica, Arial, sans-serif", 3: "'DejaVu Sans Mono', monospace"}
TRACE = ["#e6194b", "#3cb44b", "#4363d8", "#f58231", "#911eb4", "#42d4f4", "#f032e6", "#9a6324", "#800000", "#469990"]


def bbox(e):
    return e["x"], e["y"], e["x"] + e["width"], e["y"] + e["height"]


def inside(inner, outer):
    return outer[0] <= inner[0] and outer[1] <= inner[1] and inner[2] <= outer[2] and inner[3] <= outer[3]


def color(c):
    return "none" if c in (None, "transparent") else c


def element_svg(e, files):
    x, y, w, h = e["x"], e["y"], e["width"], e["height"]
    op = e.get("opacity", 100) / 100
    rot = f' transform="rotate({math.degrees(e["angle"])} {x + w / 2} {y + h / 2})"' if e.get("angle") else ""
    sc, bg, sw = color(e["strokeColor"]), color(e["backgroundColor"]), e["strokeWidth"]
    dash = {"dashed": ' stroke-dasharray="8 6"', "dotted": ' stroke-dasharray="2 4"'}.get(e.get("strokeStyle"), "")
    t = e["type"]
    if t in ("rectangle", "diamond", "ellipse"):
        common = f'stroke="{sc}" fill="{bg}" stroke-width="{sw}"{dash} opacity="{op}"{rot}'
        if t == "rectangle":
            r = min(w, h) * 0.1 if e.get("roundness") else 0
            return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" {common}/>'
        if t == "ellipse":
            return f'<ellipse cx="{x + w / 2}" cy="{y + h / 2}" rx="{w / 2}" ry="{h / 2}" {common}/>'
        return f'<polygon points="{x + w / 2},{y} {x + w},{y + h / 2} {x + w / 2},{y + h} {x},{y + h / 2}" {common}/>'
    if t in ("freedraw", "line", "arrow"):
        pts = " ".join(f"{x + px},{y + py}" for px, py in e["points"])
        width = sw * 2 if t == "freedraw" else sw
        out = f'<polyline points="{pts}" fill="none" stroke="{sc}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"{dash} opacity="{op}"{rot}/>'
        if t == "arrow" and e.get("endArrowhead") and len(e["points"]) > 1:
            (ax, ay), (bx, by) = e["points"][-2], e["points"][-1]
            a = math.atan2(by - ay, bx - ax)
            tx, ty = x + bx, y + by
            wings = [(tx - 12 * math.cos(a + s), ty - 12 * math.sin(a + s)) for s in (0.5, -0.5)]
            out += f'<polyline points="{wings[0][0]},{wings[0][1]} {tx},{ty} {wings[1][0]},{wings[1][1]}" fill="none" stroke="{sc}" stroke-width="{sw}" opacity="{op}"/>'
        return out
    if t == "text":
        fs, lh = e["fontSize"], e.get("lineHeight", 1.25)
        anchor = {"left": "start", "center": "middle", "right": "end"}[e.get("textAlign", "left")]
        tx = {"start": x, "middle": x + w / 2, "end": x + w}[anchor]
        lines = "".join(
            f'<tspan x="{tx}" y="{y + fs * lh * (i + 0.8)}">{html.escape(l) or " "}</tspan>'
            for i, l in enumerate(e["text"].split("\n"))
        )
        return f'<text font-family="{FONTS.get(e["fontFamily"], FONTS[2])}" font-size="{fs}" fill="{sc}" text-anchor="{anchor}" xml:space="preserve" opacity="{max(op, 0.85)}"{rot}>{lines}</text>'  # faded text is unreadable once scaled down
    if t == "image":
        return f'<image href="{files[e["fileId"]]["dataURL"]}" x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="none"{rot}/>'
    if t in ("frame", "magicframe", "embeddable", "iframe"):
        label = e.get("name") or e.get("link") or t
        return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="#999" stroke-dasharray="4 4"/><text x="{x}" y="{y - 4}" font-size="14" fill="#666">{html.escape(label)}</text>'
    sys.exit(f"unhandled element type {t}")


def sections(els, n):
    boxes = sorted((e for e in els if e["type"] in ("rectangle", "frame", "magicframe")), key=lambda e: -e["width"] * e["height"])[:n]
    texts = [e for e in els if e["type"] == "text"]
    for b in boxes:
        bb = bbox(b)
        above = (bb[0], bb[1] - 40, bb[2], bb[3])  # a title is often written on or just over the border
        title = b.get("name") or " | ".join(t["text"].split("\n")[0][:40] for t in texts if inside(bbox(t), above))[:160]
        print(",".join(str(round(v)) for v in bb), f"{round(b['width'])}x{round(b['height'])}", title)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("scene")
    p.add_argument("out", nargs="?")
    p.add_argument("--crop")
    p.add_argument("--scale", type=float, default=1.0)
    p.add_argument("--ids", action="store_true")
    p.add_argument("--trace", action="store_true")
    p.add_argument("--sections", type=int, nargs="?", const=20)
    a = p.parse_args()
    scene = json.load(open(a.scene))
    els = [e for e in scene["elements"] if not e.get("isDeleted")]
    if a.sections:
        return sections(els, a.sections)
    if not a.out:
        p.error("out is required unless --sections")
    boxes = [bbox(e) for e in els]
    full = (min(b[0] for b in boxes), min(b[1] for b in boxes), max(b[2] for b in boxes), max(b[3] for b in boxes))
    print("scene bbox:", ",".join(str(round(v)) for v in full))
    x0, y0, x1, y1 = map(float, a.crop.split(",")) if a.crop else (full[0] - 20, full[1] - 20, full[2] + 20, full[3] + 20)
    w, h = x1 - x0, y1 - y0
    strokes = [e for e in els if e["type"] == "freedraw"]
    if a.trace:
        for i, e in enumerate(strokes):
            e["strokeColor"], e["opacity"] = TRACE[i % len(TRACE)], 100
    body = "\n".join(element_svg(e, scene.get("files") or {}) for e in els)
    if a.trace:
        for e in strokes:
            (sx, sy), (ex, ey) = e["points"][0], e["points"][-1]
            body += f'<circle cx="{e["x"] + sx}" cy="{e["y"] + sy}" r="3" fill="lime"/><circle cx="{e["x"] + ex}" cy="{e["y"] + ey}" r="3" fill="red"/>'
    if a.ids:
        body += "".join(
            f'<text x="{e["x"]}" y="{e["y"] - 2}" font-size="9" fill="red">{e["type"][0]}@{round(e["x"])},{round(e["y"])}</text>'
            for e in els if e["type"] != "freedraw"
        )
    bg = scene.get("appState", {}).get("viewBackgroundColor", "#fff")
    svg = (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w * a.scale}" height="{h * a.scale}" viewBox="{x0} {y0} {w} {h}">'
        f'<rect x="{x0}" y="{y0}" width="{w}" height="{h}" fill="{bg}"/>{body}</svg>'
    )
    with tempfile.NamedTemporaryFile("w", suffix=".svg") as f:
        f.write(svg)
        f.flush()
        subprocess.run(["rsvg-convert", f.name, "-o", a.out], check=True)
    print("wrote", a.out, f"{round(w * a.scale)}x{round(h * a.scale)}")


main()
