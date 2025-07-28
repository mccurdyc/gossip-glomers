use crate::{config, init, node, store};
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
#[serde(untagged)]
enum Message {
    Init(init::Payload),
    Other(HashMap<String, serde_json::Value>),
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
                    Message::Init(init::Payload { src, dest, body }) => {
                        self.init(body.node_id, body.node_ids);
                        let resp = init::Resp {
                            src: dest,
                            dest: src,
                            body: init::RespBody {
                                typ: "init_ok".to_string(),
                                in_reply_to: body.msg_id,
                            },
                        };
                        let mut resp_str = serde_json::to_string(&resp)?;
                        resp_str.push('\n');
                        info!("<< output: {:?}", &resp_str);
                        writer.write_all(resp_str.as_bytes())?;
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
