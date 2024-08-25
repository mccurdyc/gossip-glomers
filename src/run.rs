use anyhow::Result;
use std::io::{BufRead, Cursor, Read, Write};
use tracing::{error, info};

pub fn run<F, BR, W>(listen: F, reader: BR, writer: &mut W) -> Result<()>
where
    W: Write,
    F: Fn(Box<dyn Read>, &mut W) -> Result<()>,
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
            let _ = listen(buf, writer);
        } else {
            error!("error reading line: {:?}", line);
        }
    }

    Ok(())
}
