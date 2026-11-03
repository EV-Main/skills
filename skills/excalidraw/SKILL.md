---
name: excalidraw
description: "Read, study or review an Excalidraw drawing (`.excalidraw`, `.excalidraw.json`, `.excalidraw.md`, excalidraw.com scenes): render it to PNG with a bundled renderer and look at it, cropped and zoomed per section. Use whenever excalidraw is mentioned, or a design board, whiteboard or sketch saved from it has to be understood, summarised, compared to code or redrawn."
license: "BlueOak-1.0.0"
compatibility: "python3, rsvg-convert (librsvg)"
metadata:
  version: "0.1.0"
---

# excalidraw

Study a scene by looking at it. Grepping its JSON alone is almost never enough. The JSON gives you the
text, but a board's meaning is in where things sit: which note is inside which box, where each freehand
arrow starts and ends, what a sketched UI looks like, what the pasted screenshots show. Freehand strokes
and images have no text to grep at all.

`renderer/render.py` turns a scene, or one crop of it, into a PNG for you to read (`--help`).

- **Orientation.** Large rectangles usually mark a board's sections. `--sections` lists them as crop
  boxes, with the text in or just over each one. Render the whole scene small to see how the sections are
  laid out. Then crop each section at a scale where its text is legible.
- **Freehand arrows.** `--trace` gives each stroke its own colour and dots its start green and its end
  red. Some strokes predate a later move of the label they point at, and land on its neighbour. Each
  element's `updated` field (epoch ms) shows when it last changed. A label that changed after the arrow
  means the label may have moved, so read what the arrow means from the content at both ends.
- **Fidelity.** Text is drawn in stand-in fonts, so its rendered width is approximate. The stored `x` and
  `width` are what decide which label spans where. Arrow bindings, roughness and pen pressure are not
  drawn.
- **Images** are base64 in `files[<fileId>].dataURL`. Decode one to see it at full resolution when a
  scaled crop blurs it.
- **Mermaid**, in the rare case it fits: a scene drawn with bound arrows (`startBinding`/`endBinding`
  naming element ids) is a graph that can be extracted exactly. A freehand board has no such edges.
