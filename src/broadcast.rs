use crate::payload::{Payload, ResponseBody};
use crate::{config, io, node, store};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use tracing::info;

// In this challenge, you’ll need to implement a broadcast system that gossips
// messages between all nodes in the cluster. Gossiping is a common way to propagate
// information across a cluster when you don’t need strong consistency guarantees.
//
// Data can be stored in-memory as node processes are not killed by Maelstrom.
// - It should store the "message" value locally so it can be read later. When a "read"
// request is sent.
//
// - Your node should propagate values it sees from broadcast messages to the other nodes in the cluster
// - It can use the topology passed to your node in the topology message or you can build your own topology.
//
// The 63% Rule
// In epidemic spreading models, once approximately 63% of susceptible nodes have been "infected" (received the message), there's a very high probability the epidemic will reach essentially 100% coverage.
// 50-70% initial coverage for reliable full propagation
// Fanout of 3-5 nodes per gossip round
// Log(N) rounds to reach full coverage (where N = total nodes)

#[derive(Serialize, Deserialize, Debug)]
struct ReadRespData {
    messages: Vec<u32>,
}

// I use "untagged" in the following because the type tag differs based on the message.
// I could split the Init message into a separate enum so that I could infer
// the type based on different internal fields in the message body.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RequestBody {
    Topology {
        msg_id: u32,
        topology: HashMap<String, Vec<String>>,
    },
    Broadcast {
        msg_id: u32,
        message: u32,
    },
    Read {
        msg_id: u32,
    },
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
    S: store::Store + std::fmt::Debug,
{
    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Payload<RequestBody> = serde_json::from_reader(reader)?;
    match msg.body {
        RequestBody::Topology {
            msg_id,
            topology: _,
        } => {
            io::to_writer(
                writer,
                &Payload {
                    src: msg.dest,
                    dest: msg.src,
                    body: ResponseBody::<()> {
                        typ: "topology_ok".to_string(),
                        in_reply_to: msg_id,
                        data: None,
                    },
                },
            )?;
        }

        RequestBody::Broadcast { msg_id, message } => {
            // "hotness" of a message
            for (k, _) in node.neighborhood.iter() {
                io::to_writer(
                    writer,
                    &Payload {
                        src: node.id.clone(),
                        dest: k.to_owned(),
                        body: RequestBody::Broadcast { msg_id, message },
                    },
                )?;
            }

            // Stores the message in the Store
            serde_json::ser::to_writer(&mut node.store, &message)?;
            writeln!(node.store)?;

            io::to_writer(
                writer,
                &Payload {
                    src: msg.dest,
                    dest: msg.src,
                    body: ResponseBody::<()> {
                        typ: "broadcast_ok".to_string(),
                        in_reply_to: msg_id,
                        data: None,
                    },
                },
            )?;
        }

        RequestBody::Read { msg_id } => {
            // Make sure we reset the file offset
            // TODO: this makes no sense for stores that are NOT file-based (maybe)
            // TODO: this breaks the interface to the store. The store should not
            // expose implementation details to consumers like this.
            node.store.rewind()?;

            // Move this to the write; writes should do the heavy lifting, reads should be fast
            let mut seen = Vec::<u32>::new();
            let lines = node.store.lines();
            for line in lines {
                let v: u32 = line.expect("failed to extract line").parse()?;
                seen.push(v);
            }

            io::to_writer(
                writer,
                &Payload {
                    src: msg.dest,
                    dest: msg.src,
                    body: ResponseBody {
                        typ: "read_ok".to_string(),
                        in_reply_to: msg_id,
                        data: Some(ReadRespData { messages: seen }),
                    },
                },
            )?;
        }

        RequestBody::Other => {
            info!("other: {:?}", msg);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    enum Store {
        Memory(store::MemoryStore),
        File(store::FileStore),
    }

    struct BroadcastCase {
        name: String,
        s: fn() -> Store,
        setup_fn: fn(&mut Store),
        input: String,
        expected: String,
    }

    #[test]
    fn broadcast() {
        let test_cases = vec![
            BroadcastCase {
                name: String::from("one"),
                s: || -> Store {
                    Store::Memory(
                        store::MemoryStore::new(Vec::<u8>::new())
                            .expect("failed to create memory store"),
                    )
                },
                setup_fn: |s: &mut Store| match s {
                    Store::Memory(_) => {}
                    _ => {
                        panic!("unexpected branch");
                    }
                },
                input: String::from(
                    r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":1, "message": 42}}
            "#,
                ),
                expected: String::from(
                    r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":1}}
            "#,
                ),
            },
            BroadcastCase {
                name: String::from("two"),
                s: || -> Store {
                    Store::Memory(
                        store::MemoryStore::new(Vec::<u8>::new())
                            .expect("failed to create memory store"),
                    )
                },
                setup_fn: |s: &mut Store| match s {
                    Store::Memory(v) => v
                        .write_all(b"1\n2\n3\n")
                        .expect("failed to write to memory store"),
                    _ => {
                        panic!("unexpected branch");
                    }
                },
                input: String::from(
                    r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
            "#,
                ),
                expected: String::from(
                    r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"messages":[1,2,3]}}
            "#,
                ),
            },
            BroadcastCase {
                name: String::from("three"),
                s: || -> Store {
                    Store::Memory(
                        store::MemoryStore::new(Vec::<u8>::new())
                            .expect("failed to create memory store"),
                    )
                },
                setup_fn: |s: &mut Store| match s {
                    Store::Memory(_) => {}
                    _ => {
                        panic!("unexpected branch");
                    }
                },
                input: String::from(
                    r#"{"src":"c1","dest":"n1","body":{"type":"topology","msg_id":101,"topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
            "#,
                ),
                expected: String::from(
                    r#"{"src":"n1","dest":"c1","body":{"type":"topology_ok","in_reply_to":101}}
            "#,
                ),
            },
            BroadcastCase {
                name: String::from("four"),
                s: || -> Store {
                    // TODO: this should become a tempfile
                    let p = std::path::Path::new("./store.txt");
                    Store::File(store::FileStore::new(p).expect("failed to create file store"))
                },
                setup_fn: |s: &mut Store| match s {
                    Store::File(v) => v
                        .write_all(b"1\n2\n3\n")
                        .expect("failed to write to file store"),
                    _ => {
                        panic!("unexpected branch");
                    }
                },
                input: String::from(
                    r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
        "#,
                ),
                expected: String::from(
                    r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"messages":[1,2,3]}}
        "#,
                ),
            },
        ];

        for case in test_cases {
            info!("TEST: {:?}", case.name);
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(case.input.as_bytes());
            let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                .expect("failed to get config");

            let mut s = (case.s)();
            (case.setup_fn)(&mut s); // calls the appropriate setup fn based on the store in
            match s {
                Store::Memory(mut v) => {
                    let mut n = node::Node::<store::MemoryStore>::new(&mut v);
                    listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");
                }
                Store::File(mut v) => {
                    let mut n = node::Node::<store::FileStore>::new(&mut v);
                    listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");
                }
            }

            assert_eq!(String::from_utf8(vec).unwrap().trim(), case.expected.trim());
        }
    }
}
