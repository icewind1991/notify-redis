# Notify Redis

[![Build Status](https://travis-ci.org/icewind1991/notify-redis.svg?branch=master)](https://travis-ci.org/icewind1991/notify-redis)

Push filesystem notifications into a redis list

## Getting the binary

There are 3 ways for getting the binary to run

- Grab a pre-compiled static binary from the [releases](https://github.com/icewind1991/notify-redis/releases) page.
- By running `nix build` to use docker to build a static binary (requires `nix`)
- By running `cargo build` (requires `rust`)

## Usage

```
notify-redis /path/to/watch redis://localhost list_name
``` 

The recorded filesystem events will be pushed to the configured list.
Details about how events are encoded can be found [here](https://github.com/icewind1991/nc-fs-events/)

Filesystem events are debounced and merge where applicable (e.g. `touch foo.txt`, `mv foo.txt bar.txt` will result in one write event for `bar.txt`)
