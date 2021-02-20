use notify_redis::{watch, Event};
use redis::{Client, Commands, Connection, ConnectionInfo};
use std::fs::{remove_file, rename, write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use tempfile::tempdir;

fn cleanup(redis: ConnectionInfo, list: &str) {
    let client = Client::open(redis).unwrap();
    let mut con = client.get_connection().unwrap();
    con.del::<_, ()>(list).unwrap();
}

struct EventList {
    redis: Connection,
    list: String,
}

impl EventList {
    fn new(redis: ConnectionInfo, list: &str) -> Self {
        let client = Client::open(redis).unwrap();
        let redis = client.get_connection().unwrap();
        EventList {
            redis,
            list: list.into(),
        }
    }

    fn next(&mut self) -> Option<Event> {
        let raw: Option<String> = self.redis.rpop(&self.list).unwrap();
        raw.map(|raw| serde_json::from_str(&raw).unwrap())
    }
}

fn spawn_watch(
    path: &Path,
    redis_connect: ConnectionInfo,
    list: &str,
) -> std::thread::JoinHandle<()> {
    let path = path.to_path_buf();
    let list = list.to_string();
    std::thread::spawn(move || {
        if let Err(e) = watch(path, redis_connect, &list, Duration::from_millis(1)) {
            eprintln!("watch error {:#}", e);
        }
    })
}

#[test]
fn test_basic() {
    let list = format!("notify_redis_test_{}", rand::random::<u16>());
    let redis_connect: ConnectionInfo = "redis://localhost".parse().unwrap();
    cleanup(redis_connect.clone(), &list);
    let dir = tempdir().unwrap();
    let mut event_list = EventList::new(redis_connect.clone(), &list);
    spawn_watch(dir.path(), redis_connect.clone(), &list);

    sleep(Duration::from_millis(10));

    write(dir.path().join("foo.txt"), "foo").unwrap();

    sleep(Duration::from_millis(10));

    assert!(
        matches!(event_list.next(), Some(Event::Modify {path ,..}) if path.ends_with("foo.txt"))
    );
    assert!(matches!(event_list.next(), None));
}

#[test]
fn test_rename_debounce() {
    let list = format!("notify_redis_test_{}", rand::random::<u16>());
    let redis_connect: ConnectionInfo = "redis://localhost".parse().unwrap();
    cleanup(redis_connect.clone(), &list);
    let dir = tempdir().unwrap();
    let mut event_list = EventList::new(redis_connect.clone(), &list);
    spawn_watch(dir.path(), redis_connect.clone(), &list);

    sleep(Duration::from_millis(10));

    write(dir.path().join("foo.txt"), "foo").unwrap();
    rename(dir.path().join("foo.txt"), dir.path().join("bar.txt")).unwrap();

    sleep(Duration::from_millis(10));

    assert!(
        matches!(event_list.next(), Some(Event::Modify {path ,..}) if path.ends_with("bar.txt"))
    );
    assert!(matches!(event_list.next(), None));
}

#[test]
fn test_rename() {
    let list = format!("notify_redis_test_{}", rand::random::<u16>());
    let redis_connect: ConnectionInfo = "redis://localhost".parse().unwrap();
    cleanup(redis_connect.clone(), &list);
    let dir = tempdir().unwrap();
    let mut event_list = EventList::new(redis_connect.clone(), &list);
    spawn_watch(dir.path(), redis_connect.clone(), &list);

    sleep(Duration::from_millis(10));

    write(dir.path().join("foo.txt"), "foo").unwrap();
    sleep(Duration::from_millis(10));
    rename(dir.path().join("foo.txt"), dir.path().join("bar.txt")).unwrap();

    sleep(Duration::from_millis(10));

    assert!(
        matches!(event_list.next(), Some(Event::Modify {path ,..}) if path.ends_with("foo.txt"))
    );
    assert!(
        matches!(event_list.next(), Some(Event::Move {from, to ,..}) if from.ends_with("foo.txt") && to.ends_with("bar.txt"))
    );
    assert!(matches!(dbg!(event_list.next()), None));
}

#[test]
fn test_delete() {
    let list = format!("notify_redis_test_{}", rand::random::<u16>());
    let redis_connect: ConnectionInfo = "redis://localhost".parse().unwrap();
    cleanup(redis_connect.clone(), &list);
    let dir = tempdir().unwrap();
    let mut event_list = EventList::new(redis_connect.clone(), &list);
    spawn_watch(dir.path(), redis_connect.clone(), &list);

    sleep(Duration::from_millis(10));

    write(dir.path().join("foo.txt"), "foo").unwrap();
    sleep(Duration::from_millis(10));
    remove_file(dir.path().join("foo.txt")).unwrap();

    sleep(Duration::from_millis(10));

    assert!(
        matches!(event_list.next(), Some(Event::Modify {path ,..}) if path.ends_with("foo.txt"))
    );
    assert!(
        matches!(event_list.next(), Some(Event::Delete {path ,..}) if path.ends_with("foo.txt"))
    );
    assert!(matches!(dbg!(event_list.next()), None));
}

#[test]
fn test_delete_debounce() {
    let list = format!("notify_redis_test_{}", rand::random::<u16>());
    let redis_connect: ConnectionInfo = "redis://localhost".parse().unwrap();
    cleanup(redis_connect.clone(), &list);
    let dir = tempdir().unwrap();
    let mut event_list = EventList::new(redis_connect.clone(), &list);
    spawn_watch(dir.path(), redis_connect.clone(), &list);

    sleep(Duration::from_millis(10));

    write(dir.path().join("foo.txt"), "foo").unwrap();
    remove_file(dir.path().join("foo.txt")).unwrap();

    sleep(Duration::from_millis(10));

    assert!(matches!(dbg!(event_list.next()), None));
}
