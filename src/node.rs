use crate::{config, store};
use anyhow::Result;
use std::io::{BufRead, Cursor, Read, Write};
use tracing::{error, info};

pub struct Node<S: store::Store> {
    #[allow(dead_code)]
    id: String, // include it as the src of any message it sends.
    #[allow(dead_code)]
    node_ids: Vec<String>,

    pub store: S,
}

impl<S: store::Store> Node<S> {
    pub fn init(&mut self, node_id: String, node_ids: Vec<String>, s: S) -> Self
    where
        S: store::Store + 'static,
    {
        Self {
            id: node_id,
            node_ids,
            store: s,
        }
    }

    pub fn run<F, BR, W, T>(
        mut self,
        listen: F,
        reader: BR,
        writer: &mut W,
        cfg: &mut config::Config<T, S>,
    ) -> Result<()>
    where
        W: Write,
        T: config::TimeSource,
        F: Fn(&mut Self, Box<dyn Read>, &mut W, &mut config::Config<T, S>) -> Result<()>,
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
                let _ = match listen(&mut self, buf, writer, cfg) {
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
