use app::{broadcast, config, node, store};
use std::io;
use std::path::Path;

fn main() {
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Which store does it use during a maelstrom run? I can't find it.
    // Maybe /tmp/store.txt. It's definitely not the one in this local dir.
    //
    // Yeah, it's definitely the /tmp/store.txt. The messages stored agree with what maelstrom
    // experiences. Messages are definitely persisted.
    //
    // It "feels" like when maelstrom "restarts" the broadcast process, it's overwriting instead of
    // appending to the file. Or writes are only happening occassionally and nodes are not
    // retrieving prior stats from the last process.
    //
    // An alternative could be that locks are too slow and broadcast isn't waiting for the file
    // lock and instead is dropping messages on the ground.
    let p = Path::new("./store.txt");
    let mut s = store::FileStore::new(p).expect("failed to create store");
    let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
        .expect("failed to get config");
    let mut n: node::Node<store::FileStore> = node::Node::new(&mut s);

    n.run(
        io::stdin().lock(),
        io::stdout().lock(),
        broadcast::listen,
        cfg,
    )
    .expect("failed to start");
}
