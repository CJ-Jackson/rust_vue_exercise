#!/usr/bin/env python3
import os
import subprocess

os.chdir(os.path.dirname(os.path.abspath(__file__)))

cmds: list = [
    ["./run_tailwind.py"],
    ["./run_minify.py"],
    ["cargo", "build", "--release"]
]

for cmd in cmds:
    if cmd == ["./run_tailwind.py"]:
        subprocess.run(cmd, env=os.environ | {
            "MINIFY": "true"
        })
    else:
        subprocess.run(cmd)
