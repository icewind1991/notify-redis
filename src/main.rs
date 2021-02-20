use color_eyre::Result;
use notify_redis::watch;
use std::env;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if let [_, path, redis, list] = args.as_slice() {
        watch(path, redis.as_str(), list, Duration::from_secs(2))?;
    } else {
        println!("usage: {} <path> <redis_connect> <redis_list>", args[0])
    }
    Ok(())
}
