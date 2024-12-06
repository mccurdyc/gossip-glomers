use app::{config, counter, node, store};
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let p = Path::new("./counter.txt");
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
        io::stdin().lock(), // Each handle is a shared reference to a global buffer of input data to this process. A handle can be lock’d to gain full access to BufRead methods (e.g., .lines()).
        io::stdout().lock(),
        counter::listen,
        cfg,
    )
    .expect("failed to start");
}
