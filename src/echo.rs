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
#[serde(rename_all = "lowercase")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn echo() {
        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"echo","msg_id":1,"echo":"Please echo 35"}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"echo_ok","in_reply_to":1,"echo":"Please echo 35"}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"echo","msg_id":99,"echo":"Please echo 99"}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"echo_ok","in_reply_to":99,"echo":"Please echo 99"}}
"#,
            ),
        ];

        let buf: Vec<u8> = Vec::new();
        let mut s = store::MemoryStore::new(buf).expect("failed to create store");
        let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
            .expect("failed to get config");
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
