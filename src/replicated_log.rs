use crate::{config, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use tracing::info;

// Goals(s):
// https://github.com/jepsen-io/maelstrom/blob/main/doc/workloads.md#workload-kafka
// - store an append-only log in order
// - Each log is identified by a string key (e.g. "k1")
// - these logs contain a series of messages which are identified by an integer offset.
// - These offsets can be sparse in that not every offset must contain a message.
// - There are no recency requirements so acknowledged send messages do not need to return in poll messages immediately.
//   - No time requirements
//
// Maelstrom is checking for:
// - Lost writes: for example, a client sees offset 10 but not offset 5.
// - Monotonic increasing offsets: an offset for a log should always be increasing.

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
    data: T,
}

// Generic response body with common fields
#[derive(Serialize, Deserialize, Debug)]
struct ResponseBody<T> {
    #[serde(rename = "type")]
    typ: String,
    in_reply_to: u32,
    #[serde(flatten)]
    data: T,
}

// Send-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct TopologyData {
    topology: HashMap<String, Vec<String>>,
}

// Type aliases for cleaner usage
type TopologyPayload = Payload<RequestBody<TopologyData>>;

// Send-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct SendData {
    key: String,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SendResponseData {
    offset: HashMap<String, u32>,
}

// Type aliases for cleaner usage
type SendPayload = Payload<RequestBody<SendData>>;
type SendResp = Payload<ResponseBody<SendResponseData>>;

// Poll-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct PollData {
    offsets: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PollResponseData {
    msgs: HashMap<String, Queue>,
}

type PollPayload = Payload<RequestBody<PollData>>;
type PollResp = Payload<ResponseBody<PollResponseData>>;

#[derive(Serialize, Deserialize, Debug)]
struct Queue {
    offset: u32,
    len: u32,
}

// Commit-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct CommitData {
    offsets: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmptyData {}

type CommitPayload = Payload<RequestBody<CommitData>>;
type CommitResp = Payload<ResponseBody<EmptyData>>;

// ListCommitted-specific data structures
#[derive(Serialize, Deserialize, Debug)]
struct ListCommittedData {
    keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListCommittedResponseData {
    offsets: HashMap<String, u32>,
}

type ListCommittedPayload = Payload<RequestBody<ListCommittedData>>;
type ListCommittedOffsetsResp = Payload<ResponseBody<ListCommittedResponseData>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Topology(TopologyPayload),
    Send(SendPayload),
    Poll(PollPayload),
    Commit(CommitPayload),
    ListCommitted(ListCommittedPayload),
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
        Message::Send(SendPayload { src, dest, body }) => {
            let resp = SendResp {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "send_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: SendResponseData {
                        // TODO: appears this is supposed to be just an int with the offset
                        // doesn't need to be keyed.
                        //
                        // Poll is when we need to remember the message key.
                        offset: HashMap::new(),
                    },
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Poll(PollPayload { src, dest, body }) => {
            panic!("not implemented");

            let resp = PollResp {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "poll_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: PollResponseData {
                        msgs: HashMap::new(),
                    },
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::Commit(CommitPayload { src, dest, body }) => {
            panic!("not implemented");

            let resp = CommitResp {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "commit_offsets_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: EmptyData {},
                },
            };

            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
        Message::ListCommitted(ListCommittedPayload { src, dest, body }) => {
            panic!("not implemented");

            let resp = ListCommittedOffsetsResp {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "list_committed_offsets_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: ListCommittedResponseData {
                        offsets: HashMap::new(),
                    },
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
    }
    Ok(())
}
