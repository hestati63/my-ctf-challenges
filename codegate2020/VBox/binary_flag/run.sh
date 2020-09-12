#!/bin/sh

timeout 20000 stdbuf -i 0 -o 0 -e 0 python3 /home/vbox/glue.py 2>/dev/null
