#!/usr/bin/env python3
"""Compatibility entry point: desktop icons now compile to vector geometry."""
from pathlib import Path
import runpy

if __name__ == '__main__':
    runpy.run_path(str(Path(__file__).with_name('generate-vector-icons.py')), run_name='__main__')
