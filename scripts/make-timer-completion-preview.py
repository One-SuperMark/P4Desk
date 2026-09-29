#!/usr/bin/env python3
"""Create a GIF from synthetic Rust UI animation frames (requires Pillow)."""
import argparse
from pathlib import Path
from PIL import Image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('frames', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--step-ms', type=int, default=33, choices=range(10, 101), metavar='10..100')
    args = parser.parse_args()
    paths = sorted(args.frames.glob('[0-9][0-9][0-9].png'))
    if not paths:
        raise SystemExit('没有找到合成动画帧。')
    frames = [Image.open(path).convert('RGB') for path in paths]
    if any(frame.size != (1024, 600) for frame in frames):
        raise SystemExit('预览帧应为 1024×600。')
    # One palette across all frames avoids GIF palette flicker at the moving glass boundary.
    samples = [frames[index] for index in [0, len(frames) // 4, len(frames) // 3,
                                         len(frames) // 2, 3 * len(frames) // 4, len(frames) - 1]]
    contact = Image.new('RGB', (1024, 600 * len(samples)))
    for index, sample in enumerate(samples):
        contact.paste(sample, (0, 600 * index))
    palette = contact.quantize(colors=256, dither=Image.Dither.NONE)
    encoded = [frame.quantize(palette=palette, dither=Image.Dither.NONE) for frame in frames]
    # Quantize absolute timestamps, preserving total time for 16 or 33 ms input.
    durations = [10 * (round((index + 1) * args.step_ms / 10) - round(index * args.step_ms / 10))
                 for index in range(len(frames))]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    encoded[0].save(args.output, save_all=True, append_images=encoded[1:], duration=durations,
                    loop=0, optimize=False, disposal=1)
    print(f'已生成合成界面动画，{len(frames)} 帧，1024×600。')


if __name__ == '__main__':
    main()
