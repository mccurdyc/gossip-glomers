use crate::payload::{Payload, RequestBody, ResponseBody};
use crate::{config, io, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
struct EchoData {
    echo: String,
}
type EchoRequest = Payload<RequestBody<EchoData>>;
type EchoResponse = Payload<ResponseBody<EchoData>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Echo(EchoRequest),
    Other(HashMap<String, serde_json::Value>),
}

pub fn listen<R, W, T, S>(
    _node: &mut node::Node<S>,
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
    match msg {
        Message::Echo(Payload { src, dest, body }) => io::to_writer(
            writer,
            &EchoResponse {
                src: dest,
                dest: src,
                body: ResponseBody {
                    typ: "echo_ok".to_string(),
                    in_reply_to: body.msg_id,
                    data: Some(EchoData {
                        echo: body.data.expect("failed to get echo body").echo,
                    }),
                },
            },
        )?,
        Message::Other(m) => {
            info!("other: {:?}", m);
        }
    };
    Ok(())
}
