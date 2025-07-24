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
//
// - Your node should propagate values it sees from broadcast messages to the other nodes in the cluster
// - It can use the topology passed to your node in the topology message or you can build your own topology.
//
// The 63% Rule
// In epidemic spreading models, once approximately 63% of susceptible nodes have been "infected" (received the message), there's a very high probability the epidemic will reach essentially 100% coverage.
// 50-70% initial coverage for reliable full propagation
// Fanout of 3-5 nodes per gossip round
// Log(N) rounds to reach full coverage (where N = total nodes)

// Generic payload wrapper for all message types
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct Payload<T> {
    src: String,
    dest: String,
    body: T,
}

// Generic request body with common fields
#[derive(Serialize, Deserialize, Debug)]
struct RequestBody<T> {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
    #[serde(flatten)]
    data: Option<T>,
}

// Generic response body with common fields
#[derive(Serialize, Deserialize, Debug)]
struct ResponseBody<T> {
    #[serde(rename = "type")]
    typ: String,
    in_reply_to: u32,
    #[serde(flatten)]
    data: Option<T>,
}

// Send-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct TopologyData {
    topology: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BroadcastReqData {
    message: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadReqData {
    msg_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadRespData {
    messages: Vec<u32>,
}

// Type aliases for cleaner usage
type TopologyPayload = Payload<RequestBody<TopologyData>>;
type BroadcastReqPayload = Payload<RequestBody<BroadcastReqData>>;
type ReadReqPayload = Payload<RequestBody<ReadReqData>>;
type ReadRespPayload = Payload<ResponseBody<ReadRespData>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Topology(TopologyPayload),
    Broadcast(BroadcastReqPayload),
    Read(ReadReqPayload),
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
        Message::Topology(TopologyPayload { src, dest, body }) => {
            // TODO: right now we do nothing here
            let resp = Payload {
                src: dest,
                dest: src,
                body: ResponseBody::<()> {
                    typ: "topology_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: None,
                },
            };
            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Broadcast(BroadcastReqPayload { src, dest, body }) => {
            serde_json::ser::to_writer(
                &mut node.store,
                &body.data.ok_or("failed").unwrap().message,
            )?;
            node.store.write_all(b"\n")?;

            let resp = Payload {
                src: dest,
                dest: src,
                body: ResponseBody::<()> {
                    typ: "broadcast_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: None,
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Read(ReadReqPayload { src, dest, body }) => {
            // Make sure we reset the file offset
            // TODO: this makes no sense for stores that are NOT file-based (maybe)
            node.store.rewind()?;

            let mut seen = Vec::<u32>::new();
            let lines = node.store.lines();
            for line in lines {
                let v: u32 = line.expect("failed to extract line").parse()?;
                seen.push(v);
            }

            let resp = Payload {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "read_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: Some(ReadRespData { messages: seen }),
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
