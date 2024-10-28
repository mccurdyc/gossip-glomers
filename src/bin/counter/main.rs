use app::config;
use app::config::SystemTime;
use app::counter;
use app::node;
use app::store;
use std::io;
use std::path::Path;

fn main() {
    let mut s: &'static store::FileStore =
        &store::FileStore::new(&Path::new("./data.txt")).expect("failed to create store");

    let mut cfg: &'static config::Config<SystemTime> =
        &config::Config::<SystemTime>::new(&SystemTime {}).expect("failed to get config");

    let mut node: node::Node<store::FileStore> = node::Node::new(&mut s);

    node.run(
        counter::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut cfg,
    )
    .expect("failed to start");
}
