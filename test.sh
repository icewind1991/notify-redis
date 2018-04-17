#!/bin/bash

function assert_notify {
    actual="$(redis-cli rpop notify)"
    if [ "$actual" != "$1" ]
    then
        echo "'$actual' not equal to expected '$1'"
        killall $(basename $bin)
        exit 1
    fi
}

mkdir -p test
rm test/*
redis-cli del notify

bin=$1;
: ${bin:="target/x86_64-unknown-linux-musl/release/notify-redis"}

echo "running tests with $bin"

$bin "$PWD/test" redis://localhost notify &

sleep 1

echo foo > test/foo.txt
sleep 2

assert_notify "write|$(pwd)/test/foo.txt"
assert_notify ''

mv test/foo.txt test/bar.txt
sleep 2

assert_notify "rename|$(pwd)/test/foo.txt|$(pwd)/test/bar.txt"
assert_notify ""

rm test/bar.txt
echo asd > test/bar.txt

sleep 2

assert_notify "write|$(pwd)/test/bar.txt"
assert_notify ""

rm test/bar.txt
sleep 2
echo asd > test/bar.txt

sleep 2

assert_notify "remove|$(pwd)/test/bar.txt"
assert_notify "write|$(pwd)/test/bar.txt"
assert_notify ""

killall $(basename $bin)