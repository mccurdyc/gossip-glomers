use crate::config;
use anyhow::Result;
use std::io::{BufRead, Cursor, Read, Write};
use tracing::{error, info};

pub fn run<F, BR, W, T>(
    listen: F,
    reader: BR,
    writer: &mut W,
    cfg: &config::Config<T>,
) -> Result<()>
where
    W: Write,
    T: config::TimeSource,
    F: Fn(Box<dyn Read>, &mut W, &config::Config<T>) -> Result<()>,
    BR: BufRead,
{
    // Initialize the default subscriber, which logs to stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr) // all debug logs have to go to stderr
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("starting listener...");

    for line in reader.lines() {
        if let Ok(l) = line {
            info!("line: {:?}", l);
            let buf: Box<dyn Read> = Box::new(Cursor::new(l));
            let _ = match listen(buf, writer, cfg) {
                Ok(_) => {}
                Err(e) => {
                    error!("error listening: {:?}", e);
                }
            };
        } else {
            error!("error reading line: {:?}", line);
        }
    }

    Ok(())
}
