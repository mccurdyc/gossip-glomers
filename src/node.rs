use crate::{config, io, node, payload, store};
use anyhow::Result;
use payload::Payload;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Cursor, Write};
use tracing::{error, info};

#[derive(Debug)]
pub struct Metadata {
    pub priority: u8,
}

#[derive(Debug)]
pub struct Node<'a, S: store::Store> {
    pub id: String, // include it as the src of any message it sends.
    pub msg_id: u32,
    pub world: HashMap<String, Metadata>,
    pub neighborhood: HashMap<String, Metadata>,
    pub store: &'a mut S,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
enum RequestBody {
    Init {
        msg_id: u32,
        node_id: String,
        node_ids: Vec<String>,
    },
    #[serde(other)]
    Other,
}

impl<'a, S: store::Store> Node<'a, S> {
    pub fn new(s: &'a mut S) -> Self
    where
        S: store::Store,
    {
        Self {
            id: std::default::Default::default(),
            msg_id: std::default::Default::default(),
            world: std::default::Default::default(),
            neighborhood: std::default::Default::default(),
            store: s,
        }
    }

    pub fn init(&mut self, node_id: String, node_ids: Vec<String>) {
        self.msg_id = 1;
        self.id = node_id;
        self.neighborhood = HashMap::new();
        self.world = HashMap::new();

        for n in node_ids {
            if n == self.id {
                continue;
            }

            // Let's pretend (for now, just use a random number generator)
            // the init message included weights
            //
            // NOTE: We don't filter the neighborhood here because filtering where
            // messages go will be up to the send/response implementation.
            // In other words, there may be cases where we want to send a message
            // to everyone and other places where we want to be selective.
            let priority: u8 = rand::random_range(0..=100);
            self.world.insert(n.clone(), Metadata { priority });

            if priority > 33 {
                self.neighborhood.insert(n.clone(), Metadata { priority });
            }
        }
    }

    pub fn run<R, W, F, T>(
        &mut self,
        reader: R,
        mut writer: W,
        f: F,
        cfg: config::Config<T>,
    ) -> Result<()>
    where
        R: BufRead,
        W: Write,
        F: Fn(&mut node::Node<S>, Box<dyn BufRead>, &mut W, &config::Config<T>) -> Result<()>,
        T: config::TimeSource,
        S: store::Store,
    {
        info!("starting listener...");

        for line in reader.lines() {
            if let Ok(l) = line {
                info!(">> input: {:?}", l);

                // This tries to deserialize into Payload<RequestBody> based on the "type" field
                // defined in RequestBody.
                let msg: Payload<RequestBody> = serde_json::from_str(&l)?;

                // Match based on the type.
                match msg.body {
                    RequestBody::Init {
                        msg_id,
                        node_id,
                        node_ids,
                    } => {
                        self.init(node_id, node_ids);

                        io::to_writer(
                            &mut writer,
                            payload::Payload {
                                src: msg.dest,
                                dest: msg.src,
                                body: payload::ResponseBody::<()> {
                                    typ: "init_ok".to_string(),
                                    in_reply_to: msg_id,
                                    data: None,
                                },
                            },
                        )?
                    }

                    RequestBody::Other => {
                        let buf = Box::new(Cursor::new(l));
                        match f(self, buf, &mut writer, &cfg) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error listening: {:?}", e);
                            }
                        };
                    }
                }
            } else {
                error!("error reading line: {:?}", line);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{config, echo, node, store};
    use once_cell::sync::Lazy;
    use std::io::Cursor;
    use std::vec::Vec;

    // Ensure that the `tracing` stack is only initialised once using `once_cell`
    pub static TRACING: Lazy<()> = Lazy::new(|| {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr) // all debug logs have to go to stderr
            .with_max_level(tracing::Level::DEBUG)
            .init();
    });

    #[test]
    fn run() {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        Lazy::force(&TRACING);

        let test_cases = vec![
            (
                r#"{"id":42,"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"32","node_ids":["n1","n2","n3"]}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"echo","msg_id":99,"echo":"Please echo 99"}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"echo_ok","in_reply_to":99,"echo":"Please echo 99"}}
"#,
            ),
        ];

        for (input, expected) in test_cases {
            let buf: Vec<u8> = Vec::new();
            let mut s = store::MemoryStore::new(buf).expect("failed to create store");
            let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                .expect("failed to get config");
            let mut n: node::Node<store::MemoryStore> = node::Node::new(&mut s);

            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            n.run(read_cursor, write_cursor, echo::listen, cfg)
                .expect("Node did NOT run");

            assert_eq!(String::from_utf8(vec).unwrap().trim(), expected.trim());
        }
    }
}
