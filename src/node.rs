use crate::{config, init, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Cursor, Write};
use tracing::{error, info};

pub struct Node<S: store::Store> {
    #[allow(dead_code)]
    id: String, // include it as the src of any message it sends.
    #[allow(dead_code)]
    node_ids: Vec<String>,

    pub store: S,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Init(init::Payload),
    Other(HashMap<String, serde_json::Value>),
}

impl<S: store::Store> Node<S> {
    pub fn new(s: S) -> Self
    where
        S: store::Store,
    {
        Self {
            id: std::default::Default::default(),
            node_ids: std::default::Default::default(),
            store: s,
        }
    }

    pub fn init(&mut self, node_id: String, node_ids: Vec<String>)
    where
        S: store::Store,
    {
        self.id = node_id;
        self.node_ids = node_ids;
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
                info!("line: {:?}", l);

                // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
                // from_reader will read to end of deserialized object
                let msg: Message = serde_json::from_str(&l)?;
                info!(">> input: {:?}", msg);

                match msg {
                    Message::Init(init::Payload { src, dest, body }) => {
                        // If the message is an Init message, we need to actually configure
                        // the node object above.
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
