use chrono::{DateTime, Timelike, Utc};
use color_eyre::{eyre::WrapErr, Result};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use redis::{Client, Commands, Connection};
use serde::Serialize;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

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

fn watch(path: &str, redis_connect: &str, redis_list: &str) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
    let client = Client::open(redis_connect).wrap_err("Invalid redis connection")?;
    let mut con = client
        .get_connection()
        .wrap_err("Failed to open redis connection")?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    while let Ok(event) = rx.recv() {
        push_event(event, &mut con, redis_list).wrap_err("Failed to send event to redis")?;
    }
    Ok(())
}

fn push_event(event: DebouncedEvent, con: &mut Connection, list: &str) -> Result<()> {
    match format_event(event) {
        Some(formatted_event) => {
            println!("{}", formatted_event);
            Ok(con.lpush(list, formatted_event)?)
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

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if let [_, path, redis, list] = args.as_slice() {
        watch(path, redis, list)?;
    } else {
        println!("usage: {} <path> <redis_connect> <redis_list>", args[0])
    }
    Ok(())
}
