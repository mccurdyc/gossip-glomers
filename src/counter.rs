use crate::payload::{Payload, RequestBody, ResponseBody, UnhandledMessage};
use crate::{config, io, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, SeekFrom, Write};
use tracing::info;

// Goals(s):
// - Increment a single global counter
// - Only need eventual consistency (seconds are fine)
//
// Workload
// - Adds a non-negative integer, called delta, to the counter.

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "lowercase")]
struct AddData {
    delta: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadData {
    value: u32,
}

type ReadResponse = Payload<ResponseBody<ReadData>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum MessageBody {
    #[serde(rename = "add")]
    Add {
        msg_id: u32,
        #[serde(flatten)]
        data: AddData,
    },
    #[serde(rename = "read")]
    Read { msg_id: u32 },
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: MessageBody,
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
    match msg.body {
        MessageBody::Add { msg_id, data } => {
            let mut buf = [0u8; 4];
            node.store.seek(SeekFrom::Start(0))?;
            let _ = node.store.read(&mut buf)?;

            let old = u32::from_le_bytes(buf);
            let new = old + data.delta;
            node.store.seek(SeekFrom::Start(0))?;
            node.store.write_all(&new.to_le_bytes())?;

            io::to_writer(
                writer,
                &Payload {
                    src: msg.dest,
                    dest: msg.src,
                    body: ResponseBody::<()> {
                        typ: "add_ok".to_string(),
                        in_reply_to: msg_id,
                        data: None,
                    },
                },
            )?;
        }

        MessageBody::Read { msg_id } => {
            let mut buf = [0u8; 4];
            node.store.seek(SeekFrom::Start(0))?;
            let _ = node.store.read(&mut buf)?;
            let v = u32::from_le_bytes(buf);

            io::to_writer(
                writer,
                &ReadResponse {
                    src: msg.dest,
                    dest: msg.src,
                    body: ResponseBody {
                        typ: "read_ok".to_string(),
                        in_reply_to: msg_id,
                        data: Some(ReadData { value: v }),
                    },
                },
            )?;
        }
    };
    Ok(())
}
