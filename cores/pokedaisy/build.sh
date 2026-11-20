#!/bin/sh
# PokeDaisy's two libretro cores for the PokeDaisy shelf:
#   pokedaisy_gba_libretro.so  Game Boy Advance, on armGBA
#   pokedaisy_gbc_libretro.so  Game Boy and Game Boy Color, on mGBA
# Both come from one source tree: PokeDaisy at a pinned commit with its mGBA, armGBA and zlib
# submodules at the commits that commit pins. GitHub's archives leave submodules out, so
# `assemble` puts the four archives together into one tarball, which ships beside the cores as
# their corresponding source (GPL-3.0, with PokeDaisy's NOTICE).
set -eu

here="$(cd "$(dirname "$0")" && pwd)"

device_cflags="-mcpu=cortex-a53"

# What `assemble` keeps of PokeDaisy's tree: the libretro cores and the code they build from.
# The Android app, the 3DS and DS ports, the website and the rest stay out.
keep="LICENSE NOTICE README.md libretro 3ds/core linux app/src/main/cpp"

usage() {
        echo "usage: $0 stamp COMMITS | assemble COMMITS PD MGBA ARMGBA ZLIB OUT | build COMMITS TARBALL WORKDIR OUTDIR" >&2
        echo "  COMMITS is pokedaisy:mgba:armgba:zlib" >&2
        exit 2
}

sha256() {
        if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1"; else shasum -a 256 "$1"; fi |
                cut -d' ' -f1
}

split() {
        pd="$(echo "$1" | cut -d: -f1)" mgba="$(echo "$1" | cut -d: -f2)"
        armgba="$(echo "$1" | cut -d: -f3)" zlib="$(echo "$1" | cut -d: -f4)"
        [ -n "$pd" ] && [ -n "$mgba" ] && [ -n "$armgba" ] && [ -n "$zlib" ] || usage
}

stamp() {
        split "$1"
        echo "commit=$pd"
        echo "source=https://github.com/lidor30/pokedaisy/archive/$pd.tar.gz"
        echo "mgba=https://github.com/mgba-emu/mgba/archive/$mgba.tar.gz"
        echo "armgba=https://github.com/ratherDashing/armGBA/archive/$armgba.tar.gz"
        echo "zlib=https://github.com/madler/zlib/archive/$zlib.tar.gz"
        echo "recipe=cmake libretro (pokedaisy_gbc_libretro) and libretro/armgba (pokedaisy_gba_libretro), Release"
        echo "device_cflags=$device_cflags"
        for p in "$here"/*.patch; do
                [ -e "$p" ] || continue
                echo "patch=$(basename "$p") sha256:$(sha256 "$p")"
        done
}

assemble() {
        split "$1"
        work="$(mktemp -d)"
        trap 'rm -rf "$work"' EXIT
        tar -xzf "$2" -C "$work"
        tar -xzf "$3" -C "$work"
        tar -xzf "$4" -C "$work"
        tar -xzf "$5" -C "$work"
        top="$work/pokedaisy-$pd"
        tree="$work/pokedaisy"
        mkdir -p "$tree/third_party"
        for k in $keep; do
                mkdir -p "$tree/$(dirname "$k")"
                cp -R "$top/$k" "$tree/$k"
        done
        mv "$work/mgba-$mgba" "$tree/third_party/mgba"
        mv "$work/armGBA-$armgba" "$tree/third_party/armgba"
        mv "$work/zlib-$zlib" "$tree/third_party/zlib"
        stamp "$1" >"$tree/SLOTPLUS-SOURCE.txt"
        (cd "$work" && tar --sort=name --owner=0 --group=0 --numeric-owner --mtime=@0 -cf - pokedaisy) |
                gzip -9n >"$6"
}

build() {
        split "$1"
        tarball="$2" work="$3" out="$4"
        rm -rf "$work"
        mkdir -p "$work" "$out"
        tar -xzf "$tarball" -C "$work"
        src="$work/pokedaisy"
        for p in "$here"/*.patch; do
                [ -e "$p" ] || continue
                git -C "$src" apply -p1 "$p"
        done
        jobs="$(getconf _NPROCESSORS_ONLN)"
        CFLAGS="$device_cflags" CXXFLAGS="$device_cflags" \
                cmake -S "$src/libretro" -B "$work/gbc" -DCMAKE_BUILD_TYPE=Release
        cmake --build "$work/gbc" --target pokedaisy_gbc_libretro -j"$jobs"
        CFLAGS="$device_cflags" CXXFLAGS="$device_cflags" \
                cmake -S "$src/libretro/armgba" -B "$work/gba" -DCMAKE_BUILD_TYPE=Release
        cmake --build "$work/gba" --target pokedaisy_gba_libretro -j"$jobs"
        cp "$work/gbc/pokedaisy_gbc_libretro.so" "$work/gba/pokedaisy_gba_libretro.so" "$out/"
        stamp "$1" >"$out/pokedaisy_libretro.meta"
}

case "${1:-}" in
stamp)
        [ $# -eq 2 ] || usage
        stamp "$2"
        ;;
assemble)
        [ $# -eq 7 ] || usage
        assemble "$2" "$3" "$4" "$5" "$6" "$7"
        ;;
build)
        [ $# -eq 5 ] || usage
        build "$2" "$3" "$4" "$5"
        ;;
*)
        usage
        ;;
esac
