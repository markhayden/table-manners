"""Writes the pixel-digest test vectors (CONTRACT.md 2.2) and digests.json beside this script.

Each PNG is written chunk by chunk from samples chosen here, and its digest is computed from the
canonical samples the contract says a decoder must produce, not by decoding the file, so the
expected values are independent of any image library. Run with `python3 -I make.py`; the output is
deterministic and committed.
"""

import hashlib
import json
import struct
import zlib
from pathlib import Path

HERE = Path(__file__).resolve().parent


def chunk(kind, data):
    body = kind + data
    return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))


def png(width, height, bit_depth, colour_type, rows, extra=()):
    """rows: one bytes object per row, already packed at the bit depth, without filter bytes."""
    header = struct.pack(">IIBBBBB", width, height, bit_depth, colour_type, 0, 0, 0)
    raw = b"".join(b"\x00" + row for row in rows)
    out = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
    for kind, data in extra:
        out += chunk(kind, data)
    return out + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")


def pixel_sha256(width, height, channels, samples):
    h = hashlib.sha256()
    h.update(b"pixels")
    h.update(b"\x00")
    h.update(struct.pack("<III", width, height, channels))
    h.update(bytes(samples))
    return h.hexdigest()


def to8(v16):
    return (v16 + 128) // 257


RGB8 = [[(255, 0, 0), (0, 255, 0), (0, 0, 255)], [(17, 34, 51), (128, 128, 128), (250, 251, 252)]]
RGB8_ROWS = [bytes(c for px in row for c in px) for row in RGB8]
RGB8_SAMPLES = [c for row in RGB8 for px in row for c in px]

# An eXIf chunk holding one tag, Orientation = 6 (rotate 90 degrees clockwise), big-endian TIFF.
EXIF_ROTATE = b"MM\x00\x2a\x00\x00\x00\x08" + b"\x00\x01" + b"\x01\x12\x00\x03\x00\x00\x00\x01\x00\x06\x00\x00" + b"\x00\x00\x00\x00"

VECTORS = []


def vector(name, note, width, height, channels, samples, data):
    path = HERE / name
    path.write_bytes(data)
    VECTORS.append({
        "file": name,
        "note": note,
        "file_sha256": hashlib.sha256(data).hexdigest(),
        "width": width,
        "height": height,
        "channels": channels,
        "pixel_sha256": pixel_sha256(width, height, channels, samples),
    })


grey = [[0, 64, 255], [1, 128, 254]]
vector("grey8.png", "8-bit grey is kept: 1 channel", 3, 2, 1,
       [v for row in grey for v in row], png(3, 2, 8, 0, [bytes(r) for r in grey]))

vector("rgb8.png", "8-bit RGB is kept: 3 channels", 3, 2, 3, RGB8_SAMPLES, png(3, 2, 8, 2, RGB8_ROWS))

vector("rgb8-gamma.png", "a gAMA chunk changes the file, not the pixels: no colour conversion", 3, 2, 3,
       RGB8_SAMPLES, png(3, 2, 8, 2, RGB8_ROWS, [(b"gAMA", struct.pack(">I", 45455))]))

vector("rgb8-exif-rotated.png", "EXIF orientation is not applied: same samples, same width and height", 3, 2, 3,
       RGB8_SAMPLES, png(3, 2, 8, 2, RGB8_ROWS, [(b"eXIf", EXIF_ROTATE)]))

rgba = [[(255, 0, 0, 255), (0, 255, 0, 0)], [(10, 20, 30, 128), (200, 100, 50, 1)]]
vector("rgba8.png", "8-bit RGBA is kept: 4 channels, alpha untouched", 2, 2, 4,
       [c for row in rgba for px in row for c in px],
       png(2, 2, 8, 6, [bytes(c for px in row for c in px) for row in rgba]))

bits = [1, 0, 1, 1, 0, 0, 1, 0]
vector("grey1.png", "1-bit grey expands to 8-bit grey at full range: 0 and 255", 8, 1, 1,
       [255 * b for b in bits], png(8, 1, 1, 0, [bytes([int("".join(map(str, bits)), 2)])]))

grey_alpha = [[(10, 255), (200, 0)], [(77, 128), (255, 3)]]
vector("grey-alpha8.png", "grey with alpha converts to RGB: grey copied to three channels, alpha dropped", 2, 2, 3,
       [g for row in grey_alpha for g, _ in row for _ in range(3)],
       png(2, 2, 8, 4, [bytes(c for px in row for c in px) for row in grey_alpha]))

grey16 = [[0, 257 * 100, 65535]]
vector("grey16.png", "16-bit grey converts to 8-bit RGB, each sample (v + 128) / 257", 3, 1, 3,
       [to8(v) for v in grey16[0] for _ in range(3)],
       png(3, 1, 16, 0, [b"".join(struct.pack(">H", v) for v in grey16[0])]))

rgb16 = [[(65535, 0, 32896), (257 * 17, 300, 65280)]]
vector("rgb16.png", "16-bit RGB converts to 8-bit RGB, each sample (v + 128) / 257, rounding half up", 2, 1, 3,
       [to8(c) for px in rgb16[0] for c in px],
       png(2, 1, 16, 2, [b"".join(struct.pack(">HHH", *px) for px in rgb16[0])]))

rgba16 = [[(65535, 0, 257 * 9, 0), (257, 514, 771, 65535)]]
vector("rgba16.png", "16-bit RGBA converts to 8-bit RGB: alpha dropped", 2, 1, 3,
       [to8(c) for px in rgba16[0] for c in px[:3]],
       png(2, 1, 16, 6, [b"".join(struct.pack(">HHHH", *px) for px in rgba16[0])]))

palette = [(255, 0, 0), (0, 128, 255), (9, 9, 9)]
indices = [2, 0, 1]
plte = bytes(c for px in palette for c in px)
vector("palette.png", "a palette without transparency expands to 8-bit RGB", 3, 1, 3,
       [c for i in indices for c in palette[i]], png(3, 1, 8, 3, [bytes(indices)], [(b"PLTE", plte)]))

trns = [255, 0]  # entry 0 opaque, entry 1 transparent, entry 2 opaque by omission
vector("palette-trns.png", "a palette with tRNS expands to 8-bit RGBA", 3, 1, 4,
       [c for i in indices for c in palette[i] + ((trns[i] if i < len(trns) else 255),)],
       png(3, 1, 8, 3, [bytes(indices)], [(b"PLTE", plte), (b"tRNS", bytes(trns))]))

(HERE / "digests.json").write_text(json.dumps({"recipe": "CONTRACT.md 2.2", "vectors": VECTORS}, indent=2) + "\n")
