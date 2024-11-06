use app::config;
use app::config::SystemTime;
use app::counter;
use app::node;
use app::store;
use std::io;
use std::path::Path;

fn main() {
    let s = store::FileStore::new(&Path::new("./data.txt")).expect("failed to create store");
    let cfg = config::Config::<SystemTime>::new(&SystemTime {}).expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(s);

    n.run(
        io::stdin().lock(), // Each handle is a shared reference to a global buffer of input data to this process. A handle can be lock’d to gain full access to BufRead methods (e.g., .lines()).
        io::stdout().lock(),
        counter::listen,
        cfg,
    )
    .expect("failed to start");
}
