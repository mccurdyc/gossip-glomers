use app::{config, node, replicated_log, store};
use std::io;
use std::path::Path;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let p = Path::new("./log.txt");
    let mut s = store::FileStore::new(p).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(&mut s);

    n.run(
        io::stdin().lock(), // Each handle is a shared reference to a global buffer of input data to this process. A handle can be lock’d to gain full access to BufRead methods (e.g., .lines()).
        io::stdout().lock(),
        replicated_log::listen,
        cfg,
    )
    .expect("failed to start");
}
