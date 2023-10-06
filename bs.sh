#!/usr/bin/env sh

cargo b -p iec61131std
cp ./target/debug/libiec61131std.so .
export LD_LIBRARY_PATH=.
# cargo r -- demo.st -liec61131std -Ltarget/debug/ -i libs/stdlib/iec61131-st/assertion.st --linker=clang
cargo r -- demo.st -liec61131std -L. -i libs/stdlib/iec61131-st/assertion.st --linker=cc --test
# ./demo.st.out
# echo $?
rm libiec61131std.so
