#!/usr/bin/env python3
"""Normalize the supplied, pinned Numix theme SVGs and compile vector geometry."""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import re
import subprocess
import xml.etree.ElementTree as ET
from fontTools.svgLib.path import parse_path
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.boundsPen import BoundsPen

ROOT = Path(__file__).resolve().parents[1]
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)
IDENTITY = (1, 0, 0, 1, 0, 0)
SCALE = 112 / 46
FRAME = (SCALE, 0, 0, SCALE, 64 - 24*SCALE, 64 - 24*SCALE)

def mul(a, b):
    aa,ab,ac,ad,ae,af = a; ba,bb,bc,bd,be,bf = b
    return (aa*ba+ac*bb,ab*ba+ad*bb,aa*bc+ac*bd,ab*bc+ad*bd,aa*be+ac*bf+ae,ab*be+ad*bf+af)

def point(t, x, y): return t[0]*x+t[2]*y+t[4], t[1]*x+t[3]*y+t[5]
def fmt(n): return f'{n:.6f}'.rstrip('0').rstrip('.') or '0'
def transform(text):
    t=IDENTITY
    for kind,values in re.findall(r'([a-zA-Z]+)\(([^)]*)\)',text or ''):
        v=[float(x) for x in re.split(r'[\s,]+',values.strip())]
        if kind=='matrix' and len(v)==6: m=tuple(v)
        elif kind=='translate': m=(1,0,0,1,v[0],v[1] if len(v)>1 else 0)
        elif kind=='scale': m=(v[0],0,0,v[-1],0,0)
        elif kind=='rotate':
            a=math.radians(v[0]);m=(math.cos(a),math.sin(a),-math.sin(a),math.cos(a),0,0)
            if len(v)==3:m=mul(mul((1,0,0,1,v[1],v[2]),m),(1,0,0,1,-v[1],-v[2]))
        else: raise ValueError(f'Unsupported transform: {kind}')
        t=mul(t,m)
    return t

def attrs(e):
    a=dict(e.attrib)
    for entry in a.pop('style','').split(';'):
        if ':' in entry:
            k,v=entry.split(':',1);a[k.strip()]=v.strip()
    return a

def path(d, t):
    pen=RecordingPen();parse_path(d,TransformPen(pen,t));out=[]
    for cmd,args in pen.value:
        if cmd in ('closePath','endPath'):
            if cmd=='closePath':out.append('Z')
        else:
            verb={'moveTo':'M','lineTo':'L','curveTo':'C','qCurveTo':'Q'}[cmd]
            out.append(verb+' '.join(fmt(v) for pair in args for v in pair))
    return ' '.join(out)

def circle_geometry(d):
    """Recognize authored circle Beziers for the existing analytic AA fast path."""
    pen=RecordingPen();parse_path(d,pen)
    if sum(cmd=='moveTo' for cmd,_ in pen.value)!=1:return None
    bounds=BoundsPen(None);pen.replay(bounds)
    x0,y0,x1,y1=bounds.bounds;radius=(x1-x0)*0.5
    if radius<1 or abs((x1-x0)-(y1-y0))>0.12:return None
    center=((x0+x1)*0.5,(y0+y1)*0.5);samples=[];current=None
    for cmd,points in pen.value:
        if cmd=='moveTo':current=points[0];samples.append(current)
        elif cmd=='curveTo' and len(points)==3:
            a,b,c=points
            for i in range(1,13):
                t=i/12;u=1-t;samples.append(tuple(u*u*u*current[j]+3*u*u*t*a[j]+3*u*t*t*b[j]+t*t*t*c[j] for j in (0,1)))
            current=c
        elif cmd not in ('closePath','endPath'):return None
    if len(samples)<25 or any(abs(math.hypot(x-center[0],y-center[1])-radius)>0.16 for x,y in samples):return None
    return center[0],center[1],radius

