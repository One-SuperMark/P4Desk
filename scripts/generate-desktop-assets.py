#!/usr/bin/env python3
"""Bake the project's original SVG icons using only Python's standard library.

This intentionally small renderer supports the SVG primitives used by our source
artwork: rounded rectangles, circles, round lines, polygons and vertical gradients.
It is not a general SVG renderer. All edges use deterministic 4 x 4 supersampling.
"""

import argparse
import hashlib
import json
import math
from pathlib import Path
import struct
import sys
import xml.etree.ElementTree as ET
import zlib


ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'assets' / 'app_icons'
IDS = ('clock', 'timer', 'notes', 'calculator', 'mac', 'settings', 'display', 'screen')
SIZE = 128
SUPERSAMPLE = 4
PREVIEW_COLUMNS = 4
BAYER_4X4 = ((0, 8, 2, 10), (12, 4, 14, 6), (3, 11, 1, 9), (15, 7, 13, 5))


def number(element, key, default=0.0):
    return float(element.get(key, default))


def color(value):
    if value == 'none':
        return None
    if value.startswith('#') and len(value) == 7:
        return tuple(int(value[i:i + 2], 16) for i in (1, 3, 5))
    raise ValueError(f'Unsupported SVG color: {value}')


def rounded_rect_contains(x, y, bx, by, width, height, radius):
    if width <= 0 or height <= 0 or not (bx <= x <= bx + width and by <= y <= by + height):
        return False
    radius = min(max(radius, 0.0), width / 2, height / 2)
    cx = min(max(x, bx + radius), bx + width - radius)
    cy = min(max(y, by + radius), by + height - radius)
    return (x - cx) ** 2 + (y - cy) ** 2 <= radius ** 2


def polygon_contains(x, y, points):
    inside = False
    previous = points[-1]
    for current in points:
        x1, y1 = previous
        x2, y2 = current
        if (y1 > y) != (y2 > y) and x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
            inside = not inside
        previous = current
    return inside


def line_contains(x, y, x1, y1, x2, y2, width):
    dx, dy = x2 - x1, y2 - y1
    length_squared = dx * dx + dy * dy
    t = max(0.0, min(1.0, ((x - x1) * dx + (y - y1) * dy) / length_squared)) if length_squared else 0
    return (x - x1 - t * dx) ** 2 + (y - y1 - t * dy) ** 2 <= (width / 2) ** 2


