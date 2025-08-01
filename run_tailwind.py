#!/usr/bin/env python3
import os
import subprocess
from pathlib import Path

os.chdir(os.path.dirname(os.path.abspath(__file__)))

minify = False
if os.getenv('MINIFY') == "true":
    minify = True

map_css: dict = {
    "src/_asset/tailwind/main.css": "src/_asset/main.css"
}

for key, value in map_css.items():
    if minify:
        value = Path(value)
        value = value.with_name(value.stem + '.min.css')
        subprocess.run(['npx', '@tailwindcss/cli', '-i', key, '-o', value, '--minify'])
    else:
        subprocess.run(['npx', '@tailwindcss/cli', '-i', key, '-o', value])
    print(f"{key} -> {value}")
