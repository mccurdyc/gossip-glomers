use app::{config, node, store, unique};
use std::io;

fn main() {
    let buf: Vec<u8> = Vec::new();
    let s = store::MemoryStore::new(buf).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

    n.run(io::stdin().lock(), io::stdout().lock(), unique::listen, cfg)
        .expect("failed to start");
}
