#!/usr/bin/env python3
import os
import subprocess
from os.path import basename

top_path = os.path.dirname(os.path.abspath(__file__))

os.chdir(top_path + "/asset/js")

downloads = [
    "https://cdnjs.cloudflare.com/ajax/libs/vue/3.5.18/vue.esm-browser.js"
]

for url in downloads:
    print(url)
    subprocess.run(["rm", basename(url)])
    subprocess.run(["wget", url])
