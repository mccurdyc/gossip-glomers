use crate::{config, init, node, store};
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

pub fn listen<'a: 'static, R, W, T, S>(
    node: &'a mut node::Node<S>,
    reader: R,
    writer: &mut W,
    _cfg: &'a mut config::Config<T>,
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
        Message::Add(AddPayload { src, dest, body }) => {
            let mut old: &mut [u8] = &mut [];
            node.store.read(&mut old)?;

            if let Ok(v) = old.try_into() {
                let o = u32::from_ne_bytes(v);
                let new = o + body.delta;
                let b: [u8; 4] = new.to_ne_bytes();
                let b_slice: &[u8] = &b;
                node.store.write(b_slice)?;
            }

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
            let mut stored_val: &mut [u8] = &mut [];
            node.store.read(&mut stored_val)?;

            let v = stored_val.try_into()?;
            let o = u32::from_ne_bytes(v);

            let resp = ReadResp {
                src: dest,
                dest: src,
                body: ReadRespBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    value: o,
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
