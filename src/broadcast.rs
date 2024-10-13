use crate::{config, init, node};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use tracing::info;

// In this challenge, you’ll need to implement a broadcast system that gossips
// messages between all nodes in the cluster. Gossiping is a common way to propagate
// information across a cluster when you don’t need strong consistency guarantees.
//
// Data can be stored in-memory as node processes are not killed by Maelstrom.
// - It should store the "message" value locally so it can be read later. When a "read"
// request is sent.
//
// TODO:
// - "broadcast" type message
// - "read" type message for all broadcast values the node saw
// - "topology" type message - just ack b/c "All nodes can communicate with each other regardless
// of the topology passed in."
// - test
// ```bash
// ./maelstrom test -w broadcast --bin ~/go/bin/maelstrom-broadcast --node-count 1 --time-limit 20 --rate 10
// ```

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
    Init(init::Payload),
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
    _cfg: &mut config::Config<T, S>,
) -> Result<()>
where
    R: Read,
    W: Write,
    T: config::TimeSource,
    S: Read + Write,
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
        Message::Broadcast(BroadcastPayload { src, dest, body }) => {
            node.store(serde_json::to_value(&body)?)?;
            info!("node: {:?}", node);

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
            info!("node in read: {:?}", node);
            let messages: Vec<u32> = node
                .retreive_seen_messages()?
                .iter()
                .map(move |m| {
                    let b: BroadcastReqBody =
                        serde_json::from_value(m.clone()).expect("failed to parse stored message");
                    b.message
                })
                .collect();
            info!("messages: {:?}", messages);
            let resp = ReadResp {
                src: dest,
                dest: src,
                body: ReadRespBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    messages,
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
