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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
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

pub fn listen<R, W, T, S>(node: &mut node::Node<S, T>, reader: R, writer: &mut W) -> Result<()>
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

            // TODO: The lock must extend around this
            // Reading Ch 55 (p1117) of The Linux System Interface on "File Locking"
            // describes this exact problem. `flock()` (full file lock) and `fcntl()` (file region
            // lock).
            //
            // - be careful using `stdio` for reading/writing as user-space buffers may not be
            // synced with locks. Alternatively, you must ensure you flush the buffer immediately after taking and
            // before releasing the lock.
            //
            // https://github.com/rust-lang/libs-team/issues/412
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Cursor, time};

    #[test]
    fn counter() {
        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":1}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":1,"value":0}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"add","msg_id":2,"delta":2}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"add_ok","in_reply_to":2}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"add","msg_id":3,"delta":2}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"add_ok","in_reply_to":3}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":4}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":4,"value":4}}
"#,
            ),
        ];

        let buf: Vec<u8> = Vec::new();
        let mut s = store::MemoryStore::new(buf).expect("failed to create store");
        let cfg = config::Config::<config::MockTime>::new(config::MockTime {
            now: time::UNIX_EPOCH + time::Duration::from_secs(1757680326),
        })
        .expect("failed to get config");
        let mut n: node::Node<store::MemoryStore, config::MockTime> = node::Node::new(&mut s, cfg);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut actual: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut actual);
            let read_cursor = Cursor::new(input.as_bytes());

            listen(&mut n, read_cursor, &mut write_cursor).expect("listen failed");
            assert_eq!(String::from_utf8(actual).unwrap().trim(), expected.trim());
        }
    }
}
