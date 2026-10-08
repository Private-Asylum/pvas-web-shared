#!/usr/bin/env bash
# Vendors a pinned build of Yeti (https://github.com/foundation/yeti) into vendor/yeti/.
#
#   scripts/update-yeti.sh <40-char commit sha>
#
# Yeti has no npm release yet and does not commit its dist/, so the only way to
# use it is to build it at a commit. This does that in a throwaway directory
# and copies the result here; the build output is what gets committed, anchored
# on the sha recorded in vendor/yeti/VENDORED. CI never touches Yeti's toolchain.
# Once Yeti publishes `yeti-css` on npm, replace vendor/yeti with the package.
set -euo pipefail

SHA="${1:?usage: scripts/update-yeti.sh <commit-sha>}"
if [[ ! "$SHA" =~ ^[0-9a-f]{40}$ ]]; then
	echo "error: a full 40-character commit sha is required (a branch or short sha is not a pin)" >&2
	exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)/Crates/assets"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

echo "Fetching foundation/yeti@$SHA..."
git -C "$WORK" init -q
git -C "$WORK" remote add origin https://github.com/foundation/yeti.git
git -C "$WORK" fetch -q --depth 1 origin "$SHA"
git -C "$WORK" checkout -q FETCH_HEAD

echo "Building..."
(
	cd "$WORK"
	# The build needs esbuild and lightningcss only; skip the test browsers.
	PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm ci --no-audit --no-fund >/dev/null
	npm run build >/dev/null
)

rm -rf "$ROOT/vendor/yeti"
mkdir -p "$ROOT/vendor/yeti"
cp -R "$WORK/dist/." "$ROOT/vendor/yeti/"
cp "$WORK/LICENSE" "$ROOT/vendor/yeti/LICENSE"

VERSION="$(node -p "require('$WORK/package.json').version")"
cat > "$ROOT/vendor/yeti/VENDORED" <<INFO
source:  https://github.com/foundation/yeti
commit:  $SHA
version: $VERSION
license: FSL-1.1-MIT (see LICENSE)
INFO

echo "Vendored Yeti $VERSION ($SHA) into vendor/yeti/"
