use app::{broadcast, config, node, store};
use std::io;
use tempfile::NamedTempFile;
use tracing::info;

// The worker_threads option configures the number of worker threads, and defaults
// to the number of cpus on the system.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let f = NamedTempFile::new().expect("failed to create new named tempfile");
    info!("created tempfile store {:?}", f.path());

    // TODO: Need to put a mutex in FileStore to protect the file.
    let mut s = store::FileStore::new(f.path()).expect("failed to create store");

    // TODO: get rid of config
    let cfg = config::Config::<config::SystemTime>::new(config::SystemTime {})
        .expect("failed to get config");

    // main()
    //  - inTx, inRx = mpsc::chan
    //  - outTx, outRx = mpsc::chan
    //
    // async run(stdin, stdout, inTx, outRx)
    //  - reads from stdin
    //  - writes to inTx (consumed by listen)
    //  - reads from outRx
    //  - writes to stdout
    //
    // async node.listen(inRx, outTx, store)
    //  - reads from inRx
    //  - decides who and whether to broadcast
    //  - takes lock of store
    //  - writes to store
    //  - writes to outTx
    //
    // async node.sync(cloned outTx, store)
    //  - takes lock of store
    //  - reads deltas since last read of store
    //  - maintains some in-memory state of the world
    //  - if memory is empty, send full state of the world (handle process restarts)
    //  - decides who to broadcast to
    //  - writes to outTx

    let mut n: node::Node<store::FileStore, config::SystemTime> = node::Node::new(&mut s, cfg);

    n.run(io::stdin().lock(), io::stdout().lock(), broadcast::listen)
        .expect("failed to start");

    Ok(())
}
