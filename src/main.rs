use chrono::{DateTime, Timelike, Utc};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use redis::{Client, Commands, Connection, RedisResult};
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::path::PathBuf;
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

impl Display for WatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WatchError::Redis(err) => err.fmt(f),
            WatchError::Notify(err) => err.fmt(f),
        }
    }
}

impl Error for WatchError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            WatchError::Redis(err) => Some(err),
            WatchError::Notify(err) => Some(err),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
enum Event {
    Modify {
        path: PathBuf,
        time: DateTime<Utc>,
    },
    Move {
        from: PathBuf,
        to: PathBuf,
        time: DateTime<Utc>,
    },
    Delete {
        path: PathBuf,
        time: DateTime<Utc>,
    },
    None,
}

impl From<DebouncedEvent> for Event {
    fn from(event: DebouncedEvent) -> Self {
        let time = Utc::now().with_nanosecond(0).unwrap();

        match event {
            DebouncedEvent::Write(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Chmod(path) => Event::Modify { path, time },
            DebouncedEvent::Rename(from, to) => Event::Move { from, to, time },
            DebouncedEvent::Remove(path) => Event::Delete { path, time },
            _ => Event::None,
        }
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
            Err(e) => println!("watch error: {}", e),
        }
    }
}

fn push_event(event: DebouncedEvent, con: &Connection, list: &str) -> RedisResult<()> {
    match format_event(event) {
        Some(formatted_event) => {
            println!("{}", formatted_event);
            con.lpush(list, formatted_event)
        }
        None => Ok(()),
    }
}

fn format_event(event: DebouncedEvent) -> Option<String> {
    let event: Event = event.into();
    match &event {
        Event::None => None,
        _ => serde_json::to_string(&event).ok(),
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if let [_, path, redis, list] = args.as_slice() {
        if let Err(e) = watch(path, redis, list) {
            println!("error: {}", e)
        }
    } else {
        println!("usage: {} <path> <redis_connect> <redis_list>", args[0])
    }
}
