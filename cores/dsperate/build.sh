#!/bin/sh
# Build DSperate's libretro core (Nintendo DS) for the device.
#
# Two pinned sources: dsperate-lr, the libretro wrapper (thwonp/dsperate-lr),
# and the DSperate commit it pins as its submodule (beebono/DSperate). Both are
# GPL-3.0. DSperate's own CMake builds its core library, then the wrapper links
# against it, as dsperate-lr's build.sh does. The wrapper's patches onto
# DSperate are applied first.
#
# Built natively in the bullseye box the other cores use, so it runs on the
# device's libraries. DSperate's committed PGO profiles match only the exact
# GCC they were made with (10.5, 12.4, 13.3), not bullseye's 10.2, so this
# build goes without one. The Vulkan presenter, SDL and headless frontends,
# tests, RetroAchievements and Wi-Fi are left out: the core needs none of them.
set -eu

here="$(cd "$(dirname "$0")" && pwd)"

device_tune="-mcpu=cortex-a53"

usage() {
	echo "usage: $0 stamp LR_COMMIT DS_COMMIT | build LR_COMMIT DS_COMMIT LR_TARBALL DS_TARBALL WORKDIR OUT" >&2
	exit 2
}

sha256() {
	if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi |
		cut -d' ' -f1
}

stamp() {
	echo "commit=$1"
	echo "source=https://github.com/thwonp/dsperate-lr/archive/$1.tar.gz"
	echo "dsperate_commit=$2"
	echo "dsperate_source=https://github.com/beebono/DSperate/archive/$2.tar.gz"
	echo "recipe=cmake DSperate (dsperate_core, PGO off, Vulkan off) then the wrapper"
	echo "device_tune=$device_tune"
	for p in "$here"/*.patch; do
		[ -e "$p" ] || continue
		echo "patch=$(basename "$p") sha256:$(sha256 "$p")"
	done
}

build() {
	lr_commit="$1" ds_commit="$2" lr_tarball="$3" ds_tarball="$4" work="$5" out="$6"
	lr="$work/dsperate-lr-$lr_commit"
	ds="$work/DSperate-$ds_commit"

	rm -rf "$work"
	mkdir -p "$work"
	tar -xzf "$lr_tarball" -C "$work"
	tar -xzf "$ds_tarball" -C "$work"
	for f in "$lr/CMakeLists.txt" "$ds/CMakeLists.txt"; do
		if [ ! -f "$f" ]; then
			echo "missing $f: the tarballs are not dsperate-lr-$lr_commit and DSperate-$ds_commit" >&2
			exit 1
		fi
	done
	# The wrapper expects DSperate as its DSperate/ submodule.
	rm -rf "$lr/DSperate"
	mv "$ds" "$lr/DSperate"

	# The wrapper's own fixes onto DSperate, then any of ours.
	for p in "$lr"/patches/*.patch "$here"/*.patch; do
		[ -e "$p" ] || continue
		(cd "$lr/DSperate" && patch -p1 --forward --quiet < "$p")
	done

	cmake -S "$lr/DSperate" -B "$work/ds-build" \
		-DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_PROJECT_INCLUDE="$lr/cmake/dsperate-pic.cmake" \
		-DDSPERATE_PGO=OFF -DDSPERATE_TUNE="$device_tune" \
		-DDSPERATE_TESTS=OFF -DDSPERATE_HEADLESS=OFF -DDSPERATE_SDL=OFF \
		-DDSPERATE_WAYLAND=OFF -DDSPERATE_VULKAN=OFF \
		-DDSPERATE_CHEEVOS=OFF -DDSPERATE_NET=OFF
	cmake --build "$work/ds-build" --target dsperate_core -j"$(getconf _NPROCESSORS_ONLN)"

	cmake -S "$lr" -B "$work/lr-build" \
		-DCMAKE_BUILD_TYPE=Release -DDSPERATE_BUILD="$work/ds-build"
	cmake --build "$work/lr-build" -j"$(getconf _NPROCESSORS_ONLN)"

	mkdir -p "$(dirname "$out")"
	strip --strip-unneeded -o "$out" "$work/lr-build/dsperate_libretro.so"
	stamp "$lr_commit" "$ds_commit" >"$out.meta"
}

case "${1:-}" in
stamp)
	[ $# -eq 3 ] && [ -n "$2" ] && [ -n "$3" ] || usage
	stamp "$2" "$3"
	;;
build)
	[ $# -eq 7 ] || usage
	build "$2" "$3" "$4" "$5" "$6" "$7"
	;;
*)
	usage
	;;
esac
