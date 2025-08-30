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
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
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
    S: store::Store + std::fmt::Debug,
{
    // https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html
    // from_reader will read to end of deserialized object
    let msg: Payload<RequestBody> = serde_json::from_reader(reader)?;
    match msg.body {
        RequestBody::Topology {
            msg_id,
            topology: _, // NOTE: we don't use the topology message because I'm trying to define my
                         // own random neighborhood generator in node.init().
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

            // Stores the message in the Store to later be served by a read().
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
    use std::{collections::HashSet, io::Cursor};

    struct BroadcastCase {
        name: String,
        setup: fn(&mut store::MemoryStore) -> Vec<u8>,
        expected: HashSet<&'static str>,
    }

    #[test]
    fn broadcast() {
        // TODO: these tests dont actually test that messages are broadcasted.
        // They don't setup neighborhoods, etc.
        //
        // These tests need to be reworked to have full setups:
        // get a topology message, get a broadcast, get a read, then assert.
        // The assertion also needs to check that the neighbor nodes received the broadcast via a read.
        // We only need to verify one layer of broadcasts, not broadcasts of broadcasts.
        let test_cases = vec![BroadcastCase {
            name: String::from("one"),
            setup: |s: &mut store::MemoryStore| -> Vec<u8> {
                let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                    .expect("failed to get config");

                // TODO: needs to be instantiated with a neighborhood here since it's in an init
                // message that instantiates a node's neighborhood.
                let mut n = node::Node::<store::MemoryStore>::new(s);
                n.neighborhood
                    .insert(String::from("n2"), node::Metadata { priority: 99 });
                n.neighborhood
                    .insert(String::from("n3"), node::Metadata { priority: 99 });

                let messages = vec![
                    r#"{"src":"c1","dest":"n1","body":{"type":"topology","msg_id":1,"topology":{"n1":["ignored","ignored"]}}}"#,
                    r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":2,"message":222}}"#,
                    r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":3,"message":333}}"#,
                    r#"{"src":"c1","dest":"n1","body":{"msg_id":5,"type":"read"}}"#,
                ];

                let mut sent = Cursor::new(Vec::<u8>::new());

                for m in messages {
                    let read_cursor = Cursor::new(String::from(m));
                    listen(&mut n, read_cursor, &mut sent, &cfg).expect("failed to listen");
                }

                sent.into_inner()
            },
            // We need to keep a list of messages that a node "sends".
            // To assert that it sends a re-broadcast message. Instead of checking node states.
            expected: HashSet::from([
                r#"{"src":"n1","dest":"c1","body":{"type":"topology_ok","in_reply_to":1}}"#,
                // TODO: this has the potential to race with the re-broadcasts
                // We are iterating over a HashMap which is deliberately non-deterministic.
                // In the tests, we don't care about order, just existence. So let's fix that.
                // Do we use a HashMap instead of a Vec or some "vec contains" capability?
                // Could use a HashSet for O(1) lookups!
                r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":2}}"#,
                r#"{"src":"n1","dest":"n2","body":{"type":"broadcast","msg_id":222222,"message":222}}"#,
                r#"{"src":"n1","dest":"n3","body":{"type":"broadcast","msg_id":333333,"message":222}}"#,
                // TODO: this has the potential to race with the re-broadcasts
                r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":3}}"#,
                r#"{"src":"n1","dest":"n2","body":{"type":"broadcast","msg_id":444444,"message":333}}"#,
                r#"{"src":"n1","dest":"n3","body":{"type":"broadcast","msg_id":555555,"message":333}}"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":5,"messages":[222,333]}}"#,
            ]),
        }];

        for case in test_cases {
            info!("TEST: {:?}", case.name);

            let mut s =
                store::MemoryStore::new(Vec::<u8>::new()).expect("failed to create memory store");

            let sent = (case.setup)(&mut s);

            let sent_str = String::from_utf8(sent).expect("Invalid UTF-8");
            let sent_lines: HashSet<&str> = HashSet::from_iter(sent_str.lines());

            assert_eq!(
                case.expected,
                sent_lines,
                "Sets don't match: \n\tactual has {:#?}, \n\texpected has {:#?}\n",
                sent_lines.difference(&case.expected).collect::<Vec<_>>(),
                case.expected.difference(&sent_lines).collect::<Vec<_>>()
            );
        }
    }
}
