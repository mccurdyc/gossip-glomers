use crate::{config, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
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
    typ: String,
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
    Add(AddPayload),
    Read(ReadPayload),
    Other(HashMap<String, serde_json::Value>),
}

pub fn listen<R, W, T, S>(
    node: &mut node::Node<S>,
    reader: R,
    writer: &mut W,
    _cfg: &config::Config<T>,
) -> Result<()>
where
    R: BufRead,
    W: Write,
    T: config::TimeSource,
    S: store::Store,
{
    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Message = serde_json::from_reader(reader)?;
    info!(">> input: {:?}", msg);
    match msg {
        Message::Add(AddPayload { src, dest, body }) => {
            let mut buf = [0u8; 4];
            let _ = node.store.read(&mut buf)?;

            let o = u32::from_le_bytes(buf);
            let new = o + body.delta;
            node.store.write_all(&new.to_le_bytes())?;

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
            // TODO: sum needs to use the old value instead of always summing zero.
            let mut sum: u32 = 0;
            let mut buf = [0u8; 4];
            let _ = node.store.read(&mut buf)?;
            sum += u32::from_le_bytes(buf);

            let resp = ReadResp {
                src: dest,
                dest: src,
                body: ReadRespBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    value: sum,
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
