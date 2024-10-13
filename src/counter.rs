use crate::{config, init, node};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::info;

// Goals(s):
// - Increment a single global counter
// - Only need eventual consistency (seconds are fine)
//
// Workload
// - Adds a non-negative integer, called delta, to the counter.

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct AddPayload {
    src: String,
    dest: String,
    body: AddReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddReqBody {
    #[serde(rename = "type")]
    typ: String, //
    msg_id: u32,
    delta: u32, // will be unique
}

#[derive(Serialize, Deserialize, Debug)]
struct AddResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: AddRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddRespBody {
    #[serde(rename = "type")]
    typ: String, // add_ok
    in_reply_to: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct ReadPayload {
    src: String,
    dest: String,
    body: ReadReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadReqBody {
    #[serde(rename = "type")]
    typ: String, //
    msg_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: ReadRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadRespBody {
    #[serde(rename = "type")]
    typ: String, // read_ok
    in_reply_to: u32,
    value: u32,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Init(init::Payload),
    Add(AddPayload),
    Read(ReadPayload),
    Other(HashMap<String, serde_json::Value>),
}

pub fn listen<R, W, T>(
    node: &mut node::Node,
    reader: R,
    writer: &mut W,
    cfg: &mut config::Config<T>,
) -> Result<()>
where
    R: Read,
    W: Write,
    T: config::TimeSource,
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
            node.init(body.node_id, body.node_ids, cfg.store);
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
        Message::Add(AddPayload { src, dest, body }) => {
            let &mut old: Vec<u8> = Vec::new();
            node.store.read(old)?;

            let old_u32 = old.from_be_bytes();
            let new = old_u32 + body.delta;

            cfg.store.write(new.to_be_bytes())?;

            let resp = AddResp {
                src: dest,
                dest: src,
                body: AddRespBody {
                    typ: "add_ok".to_string(),
                    in_reply_to: body.msg_id,
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Read(ReadPayload { src, dest, body }) => {
            let v = cfg.store.get()?;

            let resp = ReadResp {
                src: dest,
                dest: src,
                body: ReadRespBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    value: v,
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
