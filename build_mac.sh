#! /bin/sh

cargo build
rm -rf target/debug/Kotoist.vst
./osx_vst_bundler.sh Kotoist target/debug/libkotoist.dylib
mv Kotoist.vst target/debug/