def parse_artwork(path):
    root = ET.fromstring(path.read_bytes())
    if root.get('width') != str(SIZE) or root.get('height') != str(SIZE) or root.get('viewBox') != '0 0 128 128':
        raise ValueError(f'{path.name}: expected a 128 x 128 SVG')
    gradients = {}
    for element in root.iter():
        tag = element.tag.rsplit('}', 1)[-1]
        if tag == 'linearGradient':
            if element.get('gradientUnits') != 'userSpaceOnUse' or number(element, 'x1') != number(element, 'x2'):
                raise ValueError('Only vertical user-space gradients are supported')
            stops = list(element)
            if len(stops) != 2 or stops[0].get('offset') != '0' or stops[1].get('offset') != '1':
                raise ValueError('Gradients must have exactly two stops at 0 and 1')
            gradients[element.get('id')] = (
                number(element, 'y1'), number(element, 'y2'),
                color(stops[0].get('stop-color')), color(stops[1].get('stop-color')),
            )

    layers = []
    for element in root.iter():
        tag = element.tag.rsplit('}', 1)[-1]
        if tag in ('svg', 'defs', 'linearGradient', 'stop', 'title', 'desc', 'g'):
            continue
        stroke_width = number(element, 'stroke-width', 1)
        if tag == 'rect':
            x, y = number(element, 'x'), number(element, 'y')
            width, height, radius = number(element, 'width'), number(element, 'height'), number(element, 'rx')
            bounds = (x - stroke_width / 2, y - stroke_width / 2, x + width + stroke_width / 2, y + height + stroke_width / 2)
            contains = lambda px, py, x=x, y=y, w=width, h=height, r=radius: rounded_rect_contains(px, py, x, y, w, h, r)
            stroke_contains = lambda px, py, x=x, y=y, w=width, h=height, r=radius, s=stroke_width: (
                rounded_rect_contains(px, py, x - s / 2, y - s / 2, w + s, h + s, r + s / 2)
                and not rounded_rect_contains(px, py, x + s / 2, y + s / 2, w - s, h - s, r - s / 2)
            )
        elif tag == 'circle':
            cx, cy, radius = number(element, 'cx'), number(element, 'cy'), number(element, 'r')
            outer = radius + stroke_width / 2
            bounds = (cx - outer, cy - outer, cx + outer, cy + outer)
            contains = lambda px, py, cx=cx, cy=cy, r=radius: (px - cx) ** 2 + (py - cy) ** 2 <= r * r
            stroke_contains = lambda px, py, cx=cx, cy=cy, r=radius, s=stroke_width: max(0, r - s / 2) ** 2 <= (px - cx) ** 2 + (py - cy) ** 2 <= (r + s / 2) ** 2
        elif tag == 'line':
            x1, y1, x2, y2 = (number(element, key) for key in ('x1', 'y1', 'x2', 'y2'))
            bounds = (min(x1, x2) - stroke_width / 2, min(y1, y2) - stroke_width / 2, max(x1, x2) + stroke_width / 2, max(y1, y2) + stroke_width / 2)
            contains = lambda px, py: False
            stroke_contains = lambda px, py, x1=x1, y1=y1, x2=x2, y2=y2, s=stroke_width: line_contains(px, py, x1, y1, x2, y2, s)
        elif tag == 'polygon':
            coordinates = element.get('points', '').replace(',', ' ').split()
            if len(coordinates) < 6 or len(coordinates) % 2:
                raise ValueError('Invalid polygon coordinates')
            points = tuple((float(coordinates[i]), float(coordinates[i + 1])) for i in range(0, len(coordinates), 2))
            bounds = (min(p[0] for p in points), min(p[1] for p in points), max(p[0] for p in points), max(p[1] for p in points))
            contains = lambda px, py, points=points: polygon_contains(px, py, points)
            stroke_contains = lambda px, py: False
            if element.get('stroke', 'none') != 'none':
                raise ValueError('Polygon strokes are not supported')
        else:
            raise ValueError(f'Unsupported SVG primitive: {tag}')
        for key, mask in (('fill', contains), ('stroke', stroke_contains)):
            paint = element.get(key, 'none' if key == 'stroke' else '#000000')
            if paint == 'none':
                continue
            if paint.startswith('url(#') and paint.endswith(')'):
                paint = gradients[paint[5:-1]]
            else:
                paint = color(paint)
            opacity = number(element, 'opacity', 1) * number(element, f'{key}-opacity', 1)
            if not 0 <= opacity <= 1:
                raise ValueError('Invalid SVG opacity')
            layers.append((bounds, mask, paint, opacity))
    return layers


def round_byte(value):
    return max(0, min(255, int(value + 0.5)))


def render(layers):
    side = SIZE * SUPERSAMPLE
    samples = bytearray(side * side * 4)
    for bounds, contains, paint, opacity in layers:
        start_x = max(0, math.floor(bounds[0] * SUPERSAMPLE))
        start_y = max(0, math.floor(bounds[1] * SUPERSAMPLE))
        end_x = min(side, math.ceil(bounds[2] * SUPERSAMPLE))
        end_y = min(side, math.ceil(bounds[3] * SUPERSAMPLE))
        for sy in range(start_y, end_y):
            y = (sy + 0.5) / SUPERSAMPLE
            if len(paint) == 4:
                y1, y2, top, bottom = paint
                t = max(0.0, min(1.0, (y - y1) / (y2 - y1)))
                rgb = tuple(round_byte(a + (b - a) * t) for a, b in zip(top, bottom))
            else:
                rgb = paint
            for sx in range(start_x, end_x):
                x = (sx + 0.5) / SUPERSAMPLE
                if not contains(x, y):
                    continue
                index = (sy * side + sx) * 4
                if opacity == 1:
                    samples[index:index + 4] = bytes((*rgb, 255))
                else:
                    previous_alpha = samples[index + 3] / 255
                    alpha = opacity + previous_alpha * (1 - opacity)
                    if alpha:
                        for channel in range(3):
                            samples[index + channel] = round_byte((rgb[channel] * opacity + samples[index + channel] * previous_alpha * (1 - opacity)) / alpha)
                        samples[index + 3] = round_byte(alpha * 255)

    rgb565, alpha, rgba = bytearray(), bytearray(), bytearray()
    count = SUPERSAMPLE * SUPERSAMPLE
    for y in range(SIZE):
        for x in range(SIZE):
            total_alpha = 0
            totals = [0, 0, 0]
            for dy in range(SUPERSAMPLE):
                for dx in range(SUPERSAMPLE):
                    index = (((y * SUPERSAMPLE + dy) * side) + x * SUPERSAMPLE + dx) * 4
                    sample_alpha = samples[index + 3]
                    total_alpha += sample_alpha
                    for channel in range(3):
                        totals[channel] += samples[index + channel] * sample_alpha
            coverage = round_byte(total_alpha / count)
            # Average premultiplied samples then unpremultiply. Transparent edge
            # pixels retain the actual icon color, avoiding dark blending halos.
            rgb = tuple(round_byte(value / total_alpha) if total_alpha else 0 for value in totals)
            # Deterministic ordered dithering spreads RGB565 quantization error
            # across nearby pixels, preventing large horizontal gradient bands.
            dither = (BAYER_4X4[y % 4][x % 4] + 0.5) / 16 - 0.5
            r, g, b = (max(0, min(levels, math.floor(value * levels / 255 + 0.5 + dither))) for value, levels in zip(rgb, (31, 63, 31)))
            pixel = (r << 11) | (g << 5) | b
            rgb565.extend(struct.pack('<H', pixel))
            alpha.append(coverage)
            # PNG previews show the same quantized colors as the LCD resource.
            rgba.extend(((r * 255 + 15) // 31, (g * 255 + 31) // 63, (b * 255 + 15) // 31, coverage))
    if len(rgb565) != SIZE * SIZE * 2 or len(alpha) != SIZE * SIZE:
        raise AssertionError('Invalid generated asset lengths')
    if min(alpha) != 0 or max(alpha) != 255 or not any(0 < value < 255 for value in alpha):
        raise AssertionError('Expected transparent, opaque and antialiased edge pixels')
    return bytes(rgb565), bytes(alpha), bytes(rgba)


def png(width, height, rgba):
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data) & 0xffffffff)
    scanlines = b''.join(b'\0' + rgba[y * width * 4:(y + 1) * width * 4] for y in range(height))
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(scanlines, 9)) + chunk(b'IEND', b'')


