use app::{config, echo, node, store};
use std::io;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let buf: Vec<u8> = Vec::new();
    let s = store::MemoryStore::new(buf).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

    n.run(io::stdin().lock(), io::stdout().lock(), echo::listen, cfg)
        .expect("failed to start");
}
