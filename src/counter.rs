use crate::payload::{Payload, ResponseBody};
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
struct ReadData {
    value: u32,
}

type ReadResponse = Payload<ResponseBody<ReadData>>;

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum RequestBody {
    Add {
        msg_id: u32,
        delta: u32,
    },
    Read {
        msg_id: u32,
    },
    #[serde(other)]
    Other,
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
    let msg: Payload<RequestBody> = serde_json::from_reader(reader)?;
    info!(">> input: {:?}", msg);
    match msg.body {
        RequestBody::Add { msg_id, delta } => {
            let mut buf = [0u8; 4];
            node.store.seek(SeekFrom::Start(0))?;
            let _ = node.store.read(&mut buf)?;

            let old = u32::from_le_bytes(buf);
            let new = old + delta;
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

        RequestBody::Read { msg_id } => {
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

        RequestBody::Other => {
            info!("other: {:?}", msg);
        }
    };
    Ok(())
}
