#!/usr/bin/env python3
import os
import subprocess
import datetime

os.chdir(os.path.dirname(os.path.abspath(__file__)))

http_last_modified = datetime.datetime.now(datetime.UTC).strftime("%a, %d %b %Y %H:%M:%S GMT")

cmds: list = [
    ["./run_tailwind.py"],
    ["./run_minify.py"],
    ["cargo", "build", "--release", "--config", f"env.LAST_MODIFIED_STAMP='{http_last_modified}'"]
]

for cmd in cmds:
    subprocess.run(cmd, env=os.environ | {
        "MINIFY": "true"
    })
