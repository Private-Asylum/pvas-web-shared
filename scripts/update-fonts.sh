#!/usr/bin/env bash
# Vendors the house typeface, Geist Mono, into Crates/assets/vendor/fonts/geist-mono/.
#
#   scripts/update-fonts.sh <version>     # e.g. 5.3.0
#
# The files come from Fontsource's variable build on npm (@fontsource-variable/geist-mono):
# one variable woff2 per style and subset, weights 100 to 900. Kept: Latin and Latin Extended,
# upright and italic, and the upright symbols subset (box drawing and block elements, which the
# house style draws its cursor and markers with). Crates/assets/house/house.css declares exactly
# those five, with the unicode ranges Fontsource publishes. Geist is OFL-1.1; its licence ships with the files.
set -euo pipefail

VERSION="${1:?usage: scripts/update-fonts.sh <version>}"
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	echo "error: an exact version is required (a range is not a pin)" >&2
	exit 2
fi

DEST="$(cd "$(dirname "$0")/.." && pwd)/Crates/assets/vendor/fonts/geist-mono"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "Fetching @fontsource-variable/geist-mono@$VERSION..."
(cd "$WORK" && npm pack --silent "@fontsource-variable/geist-mono@$VERSION" >/dev/null && tar xzf ./*.tgz)

rm -rf "$DEST"
mkdir -p "$DEST"
for subset in latin latin-ext; do
	for style in normal italic; do
		cp "$WORK/package/files/geist-mono-$subset-wght-$style.woff2" "$DEST/"
	done
done
cp "$WORK/package/files/geist-mono-symbols2-wght-normal.woff2" "$DEST/"
cp "$WORK/package/LICENSE" "$DEST/LICENSE"
printf 'package: @fontsource-variable/geist-mono\nversion: %s\n' "$VERSION" > "$DEST/VENDORED"
echo "Vendored Geist Mono $VERSION into $DEST"
