#!/bin/sh
# One-line installer for wsctl, the WormholeSystems setup tool:
#
#   curl --proto '=https' --tlsv1.2 -sSf https://install.wormhole.systems | sh
#
# Pin a version with WSCTL_VERSION=wsctl-v0.6.0. Choose where it lands with WSCTL_BIN_DIR.
# The download is checked against the release's SHA256SUMS; a release from before those
# were published (wsctl 0.5 and older) needs WSCTL_SKIP_VERIFY=1.
set -eu

REPO="${WSCTL_REPO:-wormholesystems/wormholesystems}"
VERSION="${WSCTL_VERSION:-latest}"

case "$(uname -s)-$(uname -m)" in
	Linux-x86_64)  TARGET=x86_64-unknown-linux-gnu ;;
	Linux-aarch64) TARGET=aarch64-unknown-linux-gnu ;;
	Darwin-arm64)  TARGET=aarch64-apple-darwin ;;
	Darwin-x86_64) TARGET=x86_64-apple-darwin ;;
	*)
		echo "wsctl has no released build for $(uname -s) $(uname -m)." >&2
		echo "Build it from a checkout instead:" >&2
		echo "  cargo run -p wsctl -- setup" >&2
		exit 1
		;;
esac

ASSET="wsctl-$TARGET"
SUMS=SHA256SUMS
TOKEN="${WSCTL_TOKEN:-${GITHUB_TOKEN:-${GH_TOKEN:-}}}"

# A public release is a plain download. A private one is not reachable that way at all, so
# with a token we go through the API instead: find the asset by name, then ask for its bytes.
if [ -n "$TOKEN" ]; then
	if [ "$VERSION" = latest ]; then
		RELEASE="https://api.github.com/repos/$REPO/releases/latest"
	else
		RELEASE="https://api.github.com/repos/$REPO/releases/tags/$VERSION"
	fi
	RELEASE_JSON="$(
		curl -fsSL -H "Authorization: Bearer $TOKEN" \
			-H "Accept: application/vnd.github+json" "$RELEASE" | tr -d '\n'
	)"
	# The response is pretty-printed, so it is flattened first: one asset object per line
	# puts each id beside the name it belongs to. GitHub lists `name` before the nested
	# `uploader`, so splitting on the brace keeps the two together.
	asset_url() {
		id="$(
			printf '%s' "$RELEASE_JSON" | tr '{' '\n' |
				grep "\"name\"[[:space:]]*:[[:space:]]*\"$1\"" |
				head -1 |
				sed -n 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9][0-9]*\).*/\1/p'
		)"
		[ -n "$id" ] && echo "https://api.github.com/repos/$REPO/releases/assets/$id"
	}
	fetch() {
		curl -fsSL -H "Authorization: Bearer $TOKEN" -H "Accept: application/octet-stream" \
			-o "$1" "$2"
	}
	if ! URL="$(asset_url "$ASSET")"; then
		echo "No asset named $ASSET in $VERSION of $REPO." >&2
		echo "Check the release exists and the token can read the repository." >&2
		exit 1
	fi
	SUMS_URL="$(asset_url "$SUMS" || true)"
else
	if [ "$VERSION" = latest ]; then
		BASE="https://github.com/$REPO/releases/latest/download"
	else
		BASE="https://github.com/$REPO/releases/download/$VERSION"
	fi
	URL="$BASE/$ASSET"
	SUMS_URL="$BASE/$SUMS"
	fetch() { curl -fsSL -o "$1" "$2"; }
fi

# /usr/local/bin when it is ours to write to, so it is already on PATH.
if [ -n "${WSCTL_BIN_DIR:-}" ]; then
	BIN_DIR="$WSCTL_BIN_DIR"
elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
	BIN_DIR=/usr/local/bin
else
	BIN_DIR="$HOME/.local/bin"
fi
mkdir -p "$BIN_DIR"

