use crate::{init, node};
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
    msg_id: u8,
    echo: String,
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
    msg_id: u8,
    in_reply_to: u8,
    echo: String,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Init(init::Payload),
    Echo(Payload),
}

pub fn listen<R, W>(reader: R, writer: &mut W) -> Result<()>
where
    R: Read,
    W: Write,
{
    // CRITICAL: I'm pretty sure this will lose all state the way it
    // currently exists inside of the loop.
    let node: &mut node::Node = &mut Default::default();

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
        Message::Echo(Payload { src, dest, body }) => {
            let resp = Resp {
                src: dest,
                dest: src,
                body: RespBody {
                    typ: "echo_ok".to_string(),
                    msg_id: body.msg_id,
                    in_reply_to: body.msg_id,
                    echo: body.echo,
                },
            };
            let mut resp_str = serde_json::to_string(&resp)?;
            resp_str.push('\n');
            info!("<< output: {:?}", &resp_str);
            writer.write_all(resp_str.as_bytes())?;
        }
    };
    Ok(())
}
