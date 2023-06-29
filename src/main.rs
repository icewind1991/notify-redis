use clap::Parser;
use color_eyre::Result;
use notify_redis::watch;
use std::path::PathBuf;
use std::time::Duration;

/// Push filesystem notifications into a redis list
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Folder to watch
    path: PathBuf,
    /// Redis connection string
    redis_connect: String,
    /// Redis list to push changes to
    redis_list: String,
}

fn main() -> Result<()> {
    let args: Args = Args::parse();
    watch(
        args.path,
        args.redis_connect,
        &args.redis_list,
        Duration::from_secs(2),
    )?;
    Ok(())
}
