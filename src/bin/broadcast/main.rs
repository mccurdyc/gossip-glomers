use app::{broadcast, config, node, store};
use std::io;
use std::path::Path;

fn main() {
    let s = store::FileStore::new(Path::new("./store.txt")).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(s);

    n.run(
        io::stdin().lock(),
        io::stdout().lock(),
        broadcast::listen,
        cfg,
    )
    .expect("failed to start");
}
