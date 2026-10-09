"""Writes a plain 1024x1024 PNG, the source the Tauri icon command needs. Usage: python -I make_icon.py <out.png>"""
import struct
import sys
import zlib

w = h = 1024
raw = (b"\x00" + bytes([37, 99, 235, 255]) * w) * h


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
with open(sys.argv[1], "wb") as f:
    f.write(png)
