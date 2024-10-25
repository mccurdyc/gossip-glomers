use app::config;
use app::config::SystemTime;
use app::counter;
use app::node;
use app::store;
use std::io;
use std::path::Path;

fn main() {
    let mut s = store::FileStore::new(&Path::new("./data.txt")).expect("failed to create store");
    let node = node::Node::new(&mut s);

    node.run(
        counter::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut config::Config::<SystemTime>::new(&SystemTime {}).expect("failed to get config"),
    )
    .expect("failed to start");
}
