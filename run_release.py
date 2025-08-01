#!/usr/bin/env python3
import os
import subprocess
import datetime
import uuid

os.chdir(os.path.dirname(os.path.abspath(__file__)))

etag = str(uuid.uuid4())

http_last_modified = datetime.datetime.now(datetime.UTC).strftime("%a, %d %b %Y %H:%M:%S GMT")

cmds: list = [
    ["./run_tailwind.py"],
    ["./run_minify.py"],
    ["cargo", "build", "--release", "--config", f"env.ETAG='{etag}'"]
]

for cmd in cmds:
    subprocess.run(cmd, env=os.environ | {
        "MINIFY": "true"
    })
