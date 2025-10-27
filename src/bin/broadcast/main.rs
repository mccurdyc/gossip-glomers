use app::{broadcast, config, node, store};
use std::io;
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use tracing::info;

// The worker_threads option configures the number of worker threads, and defaults
// to the number of cpus on the system.
#[tokio::main]
async fn main() {
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

    //  We use unbounded channels because we don't need guarantees.
    //  that's the purpose of the gossip protocol in the first place is to proceed w/o guarantees.
    //
    //  Although, I think I may want to use a bounded channel of capacity 1 to keep operations more
    //  synchronous. Especially due to the synchronous nature of IO.
    //
    //  Additionally, we use mspc instead of broadcast or watch channels b/c we are interacting
    //  with naturally synchronous resources (IO-bound and not CPU bound) and the thought was
    //  that we would end up fighting over IO resource locks anyway. Might be a future improvement
    //  to "fan out". We should look into the tokio::io and tokio::fs modules
    let (in_tx, mut in_rx) = mpsc::unbounded_channel::<Payload<RequestBody<T>>>();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Payload<ResponseBody<T>>>();

    let mut n: node::Node<store::FileStore, config::SystemTime> = node::Node::new(&mut s, cfg);

    let listen = tokio::spawn(async move {
        let read = read(io::stdin().lock(), in_tx).await;
        let listen = broadcast::listen(n, in_rx, out_tx).await;
        let write = write(io::stdout().lock(), out_rx).await;
        try_join!(read, listen, write);
    });

    let sync = tokio::spawn(async move {
        // TODO: we need to wrap the store in a mutex since it will be accessed by multiple threads.
        todo!("include sync() here");
    });

    try_join!(listen, sync);
}
