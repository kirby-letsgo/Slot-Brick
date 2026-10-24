# The Mega Drive scenario's card: Blast Test zipped, on the MD shelf beside the plain one.
set -eu
games="$1/Games/MD"
python3 -I brick/sim/make-archive.py zip "$games/Blast Test.md" "$games/Blast Zip.zip"
