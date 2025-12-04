use anyhow::Context;
use app::broadcast::{Body, RequestBody};
use app::payload::Payload;
use app::{broadcast, config, node, store};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use tokio::try_join;
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
    let (in_tx, in_rx) = mpsc::unbounded_channel::<Payload<RequestBody>>();
    let (out_tx, out_rx) = mpsc::unbounded_channel::<Payload<Body>>();

    // TODO: Need to put a mutex in FileStore to protect the file.
    let s = store::FileStore::new(f.path().to_path_buf()).expect("failed to create store");
    let mut n: node::Node<store::FileStore, config::SystemTime> = node::Node::new(s, cfg);

    // Thread that reads, processes and writes messages.
    let listen = tokio::spawn(async move {
        let reader = tokio::io::BufReader::new(tokio::io::stdin());
        let _ = node::read(reader, in_tx)
            .await
            .context("failed while reading");
        // needs to take a reference to node because sync will also need to borrow node
        // this means that we need to enable cloning the store
        // but File does not implement Clone so maybe we should actually keep store as a reference
        broadcast::listen(&mut n, in_rx, out_tx).await;
        let _ = node::write(tokio::io::stdout(), out_rx)
            .await
            .context("failed while writing"); // reads responses and writes out
    });

    // Thread for background syncing
    let sync = tokio::spawn(async move {
        // TODO: we need to wrap the store in a mutex since it will be accessed by multiple threads.
        todo!("include sync() here");
    });

    let (_, _) = try_join!(listen, sync)?;
    Ok(())
}
