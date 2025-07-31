#!/usr/bin/env python3
import os
import subprocess

os.chdir(os.path.dirname(os.path.abspath(__file__)))

map_css: dict = {
    "src/_asset/tailwind/main.css": "src/_asset/main.css"
}

for key, value in map_css.items():
    subprocess.run(['npx', '@tailwindcss/cli', '-i', key, '-o', value])
    print(f"{key} -> {value}")
