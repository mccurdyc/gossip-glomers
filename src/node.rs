use crate::config;
use anyhow::Result;
use serde_json;
use std::io::{BufRead, Cursor, Read, Write};
use tracing::{error, info};

#[derive(Default, Debug)]
pub struct Node {
    #[allow(dead_code)]
    id: String, // include it as the src of any message it sends.
    #[allow(dead_code)]
    node_ids: Vec<String>,
    // I'm not sure I love the idea of storing the entire JSON object
    store: Vec<serde_json::Value>,
}

impl Node {
    pub fn init(&mut self, node_id: String, node_ids: Vec<String>) -> Self {
        Self {
            id: node_id,
            node_ids,
            store: Vec::<serde_json::Value>::new(),
        }
    }

    pub fn run<F, BR, W, T>(
        mut self,
        listen: F,
        reader: BR,
        writer: &mut W,
        cfg: &config::Config<T>,
    ) -> Result<()>
    where
        W: Write,
        T: config::TimeSource,
        F: Fn(&mut Self, Box<dyn Read>, &mut W, &config::Config<T>) -> Result<()>,
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

    pub fn store(&mut self, v: serde_json::Value) -> Result<()> {
        self.store.push(v);
        Ok(())
    }

    pub fn retreive_seen_messages(&mut self) -> Result<Vec<serde_json::Value>> {
        Ok(self.store.clone())
    }
}
