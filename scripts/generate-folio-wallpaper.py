#!/usr/bin/env python3
"""Compile original P4Desk landscape curves to AA spans, no framebuffer asset.

Folio-inspired palette; geometry is original. Fixed 1024x600 panel. Each 8-byte
record is little-endian y, x, length (u16), alpha (u8), layer (u8).
"""
import argparse
from pathlib import Path
import struct
ROOT=Path(__file__).resolve().parents[1]
W,H=1024,600

def cubic(t,a,b,c,d):
    u=1-t
    return u*u*u*a+3*u*u*t*b+3*u*t*t*c+t*t*t*d

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    samples=[]
    for x in range(W):
        values=[]
        for q in range(4):
            left,right=0.,1.
            for _ in range(30):
                t=(left+right)*.5
                if cubic(t,-20,W*.26,W*.52,W+20)<x+(q+.5)/4: left=t
                else: right=t
            values.append(cubic((left+right)*.5,0,-.34,.38,-.05)*H)
        samples.append(values)
    data=bytearray()
    for layer,level in enumerate([.30,.46,.63,.82]):
        heights=[[h+level*H for h in group] for group in samples]
        for y in range(H):
            start=0
            previous=0
            for x in range(W+1):
                alpha=round(sum(max(0.,min(1.,y+1-h)) for h in heights[x])*255/4) if x<W else -1
                if alpha!=previous:
                    if previous>0:data.extend(struct.pack('<HHHBB',y,start,x-start,previous,layer))
                    start=x;previous=alpha
    target=ROOT/'assets/wallpaper/folio-waves.spans'
    if args.check:
        if target.read_bytes()!=data:raise SystemExit('Folio wallpaper spans are stale')
    else:target.write_bytes(data)
    print(f'{len(data)} bytes; {len(data)//8} antialiased spans; no runtime allocation')
if __name__=='__main__':main()
