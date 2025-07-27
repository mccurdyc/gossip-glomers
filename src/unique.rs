use crate::payload::{Payload, RequestBody, ResponseBody, UnhandledMessage};
use crate::{config, io, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{BufRead, Write};
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    #[serde(rename = "id")]
    unique_id: String,
}

type UniqueRequest = Payload<RequestBody<Data>>;
type UniqueResponse = Payload<ResponseBody<Data>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    Unique(UniqueRequest),
    Other(UnhandledMessage),
}

pub fn listen<R, W, T, S>(
    _node: &mut node::Node<S>,
    reader: R,
    writer: &mut W,
    cfg: &config::Config<T>,
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
        Message::Unique(Payload { src, dest, body }) => {
            let hash = Sha256::digest(
                format!("{}-{}-{:?}", dest, body.msg_id, cfg.time_source.now()).into_bytes(),
            );
            io::to_writer(
                writer,
                &UniqueResponse {
                    src: dest,
                    dest: src,
                    body: ResponseBody {
                        typ: "generate_ok".to_string(),
                        in_reply_to: body.msg_id,
                        data: Some(Data {
                            unique_id: hex::encode(hash),
                        }),
                    },
                },
            )?
        }
        Message::Other(m) => {
            info!("other: {:?}", m);
        }
    };
    Ok(())
}
