#!/usr/bin/env python3
import os
import subprocess
from pathlib import Path

os.chdir(os.path.dirname(os.path.abspath(__file__)))

# go install github.com/tdewolff/minify/cmd/minify@latest

for item in Path('src').glob('**/_asset/*.js').__iter__():
    if str(item).endswith('min.js'):
        continue
    output = item.with_name(item.stem + '.min.js')
    subprocess.run(['minify', '-o', output, item])
    print(output)

for item in Path('src').glob('**/_asset/*.json').__iter__():
    if str(item).endswith('min.json'):
        continue
    output = item.with_name(item.stem + '.min.json')
    subprocess.run(['minify', '-o', output, item])
    print(output)

for item in Path('src').glob('**/_asset/*.css').__iter__():
    if str(item).endswith('min.css'):
        continue
    output = item.with_name(item.stem + '.min.css')
    subprocess.run(['minify', '-o', output, item])
    print(output)
