use crate::{config, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use tracing::info;

// In this challenge, you’ll need to implement a broadcast system that gossips
// messages between all nodes in the cluster. Gossiping is a common way to propagate
// information across a cluster when you don’t need strong consistency guarantees.
//
// Data can be stored in-memory as node processes are not killed by Maelstrom.
// - It should store the "message" value locally so it can be read later. When a "read"
// request is sent.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct BroadcastPayload {
    src: String,
    dest: String,
    body: BroadcastReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastReqBody {
    #[serde(rename = "type")]
    typ: String, // broadcast
    msg_id: u32,
    message: u32, // will be unique
}

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: BroadcastRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastRespBody {
    #[serde(rename = "type")]
    typ: String, // broadcast_ok
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
    typ: String, // read
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
    messages: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct TopologyPayload {
    src: String,
    dest: String,
    body: TopologyReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopologyReqBody {
    #[serde(rename = "type")]
    typ: String, // topology
    msg_id: u32,
    topology: Topology,
}

type Topology = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Debug)]
struct TopologyResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: TopologyRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct TopologyRespBody {
    #[serde(rename = "type")]
    typ: String, // topology_ok
    in_reply_to: u32,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Broadcast(BroadcastPayload),
    Topology(TopologyPayload),
    // CRITICAL: read has to come AFTER topology because the message is less specific
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
    S: store::Store + std::fmt::Debug,
{
    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Message = serde_json::from_reader(reader)?;
    match msg {
        Message::Broadcast(BroadcastPayload { src, dest, body }) => {
            serde_json::ser::to_writer(&mut node.store, &body.message)?;
            node.store.write_all(b"\n")?;

            let resp = BroadcastResp {
                src: dest,
                dest: src,
                body: BroadcastRespBody {
                    typ: "broadcast_ok".to_string(),
                    in_reply_to: body.msg_id,
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Read(ReadPayload { src, dest, body }) => {
            // Make sure we reset the file offset
            // TODO: this makes no sense for stores that are NOT file-based (maybe)
            node.store.rewind()?;

            let mut seen = Vec::<u32>::new();
            let lines = node.store.lines();
            for line in lines {
                let v: u32 = line.expect("failed to extract line").parse()?;
                seen.push(v);
            }

            let resp = ReadResp {
                src: dest,
                dest: src,
                body: ReadRespBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    messages: seen,
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Topology(TopologyPayload { src, dest, body }) => {
            // TODO: right now we do nothing here
            let resp = TopologyResp {
                src: dest,
                dest: src,
                body: TopologyRespBody {
                    typ: "topology_ok".to_string(),
                    in_reply_to: body.msg_id,
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
