use app::config;
use app::config::SystemTime;
use app::run::run;
use app::unique;
use std::io;

fn main() {
    run(
        unique::listen,
        io::stdin().lock(),
        &mut io::stdout().lock(),
        &config::Config::<SystemTime>::new(&SystemTime {}),
    )
    .expect("failed to start");
}
