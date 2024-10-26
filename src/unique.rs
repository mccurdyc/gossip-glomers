use crate::{config, init, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct Payload {
    src: String,
    dest: String,
    body: ReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReqBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Resp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: RespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct RespBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
    in_reply_to: u32,
    #[serde(rename = "id")]
    unique_id: String,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Init(init::Payload),
    Unique(Payload),
    Other(HashMap<String, serde_json::Value>), // Why is it getting deserialized as Other? It fails
                                               // to deserialize as Other(Payload).
}

pub fn listen<'a, R, W, T, S>(
    node: &'a mut node::Node<S>,
    reader: R,
    writer: &mut W,
    cfg: &'a mut config::Config<T>,
) -> Result<()>
where
    R: Read,
    W: Write,
    T: config::TimeSource,
    S: store::Store,
{
    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Message = serde_json::from_reader(reader)?;
    info!(">> input: {:?}", msg);
    match msg {
        // Node didn't respond to init message
        Message::Init(init::Payload { src, dest, body }) => {
            // If the message is an Init message, we need to actually configure
            // the node object above.
            node.init(body.node_id, body.node_ids);
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
        Message::Unique(Payload { src, dest, body }) => {
            let hash = Sha256::digest(
                format!("{}-{}-{:?}", dest, body.msg_id, cfg.time_source.now()).into_bytes(),
            );
            let resp = Resp {
                src: dest,
                dest: src,
                body: RespBody {
                    typ: "generate_ok".to_string(),
                    msg_id: body.msg_id,
                    in_reply_to: body.msg_id,
                    unique_id: hex::encode(hash),
                },
            };
            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Other(m) => {
            info!("other: {:?}", m);
        }
    };
    Ok(())
}
