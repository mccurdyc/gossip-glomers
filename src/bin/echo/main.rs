use app::echo;
use app::run::run;
use std::io;

fn main() {
    run(echo::listen, io::stdin().lock(), &mut io::stdout().lock()).expect("failed to start");
}
