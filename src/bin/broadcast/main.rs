use app::{broadcast, config, node, store};
use std::io;
use tempfile::NamedTempFile;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // TODO: each process should get a unique "store file" instead of using a shared store.
    let f = NamedTempFile::new().expect("failed to create new named tempfile");
    let mut s = store::FileStore::new(f.path()).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(&mut s);

    n.run(
        io::stdin().lock(),
        io::stdout().lock(),
        broadcast::listen,
        cfg,
    )
    .expect("failed to start");
}