def normalize(source, clock):
    root=ET.fromstring(source.read_bytes());out=ET.Element(f'{{{NS}}}svg',width='128',height='128',viewBox='0 0 128 128')
    ET.SubElement(out,f'{{{NS}}}desc').text='Numix Circle derivative, GPL-3.0-or-later. Normalized for P4Desk; original sources retained in third_party/numix-p4desk.'
    defs=ET.SubElement(out,f'{{{NS}}}defs');gradients={e.get('id'):e for e in root.iter() if e.tag.endswith('}linearGradient')}
    removed=0
    def visit(e,parent,style,parent_opacity):
        nonlocal removed
        tag=e.tag.split('}')[-1];local=attrs(e)
        if tag in ('defs','title','desc'):return
        t=mul(parent,transform(local.get('transform')))
        opacity=parent_opacity*float(local.get('opacity',1))
        a=style|{k:v for k,v in local.items() if k not in ('opacity','transform')}
        if tag in ('svg','g'):
            for child in e:visit(child,t,a,opacity)
            return
        # The supplied dial's hand shadow, fixed hands and hubs are replaced by
        # the real clock hands at runtime. Dial, rim and disk stay unchanged.
        d=local.get('d','')
        if clock and (d.startswith(('m23 25','m22.454','m18.055','m26 24','m25 24')) or (tag=='rect' and local.get('x')=='23' and local.get('y')=='23')):
            removed+=1;return
        n=lambda key,default=0:float(local.get(key,default))
        if tag=='path':
            normalized=path(d,t);circle=circle_geometry(normalized)
            if circle:dest=ET.SubElement(out,f'{{{NS}}}circle',cx=fmt(circle[0]),cy=fmt(circle[1]),r=fmt(circle[2]))
            else:dest=ET.SubElement(out,f'{{{NS}}}path',d=normalized)
        elif tag=='circle':
            x,y=point(t,n('cx'),n('cy'));sx=math.hypot(t[0],t[1]);sy=math.hypot(t[2],t[3])
            if abs(sx-sy)>0.001:raise ValueError('Non-uniform circle transform')
            dest=ET.SubElement(out,f'{{{NS}}}circle',cx=fmt(x),cy=fmt(y),r=fmt(n('r')*sx))
        elif tag=='rect':
            x,y=point(t,n('x'),n('y'))
            if abs(t[1])+abs(t[2])>0.0001:raise ValueError('Rotated rectangle unsupported')
            dest=ET.SubElement(out,f'{{{NS}}}rect',x=fmt(x),y=fmt(y),width=fmt(n('width')*t[0]),height=fmt(n('height')*t[3]),rx=fmt(n('rx',n('ry'))*t[0]))
        else:raise ValueError(f'Unsupported element {tag}')
        for key in ('fill','stroke'):
            value=a.get(key,'#000000' if key=='fill' else 'none')
            if value.startswith('url(#'):
                original=gradients[value[5:-1]];g=attrs(original)
                gt=mul(t,transform(g.get('gradientTransform')))
                p1=point(gt,float(g.get('x1',0)),float(g.get('y1',0)));p2=point(gt,float(g.get('x2',0)),float(g.get('y2',0)))
                if abs(p1[0]-p2[0])>0.001:raise ValueError(f'Non-vertical gradient in {source}')
                ident='g'+str(len(defs));grad=ET.SubElement(defs,f'{{{NS}}}linearGradient',id=ident,gradientUnits='userSpaceOnUse',x1='0',x2='0',y1=fmt(p1[1]),y2=fmt(p2[1]))
                for stop in original:
                    st=attrs(stop);ET.SubElement(grad,f'{{{NS}}}stop',offset=st['offset'],attrib={'stop-color':st['stop-color']})
                value=f'url(#{ident})'
            dest.set(key,value)
            if key+'-opacity' in a:dest.set(key+'-opacity',a[key+'-opacity'])
        if opacity!=1:dest.set('opacity',fmt(opacity))
        if dest.get('stroke')!='none':dest.set('stroke-width',fmt(float(a.get('stroke-width',1))*math.sqrt(abs(t[0]*t[3]-t[1]*t[2]))))
        for key in ('fill-rule','stroke-linejoin','stroke-linecap'):
            if key in a:dest.set(key,a[key])
    visit(root,FRAME,{},1)
    if clock and removed!=6:raise ValueError('Clock source changed; verify fixed-hand layers')
    ET.indent(out)
    return ET.tostring(out,encoding='utf-8',xml_declaration=True)+b'\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    spec=importlib.util.spec_from_file_location('owned_icons',ROOT/'scripts/generate-vector-icons.py');compiler=importlib.util.module_from_spec(spec);spec.loader.exec_module(compiler)
    parts=['// Generated Numix Circle derivatives. GPL-3.0-or-later; see third_party/numix-p4desk.\n','use super::{Color, VectorCommand, VectorIcon, VectorLayer, VectorPaint, VectorShape};\n','use crate::tiny_gfx::{FillRule, LineCap, LineJoin};\n'];entries=[]
    for theme in ('dark','light'):
        base=ROOT/'third_party/numix-p4desk'/theme;manifest=json.loads((base/'manifest.json').read_text())
        for entry in manifest['icons']:
            if entry['group']=='alternatives':continue
            source=base/entry['file'];assert hashlib.sha256(source.read_bytes()).hexdigest()==entry['sha256']
            data=normalize(source,entry['id']=='clock');dest=ROOT/'assets/numix'/theme/(entry['id']+'.svg')
            if args.check:
                if not dest.exists() or dest.read_bytes()!=data:raise SystemExit(f'Stale normalized source: {dest}')
            else:dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
            name='NUMIX_'+theme.upper()+'_'+entry['id'].replace('-','_').upper();rust,layers=compiler.compile_svg(dest,name);parts.append(rust)
            entries.append({'name':name,'id':entry['id'],'theme':theme,'group':entry['group'],'source':str(source.relative_to(ROOT)),'source_sha256':entry['sha256'],'normalized':str(dest.relative_to(ROOT)),'layers':layers,'license':'GPL-3.0-or-later'})
    parts.append('pub static ALL_NUMIX_ICONS: &[(&str, &VectorIcon)] = &['+','.join(f'("{e["name"]}",&{e["name"]})' for e in entries)+'];\n')
    rust=subprocess.run(['rustfmt','--edition','2021','--emit','stdout'],input=''.join(parts),text=True,capture_output=True,check=True).stdout.encode()
    metadata={'license':'GPL-3.0-or-later','icons':entries,'bitmap_bytes_embedded':0,'generated_sha256':hashlib.sha256(rust).hexdigest()}
    for file,data in [('crates/tiny-flutter/src/graphics/numix_icons_generated.rs',rust),('assets/numix-icons.json',(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n').encode())]:
        dest=ROOT/file
        if args.check:
            if dest.read_bytes()!=data:raise SystemExit(f'Stale generated output: {file}')
        else:dest.write_bytes(data)
    print(json.dumps({'icons':len(entries),'valid':True,'mode':'check' if args.check else 'generate','generated_bytes':len(rust),'bitmap_bytes_embedded':0}))

if __name__=='__main__':main()
