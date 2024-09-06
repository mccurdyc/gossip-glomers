use app::broadcast;
use app::config;
use app::config::SystemTime;
use app::node;
use std::io;

fn main() {
    let node: node::Node = Default::default();

    node.run(
        broadcast::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &config::Config::<SystemTime>::new(&SystemTime {}),
    )
    .expect("failed to start");
}
