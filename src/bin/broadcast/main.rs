use app::broadcast;
use app::config;
use app::config::SystemTime;
use app::run::run;
use std::io;

fn main() {
    run(
        broadcast::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &config::Config::<SystemTime>::new(&SystemTime {}),
    )
    .expect("failed to start");
}
