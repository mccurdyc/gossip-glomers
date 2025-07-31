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
#[serde(rename_all = "lowercase")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use config;
    use std::io::Cursor;

    struct MockTime;
    impl config::TimeSource for MockTime {
        fn now(&self) -> std::time::SystemTime {
            std::time::SystemTime::UNIX_EPOCH
        }
    }

    #[test]
    fn unique() {
        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"generate_ok","in_reply_to":1,"id":"979f89fa9ea19c49f86ff60ea893db2d66df54d8bba01bd024ca2b837d731c6a"}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"generate","msg_id":99}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"generate_ok","in_reply_to":99,"id":"575302209a4a1459a01354f6791242f5cf469f6f0a407788f61bb4c2bf3299d0"}}
"#,
            ),
        ];

        let buf: Vec<u8> = Vec::new();
        let mut s = store::MemoryStore::new(buf).expect("failed to create store");
        let cfg = config::Config::<MockTime>::new(&MockTime {}).expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(&mut s);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap().trim(), expected.trim());
        }
    }
}
