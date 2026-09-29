#!/usr/bin/env python3
"""Compile P4Desk's owned SVG subset to static Rust vector geometry, never pixels."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / 'crates/tiny-flutter/src/graphics/svg_icons_generated.rs'
MANIFEST = ROOT / 'assets/vector-icons.json'


def number(value):
    result = float(value)
    if not (-1e6 < result < 1e6):
        raise ValueError('Non-finite or unreasonable coordinate')
    return f'{result:.6f}'.rstrip('0').rstrip('.') + ('.0' if result == int(result) else '')


def color(value, opacity=1.0):
    if value == 'none':
        return 'None'
    if value == 'currentColor':
        return f'Some(VectorPaint::Tint({number(opacity)}))'
    if not re.fullmatch(r'#[0-9a-fA-F]{6}', value):
        raise ValueError(f'Unsupported color {value}')
    channels = [int(value[i:i+2], 16) for i in (1, 3, 5)]
    return f'Some(VectorPaint::Solid(Color::from_rgba({channels[0]}, {channels[1]}, {channels[2]}, {round(opacity * 255)})))'


def path_commands(value):
    tokens = re.findall(r'[MLQCZ]|[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?', value)
    stripped = re.sub(r'[MLQCZ]|[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?|[\s,]', '', value)
    if stripped:
        raise ValueError(f'Unsupported SVG path data: {stripped}')
    arities = {'M': 2, 'L': 2, 'Q': 4, 'C': 6, 'Z': 0}
    verbs = {'M': 'Move', 'L': 'Line', 'Q': 'Quad', 'C': 'Cubic', 'Z': 'Close'}
    out, index, command = [], 0, None
    while index < len(tokens):
        if tokens[index] in arities:
            command = tokens[index]
            index += 1
        if command is None:
            raise ValueError('Path must begin with a command')
        arity = arities[command]
        values = tokens[index:index + arity]
        if len(values) != arity or any(v in arities for v in values):
            raise ValueError('Invalid path command arguments')
        out.append('VectorCommand::' + verbs[command] + ('(' + ','.join(number(v) for v in values) + ')' if arity else ''))
        index += arity
        command = 'L' if command == 'M' else None if command == 'Z' else command
    return 'VectorShape::Path(&[' + ','.join(out) + '])'


def compile_svg(path, name):
    root = ET.fromstring(path.read_bytes())
    width, height = root.attrib['width'], root.attrib['height']
    if root.attrib['viewBox'].split() != ['0', '0', width, height]:
        raise ValueError('Only origin-aligned viewBox matching width/height is supported')
    gradients = {}
    for element in root.iter():
        if element.tag.endswith('}linearGradient'):
            if element.get('gradientUnits') != 'userSpaceOnUse' or element.get('x1') != element.get('x2'):
                raise ValueError('Only vertical user-space linear gradients are supported')
            stops = list(element)
            if len(stops) != 2 or [s.get('offset') for s in stops] != ['0', '1']:
                raise ValueError('Gradient must have stops at 0 and 1')
            gradients[element.attrib['id']] = (element.attrib['y1'], element.attrib['y2'], stops[0].attrib['stop-color'], stops[1].attrib['stop-color'])
    layers = []
    def visit(element, inherited):
        tag = element.tag.rsplit('}', 1)[-1]
        if tag in ('defs', 'title', 'desc'):
            return
        attributes = inherited | element.attrib
        if 'transform' in attributes or 'style' in attributes:
            raise ValueError('SVG transforms/styles must be resolved into explicit geometry')
        if tag in ('svg', 'g'):
            for child in element:
                visit(child, attributes)
            return
        def n(key, default='0'):
            return number(element.get(key, default))
        if tag == 'rect':
            shape = f'VectorShape::Rect {{ x:{n("x")}, y:{n("y")}, width:{n("width")}, height:{n("height")}, radius:{n("rx")} }}'
        elif tag == 'circle':
            shape = f'VectorShape::Circle {{ x:{n("cx")}, y:{n("cy")}, radius:{n("r")} }}'
        elif tag == 'line':
            shape = f'VectorShape::Line({n("x1")},{n("y1")},{n("x2")},{n("y2")})'
        elif tag == 'polygon':
            values = element.attrib['points'].replace(',', ' ').split()
            if len(values) < 6 or len(values) % 2:
                raise ValueError('Invalid polygon')
            shape = 'VectorShape::Polygon(&[' + ','.join(f'({number(values[i])},{number(values[i+1])})' for i in range(0,len(values),2)) + '])'
        elif tag == 'path':
            shape = path_commands(element.attrib['d'])
        else:
            raise ValueError(f'Unsupported element {tag}')
        def paint(key):
            value = attributes.get(key, 'none' if key == 'stroke' else '#000000')
            opacity = float(attributes.get('opacity', 1)) * float(attributes.get(f'{key}-opacity', 1))
            if not 0 <= opacity <= 1:
                raise ValueError('Invalid SVG opacity')
            if value.startswith('url(#') and value.endswith(')'):
                y1, y2, top, bottom = gradients[value[5:-1]]
                # The SVGs' gradients are opaque; enforce this rather than losing alpha.
                if opacity != 1:
                    raise ValueError('Gradient opacity must be baked into its stops')
                return 'Some(VectorPaint::VerticalGradient {' + f'y1:{number(y1)},y2:{number(y2)},top:Color::from_hex(0x{top[1:]}),bottom:Color::from_hex(0x{bottom[1:]})' + '})'
            return color(value,opacity)
        cap = {'butt':'Butt','round':'Round','square':'Square'}[attributes.get('stroke-linecap','butt')]
        join = {'miter':'Miter','round':'Round','bevel':'Bevel'}[attributes.get('stroke-linejoin','miter')]
        rule = {'nonzero':'Winding','evenodd':'EvenOdd'}[attributes.get('fill-rule','nonzero')]
        layers.append(f'VectorLayer {{shape:{shape},fill:{paint("fill")},stroke:{paint("stroke")},stroke_width:{number(attributes.get("stroke-width",1))},line_cap:LineCap::{cap},line_join:LineJoin::{join},fill_rule:FillRule::{rule}}}')
    visit(root,{})
    rust = f'pub static {name}: VectorIcon = VectorIcon {{width:{number(width)},height:{number(height)},layers:&[' + ','.join(layers) + ']};\n'
    return rust, len(layers)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    parts=['// Generated by scripts/generate-vector-icons.py. MIT. Do not edit by hand.\n',
           'use super::{Color, VectorCommand, VectorIcon, VectorLayer, VectorPaint, VectorShape};\n',
           'use crate::tiny_gfx::{FillRule, LineCap, LineJoin};\n']
    entries=[]
    for directory,prefix in [('assets/app_icons/source','DESKTOP_'),('assets/ui_icons/source','UI_')]:
        for source in sorted((ROOT/directory).glob('*.svg')):
            name=prefix+source.stem.replace('-','_').upper()
            rust,layers=compile_svg(source,name)
            parts.append(rust)
            entries.append({'name':name,'source':str(source.relative_to(ROOT)),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'layers':layers})
    parts.append('pub static ALL_VECTOR_ICONS: &[(&str, &VectorIcon)] = &[' + ','.join(f'("{e["name"]}",&{e["name"]})' for e in entries)+'];\n')
    formatted=subprocess.run(['rustfmt','--edition','2021','--emit','stdout'],input=''.join(parts),text=True,capture_output=True,check=True).stdout.encode()
    manifest={'schema':'p4desk.vector-icons.v1','license':'MIT','runtime':'static SVG geometry to tiny_gfx coverage rasterizer','vertical_coverage_samples':8,'bitmap_bytes_embedded':0,'generated_file':str(OUTPUT.relative_to(ROOT)),'generated_sha256':hashlib.sha256(formatted).hexdigest(),'icons':entries}
    metadata=(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n').encode()
    for path,data in [(OUTPUT,formatted),(MANIFEST,metadata)]:
        if args.check:
            if not path.is_file() or path.read_bytes()!=data:
                raise SystemExit(f'矢量资源需要重新生成：{path.relative_to(ROOT)}')
        else:
            path.write_bytes(data)
    print(json.dumps({'valid':True,'mode':'check' if args.check else 'generate','icons':len(entries),'generated_bytes':len(formatted),'bitmap_bytes_embedded':0},ensure_ascii=False))


if __name__=='__main__':
    main()
