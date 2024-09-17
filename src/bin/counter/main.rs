use app::config;
use app::config::SystemTime;
use app::counter;
use app::node;
use std::io;
use std::path::Path;

fn main() {
    let node: node::Node = Default::default();

    node.run(
        counter::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut config::Config::<SystemTime>::new(&SystemTime {}, Path::new("./data.txt"))
            .expect("failed to get config"),
    )
    .expect("failed to start");
}
