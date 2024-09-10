use app::config;
use app::config::SystemTime;
use app::echo;
use app::node;
use std::io;

fn main() {
    let node: node::Node = Default::default();

    node.run(
        echo::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &config::Config::<SystemTime>::new(&SystemTime {}),
    )
    .expect("failed to start");
}
