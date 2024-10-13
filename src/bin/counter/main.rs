use app::config;
use app::config::SystemTime;
use app::counter;
use app::node;
use app::store;
use std::io;
use std::path::Path;

fn main() {
    let node: node::Node = Default::default();

    // store should be created here and injected
    let mut s = store::FileStore
        .new(Path("./data.txt"))
        .expect("failed to create store");

    node.run(
        counter::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut config::Config::<SystemTime>::new(&SystemTime {}, s).expect("failed to get config"),
    )
    .expect("failed to start");
}
