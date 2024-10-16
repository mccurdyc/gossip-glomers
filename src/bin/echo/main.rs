use app::config;
use app::config::SystemTime;
use app::echo;
use app::node;
use std::io;
use std::path::Path;

fn main() {
    let node: node::Node = Default::default();

    node.run(
        echo::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &mut config::Config::<SystemTime>::new(&SystemTime {}, &Path::new("empty"))
            .expect("failed to create config"),
    )
    .expect("failed to start");
}
