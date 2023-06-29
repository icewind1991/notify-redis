use chrono::{DateTime, Timelike, Utc};
use color_eyre::{eyre::WrapErr, Result};
use notify::event::{ModifyKind, RenameMode};
use notify::{EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use redis::{Client, Commands, Connection, IntoConnectionInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event")]
#[serde(rename_all = "snake_case")]
pub enum Event {
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

        let path_count = event.paths.len();
        let mut paths = event.event.paths.into_iter();

        match (event.event.kind, path_count) {
            (EventKind::Modify(ModifyKind::Name(RenameMode::Both)), 2..) => Event::Move {
                from: paths.next().unwrap(),
                to: paths.next().unwrap(),
                time,
            },
            (EventKind::Modify(_) | EventKind::Create(_), 1..) => Event::Modify {
                path: paths.next().unwrap(),
                time,
            },
            (EventKind::Remove(_), 1..) => Event::Delete {
                path: paths.next().unwrap(),
                time,
            },
            _ => Event::None,
        }
    }
}

pub fn watch(
    path: impl AsRef<Path>,
    redis_connect: impl IntoConnectionInfo,
    redis_list: &str,
    debounce: Duration,
) -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = new_debouncer(debounce, None, tx)?;
    let client = Client::open(redis_connect).wrap_err("Invalid redis connection")?;
    let mut con = client
        .get_connection()
        .wrap_err("Failed to open redis connection")?;

    watcher
        .watcher()
        .watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Ok(event) = rx.recv() {
        for event in event.into_iter().flatten() {
            push_event(event, &mut con, redis_list).wrap_err("Failed to send event to redis")?;
        }
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