echo "Downloading $URL"
# To a temporary directory first: a half-written or unverified binary in place is worse
# than none.
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if ! fetch "$TMP/$ASSET" "$URL"; then
	echo "Download failed." >&2
	echo "If $REPO is private, set WSCTL_TOKEN to a token that can read it." >&2
	exit 1
fi

# The sums come from the same release, so this catches a corrupted or cut-short download
# and a wrong asset, not a release that was tampered with as a whole; that would take a
# signature.
if [ "${WSCTL_SKIP_VERIFY:-}" = 1 ]; then
	echo "Not verifying the download (WSCTL_SKIP_VERIFY=1)."
else
	if [ -z "$SUMS_URL" ] || ! fetch "$TMP/$SUMS" "$SUMS_URL"; then
		echo "$VERSION of $REPO publishes no $SUMS to check the download against." >&2
		echo "Releases before wsctl 0.6 did not; WSCTL_SKIP_VERIFY=1 installs one anyway." >&2
		exit 1
	fi
	if command -v sha256sum >/dev/null 2>&1; then
		ACTUAL="$(sha256sum "$TMP/$ASSET" | cut -d' ' -f1)"
	elif command -v shasum >/dev/null 2>&1; then
		ACTUAL="$(shasum -a 256 "$TMP/$ASSET" | cut -d' ' -f1)"
	else
		echo "Neither sha256sum nor shasum is here to check the download with." >&2
		exit 1
	fi
	EXPECTED="$(awk -v f="$ASSET" '$2 == f || $2 == "*" f { print $1 }' "$TMP/$SUMS")"
	if [ -z "$EXPECTED" ] || [ "$ACTUAL" != "$EXPECTED" ]; then
		echo "The download does not match its checksum in $SUMS; not installing it." >&2
		echo "  expected ${EXPECTED:-(no entry for $ASSET)}" >&2
		echo "  got      $ACTUAL" >&2
		exit 1
	fi
	echo "Checksum verified."
fi

chmod +x "$TMP/$ASSET"
mv "$TMP/$ASSET" "$BIN_DIR/wsctl"

echo "Installed $("$BIN_DIR/wsctl" --version) to $BIN_DIR/wsctl"

case ":$PATH:" in
	*":$BIN_DIR:"*) ;;
	*) echo "Note: $BIN_DIR is not on your PATH." ;;
esac

# stdin is the pipe this script arrived through, so anything interactive has to read the
# terminal itself. With no terminal there is nothing to prompt, and that is fine.
[ -r /dev/tty ] || exit 0

# Braced so a /dev/tty that exists but cannot be opened fails quietly: the redirect's own
# error is the shell's, not the read's, and would otherwise print before the fallback.
ask() {
	printf '%s [Y/n] ' "$1"
	{ read -r answer < /dev/tty; } 2>/dev/null || answer=n
	case "$answer" in
		[nN]*) return 1 ;;
		*) return 0 ;;
	esac
}

# The setup works on a checkout, and running it anywhere else only gets as far as saying so.
if [ -f docker-compose.yml ] && [ -d deploy ]; then
	if ask "Run the setup now, in $(pwd)?"; then
		exec "$BIN_DIR/wsctl" setup < /dev/tty
	fi
	echo "Run \`wsctl setup\` whenever you are ready."
	exit 0
fi

CHECKOUT="$(basename "$REPO")"
if [ -e "$CHECKOUT" ]; then
	echo "Then, in your checkout:  wsctl setup"
	exit 0
fi
if ! command -v git >/dev/null 2>&1; then
	echo "Then clone https://github.com/$REPO and run \`wsctl setup\` inside it."
	exit 0
fi
if ask "Clone $REPO into ./$CHECKOUT and run the setup there?"; then
	git clone "https://github.com/$REPO.git" "$CHECKOUT"
	cd "$CHECKOUT"
	exec "$BIN_DIR/wsctl" setup < /dev/tty
fi
echo "Then clone https://github.com/$REPO and run \`wsctl setup\` inside it."
