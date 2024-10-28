use crate::{config, store};
use anyhow::Result;
use std::io::{BufRead, Cursor, Read, Write};
use tracing::{error, info};

pub struct Node<S: store::Store + 'static> {
    #[allow(dead_code)]
    id: String, // include it as the src of any message it sends.
    #[allow(dead_code)]
    node_ids: Vec<String>,

    pub store: &'static mut S,
}

impl<S: store::Store + 'static> Node<S> {
    pub fn new(s: &'static mut S) -> Self
    where
        S: store::Store,
    {
        Self {
            id: std::default::Default::default(),
            node_ids: std::default::Default::default(),
            store: s,
        }
    }

    pub fn init(&'static mut self, node_id: String, node_ids: Vec<String>)
    where
        S: store::Store,
    {
        self.id = node_id;
        self.node_ids = node_ids;
    }

    pub fn run<F, BR, W, T>(
        &'static mut self, // take ownership
        listen: F,
        reader: BR,
        writer: &mut W,
        cfg: &'static mut config::Config<T>,
    ) -> Result<()>
    where
        W: Write,
        T: config::TimeSource,
        F: Fn(&mut Self, Box<dyn Read>, &mut W, &mut config::Config<T>) -> Result<()>,
        S: store::Store,
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
                let _ = match listen(self, buf, writer, cfg) {
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
}
