from __future__ import annotations

import struct
import zlib
from pathlib import Path


def png_bytes(size: int, rgb: tuple[int, int, int] = (44, 107, 90)) -> bytes:
    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    raw = b"".join(b"\x00" + (bytes(rgb) * size) for _ in range(size))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def write_png(path: Path, size: int) -> None:
    path.write_bytes(png_bytes(size))


def write_ico(path: Path, size: int = 256) -> None:
    png = png_bytes(size)
    header = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack(
        "<BBBBHHII",
        size if size < 256 else 0,
        size if size < 256 else 0,
        0,
        0,
        1,
        32,
        len(png),
        22,
    )
    path.write_bytes(header + entry + png)


def main() -> None:
    icons = Path("desktop/src-tauri/icons")
    icons.mkdir(parents=True, exist_ok=True)
    write_png(icons / "32x32.png", 32)
    write_png(icons / "128x128.png", 128)
    write_png(icons / "128x128@2x.png", 256)
    write_png(icons / "icon.png", 512)
    write_ico(icons / "icon.ico", 256)
    # Minimal ICNS with a 512px PNG (ic07)
    png = png_bytes(512)
    payload = b"ic07" + struct.pack(">I", len(png) + 8) + png
    icns = b"icns" + struct.pack(">I", len(payload) + 8) + payload
    (icons / "icon.icns").write_bytes(icns)


if __name__ == "__main__":
    main()
