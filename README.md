# Notify Redis

Push filesystem notifications into a redis list

## Usage

```
notify-redis /path/to/watch redis://localhost list_name
``` 

Filesystem events are debounced and merge where applicable (e.g. `touch foo.txt`, `mv foo.txt bar.txt` will result in one write event for `bar.txt`)