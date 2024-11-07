use app::{config, echo, node, store};
use std::io;

fn main() {
    let s = store::MemoryStore::new().expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

    n.run(io::stdin().lock(), io::stdout().lock(), echo::listen, cfg)
        .expect("failed to start");
}
