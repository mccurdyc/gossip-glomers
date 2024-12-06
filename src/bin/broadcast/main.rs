use app::{broadcast, config, node, store};
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let p = Path::new("./store.txt");
    let f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(p)
        .expect("failed to create store file");
    let mut s = store::FileStore::new(&f).expect("failed to create store");
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
