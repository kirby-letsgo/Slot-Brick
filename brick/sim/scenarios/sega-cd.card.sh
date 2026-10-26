#!/bin/sh
set -eu
# A generated test BIOS only, never included in the real card package.
python3 brick/sim/make-segacd.py "$1"
