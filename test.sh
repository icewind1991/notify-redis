#!/bin/bash

function assert_notify {
    actual="$(echo $(redis-cli rpop notify) | sed -e 's/,\"time\":[^Z]*Z\"//g')" # strip time
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

assert_notify "{\"event\":\"modify\",\"path\":\"$(pwd)/test/foo.txt\"}"
assert_notify ''

mv test/foo.txt test/bar.txt
sleep 2

assert_notify "{\"event\":\"move\",\"from\":\"$(pwd)/test/foo.txt\",\"to\":\"$(pwd)/test/bar.txt\"}"
assert_notify ""

rm test/bar.txt
echo asd > test/bar.txt

sleep 2

assert_notify "{\"event\":\"modify\",\"path\":\"$(pwd)/test/bar.txt\"}"
assert_notify ""

rm test/bar.txt
sleep 2
echo asd > test/bar.txt

sleep 2

assert_notify "{\"event\":\"delete\",\"path\":\"$(pwd)/test/bar.txt\"}"
assert_notify "{\"event\":\"modify\",\"path\":\"$(pwd)/test/bar.txt\"}"
assert_notify ""

killall $(basename $bin)