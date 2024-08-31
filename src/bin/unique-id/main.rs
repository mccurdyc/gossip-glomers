use app::run::run;
use app::unique;
use std::io;

fn main() {
    run(unique::listen, io::stdin().lock(), &mut io::stdout().lock()).expect("failed to start");
}
