"""Generate the Windows icon from the rendered application logo (requires Pillow)."""
from pathlib import Path
from PIL import Image

brand = Path(__file__).resolve().parent.parent / "assets" / "brand"
with Image.open(brand / "logo.png") as image:
    image.save(
        brand / "app.ico",
        format="ICO",
        sizes=[(size, size) for size in (16, 24, 32, 48, 64, 128, 256)],
        bitmap_format="bmp",
    )
