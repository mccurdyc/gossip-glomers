use app::{config, counter, node, store};
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

fn main() {
    let p = Path::new("./counter.txt");
    let f = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(p)
        .expect("failed to create store file");
    let s = store::FileStore::new(&f).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(s);

    n.run(
        io::stdin().lock(), // Each handle is a shared reference to a global buffer of input data to this process. A handle can be lock’d to gain full access to BufRead methods (e.g., .lines()).
        io::stdout().lock(),
        counter::listen,
        cfg,
    )
    .expect("failed to start");
}
