#! /bin/sh

os=$1
num_of_args=$#

check_args() {
    if [ $num_of_args -ne 1 ]; then
        printf "\033[31mError: wrong number of arguments\033[0m\n"
        print_usage
        exit 1
    fi
}

print_usage() {
    echo "Usage:"
    echo "$0 <os-type>"
    printf \
        "\twhere <os-type> is either 'win' (for Windows) or 'mac' (for macOS)\n"
}

build() {
    if [ "$os" = "win" ]; then
        _build_win
    elif [ "$os" = "mac" ]; then
        _build_mac
    elif [ "$os" = "help" ]; then
        print_usage
    else
        printf "\033[31mError: unknown os type: %s\033[0m\n" "$os"
        exit 1
    fi
}

_build_win() {
    cd gui || exit 1
    yarn build
    cd - || exit 1
    cargo build
}

_build_mac() {
    cd gui || exit 1
    yarn build
    cd - || exit 1
    cargo build
    rm -rf target/debug/Kotoist.vst
    ./osx_vst_bundler.sh Kotoist target/debug/libkotoist.dylib
    mv Kotoist.vst target/debug/
}

check_args
build