def preview_sheet(images):
    width, height = PREVIEW_COLUMNS * 160, math.ceil(len(images) / PREVIEW_COLUMNS) * 160
    output = bytearray(bytes((20, 28, 44, 255)) * width * height)
    for index, image in enumerate(images):
        left, top = (index % PREVIEW_COLUMNS) * 160 + 16, (index // PREVIEW_COLUMNS) * 160 + 16
        for y in range(SIZE):
            for x in range(SIZE):
                source = (y * SIZE + x) * 4
                destination = ((top + y) * width + left + x) * 4
                coverage = image[source + 3]
                for channel in range(3):
                    output[destination + channel] = (image[source + channel] * coverage + output[destination + channel] * (255 - coverage) + 127) // 255
    return png(width, height, output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='Verify reproducibility without writing files')
    args = parser.parse_args()
    outputs, entries, previews = {}, {}, []
    for icon_id in IDS:
        source = ASSETS / 'source' / f'{icon_id}.svg'
        rgb565, alpha, rgba = render(parse_artwork(source))
        files = {f'{icon_id}.rgb565': rgb565, f'{icon_id}.alpha': alpha, f'{icon_id}.png': png(SIZE, SIZE, rgba)}
        outputs.update(files)
        previews.append(rgba)
        entries[icon_id] = {
            'source': f'source/{icon_id}.svg',
            'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
            'width': SIZE, 'height': SIZE,
            'files': {name: {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()} for name, data in files.items()},
        }
    outputs['preview.png'] = preview_sheet(previews)
    manifest = {
        'schema': 'p4desk.original-app-icons.v1', 'license': 'MIT',
        'pixel_format': 'RGB565', 'byte_order': 'little', 'alpha_format': 'unassociated-u8',
        'supersampling': [SUPERSAMPLE, SUPERSAMPLE],
        'quantization': 'ordered-4x4-bayer',
        'rust_storage_alignment_bytes': 2, 'icons': entries,
        'preview': {'file': 'preview.png', 'columns': PREVIEW_COLUMNS, 'rows': math.ceil(len(IDS) / PREVIEW_COLUMNS), 'order': list(IDS), 'sha256': hashlib.sha256(outputs['preview.png']).hexdigest()},
    }
    outputs['manifest.json'] = (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
    failures = []
    for name, data in outputs.items():
        path = ASSETS / name
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                failures.append(name)
        else:
            path.write_bytes(data)
    if failures:
        print('Asset reproducibility check failed: ' + ', '.join(failures), file=sys.stderr)
        return 1
    print(json.dumps({'valid': True, 'mode': 'check' if args.check else 'generate', 'icons': len(IDS), 'size': [SIZE, SIZE], 'rgb565_bytes_per_icon': SIZE * SIZE * 2, 'alpha_bytes_per_icon': SIZE * SIZE, 'byte_order': 'little', 'supersampling': [SUPERSAMPLE, SUPERSAMPLE]}, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
