use crate::{config, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use tracing::info;

// Goals(s):
// https://github.com/jepsen-io/maelstrom/blob/main/doc/workloads.md#workload-kafka

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct SendPayload {
    src: String,
    dest: String,
    body: SendReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct SendReqBody {
    #[serde(rename = "type")]
    typ: String, //
    msg_id: u32,
    key: String,
    msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SendResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: SendRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct SendRespBody {
    #[serde(rename = "type")]
    typ: String, // send_ok
    in_reply_to: u32,
    offset: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct PollPayload {
    src: String,
    dest: String,
    body: PollReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct PollReqBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
    offsets: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct PollResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: PollRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct PollRespBody {
    #[serde(rename = "type")]
    typ: String, // poll_ok
    in_reply_to: u32,
    msgs: HashMap<String, Queue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Queue {
    offset: u32,
    len: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct CommitPayload {
    src: String,
    dest: String,
    body: CommitReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommitReqBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
    offsets: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommitResp {
    src: String,
    dest: String,
    body: CommitRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct CommitRespBody {
    #[serde(rename = "type")]
    typ: String, // commit_offsets_ok
    in_reply_to: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct ListCommittedPayload {
    src: String,
    dest: String,
    body: ListCommittedReqBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListCommittedReqBody {
    #[serde(rename = "type")]
    typ: String,
    msg_id: u32,
    keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListCommittedOffsetsResp {
    src: String,
    dest: String,
    // You can't nest structures in Rust for ownership reasons.
    body: ListCommittedOffsetsRespBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ListCommittedOffsetsRespBody {
    #[serde(rename = "type")]
    typ: String, // list_committed_offsets_ok
    in_reply_to: u32,
    offsets: HashMap<String, u32>,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
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
            panic!("not implemented");

            let resp = SendResp {
                src: dest,
                dest: src,
                body: SendRespBody {
                    typ: "send_ok".to_string(),
                    in_reply_to: body.msg_id,
                    offset: HashMap::new(),
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
                body: PollRespBody {
                    typ: "poll_ok".to_string(),
                    in_reply_to: body.msg_id,
                    msgs: HashMap::new(),
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
                body: CommitRespBody {
                    typ: "commit_offsets_ok".to_string(),
                    in_reply_to: body.msg_id,
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
                body: ListCommittedOffsetsRespBody {
                    typ: "list_committed_offsets_ok".to_string(),
                    in_reply_to: body.msg_id,
                    offsets: HashMap::new(),
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
