use crate::{config, io, node, payload, store};
use anyhow::Result;
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
    pub world: HashMap<String, Metadata>,
    pub neighborhood: HashMap<String, Metadata>,
    pub store: &'a mut S,
}

#[derive(Debug, Serialize, Deserialize)]
struct InitData {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Init(payload::Payload<payload::RequestBody<InitData>>),
    Other(payload::UnhandledMessage),
}

impl<'a, S: store::Store> Node<'a, S> {
    pub fn new(s: &'a mut S) -> Self
    where
        S: store::Store,
    {
        Self {
            id: std::default::Default::default(),
            world: std::default::Default::default(),
            neighborhood: std::default::Default::default(),
            store: s,
        }
    }

    pub fn init(&mut self, node_id: String, node_ids: Vec<String>)
    where
        S: store::Store,
    {
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

                // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
                // from_reader will read to end of deserialized object
                let msg: Message = serde_json::from_str(&l)?;

                match msg {
                    Message::Init(payload::Payload { src, dest, body }) => {
                        let b = body.data.expect("expected init request body");

                        self.init(b.node_id, b.node_ids);

                        io::to_writer(
                            &mut writer,
                            payload::Payload {
                                src: dest,
                                dest: src,
                                body: payload::ResponseBody::<()> {
                                    typ: "init_ok".to_string(),
                                    in_reply_to: body.msg_id,
                                    data: None,
                                },
                            },
                        )?
                    }
                    Message::Other(_) => {
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
