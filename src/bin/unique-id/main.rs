use app::config;
use app::config::SystemTime;
use app::node;
use app::unique;
use std::io;
use std::path::Path;

fn main() {
    let mut s = store::MemoeryStore::new().expect("failed to create store");
    let node = node::Node::new(&mut s);

    node.run(
        unique::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut config::Config::<SystemTime>::new(&SystemTime {}).expect("failed to create config"),
    )
    .expect("failed to start");
}
