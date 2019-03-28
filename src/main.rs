use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use redis::{Client, Commands, Connection, RedisResult};
use std::env;
use std::result::Result;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
enum WatchError {
    Notify(notify::Error),
    Redis(redis::RedisError),
}

impl From<notify::Error> for WatchError {
    fn from(err: notify::Error) -> WatchError {
        WatchError::Notify(err)
    }
}

impl From<redis::RedisError> for WatchError {
    fn from(err: redis::RedisError) -> WatchError {
        WatchError::Redis(err)
    }
}

fn watch(path: &str, redis_connect: &str, redis_list: &str) -> Result<(), WatchError> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
    let client = Client::open(redis_connect)?;
    let con = client.get_connection()?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => push_event(event, &con, redis_list)?,
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn push_event(event: DebouncedEvent, con: &Connection, list: &str) -> RedisResult<()> {
    match format_event(event) {
        Some(formatted_event) => {
            println!("{}", formatted_event);
            return con.lpush(list, formatted_event);
        }
        None => Ok(()),
    }
}

fn format_event(event: DebouncedEvent) -> Option<String> {
    match event {
        DebouncedEvent::Write(path)
        | DebouncedEvent::Create(path)
        | DebouncedEvent::Chmod(path) => Some(format!("write|{}", path.to_str()?)),
        DebouncedEvent::Rename(from, to) => {
            Some(format!("rename|{}|{}", from.to_str()?, to.to_str()?))
        }
        DebouncedEvent::Remove(path) => Some(format!("remove|{}", path.to_str()?)),
        _ => None,
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if let [_, path, redis, list] = args.as_slice() {
        if let Err(e) = watch(path, redis, list) {
            println!("error: {:?}", e)
        }
    } else {
        println!("usage: {} <path> <redis_connect> <redis_list>", args[0])
    }
}
