"""Writes Steplet's 1024 x 1024 icon source as a PNG: three white steps on a blue rounded square.
The Tauri icon command makes every size from it. Usage: python -I make_icon.py <out.png>"""
import struct
import sys
import zlib

SIZE = 1024
RADIUS = 225  # the rounded square's corner radius
BLUE = (37, 99, 235)  # the accent colour of web/shared/tokens.css (light theme)
WHITE = (255, 255, 255)
# The steps: (left, top, right, bottom), each rising to the right; they touch, so they read as one staircase.
STEPS = [(232, 589, 419, 776), (419, 403, 605, 776), (605, 216, 792, 776)]
SUB = 4  # samples per pixel side at the rounded corners, so their edge is smooth


def inside_square(x, y):
    """Whether the point (in pixels, fractional) is inside the rounded square."""
    cx = min(max(x, RADIUS), SIZE - RADIUS)
    cy = min(max(y, RADIUS), SIZE - RADIUS)
    return (x - cx) ** 2 + (y - cy) ** 2 <= RADIUS**2


def square_cover(px, py):
    """How much of pixel (px, py) the rounded square covers, from 0 to 1."""
    near_corner = (px < RADIUS or px >= SIZE - RADIUS) and (py < RADIUS or py >= SIZE - RADIUS)
    if not near_corner:
        return 1.0
    hits = 0
    for i in range(SUB):
        for j in range(SUB):
            hits += inside_square(px + (i + 0.5) / SUB, py + (j + 0.5) / SUB)
    return hits / (SUB * SUB)


def pixel(px, py):
    cover = square_cover(px, py)
    if cover == 0:
        return (0, 0, 0, 0)
    on_step = any(left <= px < right and top <= py < bottom for left, top, right, bottom in STEPS)
    r, g, b = WHITE if on_step else BLUE
    return (r, g, b, round(255 * cover))


rows = []
for py in range(SIZE):
    row = bytearray(b"\x00")
    for px in range(SIZE):
        row.extend(pixel(px, py))
    rows.append(bytes(row))
raw = b"".join(rows)


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(raw, 9))
    + chunk(b"IEND", b"")
)
with open(sys.argv[1], "wb") as f:
    f.write(png)
