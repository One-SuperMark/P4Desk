#!/usr/bin/env python3
"""Prefilter the original Folio wallpaper for bounded, repeatable glass rendering."""
import argparse
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
W, H = 256, 150
PALETTES = {
    'dark': ((116,111,116), (62,88,109), [0x797b83,0x525f70,0x354f60,0x253b4d]),
    'light': ((226,231,245), (182,217,232), [0xcbd3e4,0xb4cbdc,0x92b8ce,0x749daf]),
}
def rgb(n): return (n >> 16, (n >> 8) & 255, n & 255)
def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    spans = (ROOT/'assets/wallpaper/folio-waves.spans').read_bytes()
    for name, (left,right,layers) in PALETTES.items():
        # Render the same AA spans at full resolution, then average 4x4 blocks.
        rows = [[tuple(round(a+(b-a)*x/1023) for a,b in zip(left,right)) for x in range(1024)] for _ in range(600)]
        for y,x,n,alpha,layer in struct.iter_unpack('<HHHBB',spans):
            color=rgb(layers[layer])
            for xx in range(x,x+n): rows[y][xx]=tuple((old*(255-alpha)+new*alpha+127)//255 for old,new in zip(rows[y][xx],color))
        small=[tuple(sum(rows[y*4+dy][x*4+dx][c] for dy in range(4) for dx in range(4))//16 for c in range(3)) for y in range(H) for x in range(W)]
        # Separable three-tap filter at quarter resolution (12 px footprint).
        horizontal=[tuple(sum(small[y*W+max(0,min(W-1,x+dx))][c] for dx in (-1,0,1))//3 for c in range(3)) for y in range(H) for x in range(W)]
        filtered=[tuple(sum(horizontal[max(0,min(H-1,y+dy))*W+x][c] for dy in (-1,0,1))//3 for c in range(3)) for y in range(H) for x in range(W)]
        data=bytes(channel for pixel in filtered for channel in pixel)
        target=ROOT/f'assets/wallpaper/glass-{name}.rgb888'
        if args.check:
            if target.read_bytes()!=data: raise SystemExit(f'{target.name} is stale')
        else: target.write_bytes(data)
        print(f'{name}: {len(data)} bytes, {W}x{H}, prefiltered original wallpaper')
if __name__=='__main__': main()
