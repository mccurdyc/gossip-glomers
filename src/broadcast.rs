use crate::payload::{Payload, ResponseBody};
use crate::{config, io, node, store};
use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{TimestampSeconds, serde_as};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::time::{Duration, SystemTime};
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

#[serde_as]
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
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<TimestampSeconds<String>>")]
        expiration: Option<SystemTime>,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<MessageState>,
    },
    Read {
        msg_id: u32,
    },
    #[serde(other)]
    Other,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct BroadcastMessage {
    msg_id: u32,
    src: String,
    message: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<TimestampSeconds<String>>")]
    expiration: Option<SystemTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<MessageState>,
}

// I dont want global state syncing. I want state to be entirely filled by the message
// sharing protocol. I do believe if you are good enough at sharing messages you don't need
// to sync states. You don't ask someone "what all do you know?" you say, "have you heard X?"
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct MessageState {
    seen_by: HashSet<String>,
}

#[allow(dead_code)]
fn broadcast_naive<W, S, T>(
    node: &mut node::Node<S, T>,
    writer: &mut W,
    msg: BroadcastMessage,
) -> Result<()>
where
    W: Write,
    S: store::Store + std::fmt::Debug,
    T: config::TimeSource,
{
    for (k, _) in node.neighborhood.iter() {
        io::to_writer(
            writer,
            &Payload {
                src: node.id.clone(),
                dest: k.to_owned(),
                body: RequestBody::Broadcast {
                    msg_id: msg.msg_id,
                    message: msg.message,
                    expiration: msg.expiration.clone(),
                    state: msg.state.clone(),
                },
            },
        )?;
    }

    // Stores the message in the Store to later be served by a read().
    serde_json::ser::to_writer(&mut node.store, &msg.message)?;
    writeln!(node.store)?;

    io::to_writer(
        writer,
        &Payload {
            src: node.id.clone(),
            dest: msg.src,
            body: ResponseBody::<()> {
                typ: "broadcast_ok".to_string(),
                in_reply_to: msg.msg_id,
                data: None,
            },
        },
    )
}

fn anthropomorphic_gossip<W, S, T>(
    node: &mut node::Node<S, T>,
    writer: &mut W,
    msg: BroadcastMessage,
) -> Result<()>
where
    W: Write,
    S: store::Store + std::fmt::Debug,
    T: config::TimeSource,
{
    // 1/ messages need to have a relevancy TTL or expiration
    let expiration = msg
        .expiration
        .unwrap_or_else(|| SystemTime::now() + Duration::from_millis(100));

    let mut message_state = msg.state.unwrap_or_else(|| MessageState {
        seen_by: HashSet::<String>::new(),
    });

    if node.config.time_source.now() < expiration {
        // 2/ nodes store where they heard about a message and this occurs before sending to neighbors
        // so that a node doesn't send a message back to the neighbor that sent the message.
        message_state.seen_by.insert(msg.src.clone());

        // 3/ a node has a neighborhood that it needs to communicate with as long as the message
        // hasn't expired its relevancy.
        for (k, _) in node.neighborhood.iter() {
            // don't send the message to a node that has been confirmed to have seen the message
            if message_state.seen_by.contains(k) {
                continue;
            }

            io::to_writer(
                writer,
                &Payload {
                    src: node.id.clone(),
                    dest: k.to_owned(),
                    body: RequestBody::Broadcast {
                        msg_id: msg.msg_id,
                        message: msg.message,
                        expiration: Some(expiration.clone()),
                        state: Some(message_state.clone()),
                    },
                },
            )?;
        }
    }

    // 4/ TODO: strangers come from a node's "world" at random

    // 5/ nodes need to maintain a state store for their view of the world's state or for now their neighborhood's state
    //
    // TODO(bug) - we are just writing a message id to the store. No newlines, etc
    // What all do we actually care to persist to the store? Is it just the message id?
    //
    // Probably append-only, newline delimited, object
    //
    // Do we persist `seen_by`? - Initial thought is no, because we aren't going to be replaying
    //
    // old messages at this point, so we don't really care about those old messages. If we see an
    // old message again, I think it's safe to assume our neighborhood hasn't seen it until we
    // rebuild the "seen_by" state.
    serde_json::ser::to_writer(&mut node.store, &msg.msg_id)?;
    writeln!(&mut node.store)?;

    io::to_writer(
        writer,
        &Payload {
            src: node.id.clone(),
            dest: msg.src.clone(),
            body: ResponseBody::<()> {
                typ: "broadcast_ok".to_string(),
                in_reply_to: msg.msg_id,
                data: None,
            },
        },
    )
}

pub fn listen<R, W, T, S>(node: &mut node::Node<S, T>, reader: R, writer: &mut W) -> Result<()>
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

        RequestBody::Broadcast {
            msg_id,
            message,
            expiration,
            state,
        } => anthropomorphic_gossip(
            node,
            writer,
            BroadcastMessage {
                src: msg.src.clone(),
                msg_id,
                message,
                // expiration is set by the first gossip node
                expiration: Some(expiration.unwrap_or_else(|| {
                    let random_seconds = rand::rng().random_range(1..=5); // 1 to 5 inclusive
                    node.config.time_source.now() + std::time::Duration::from_secs(random_seconds)
                })),
                // if state is empty it's likely due to this being the first gossip node receiving
                // the message from a maelstrom server node.
                state: Some(state.unwrap_or_else(|| {
                    let mut seen_by: HashSet<String> = HashSet::new();
                    seen_by.insert(msg.src);

                    MessageState { seen_by }
                })),
            },
        )?,

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
    use std::time::{self, Duration};

    use crate::config::{MockTime, TimeSource};

    use super::*;
    use std::{collections::HashSet, io::Cursor};

    struct BroadcastCase {
        name: String,
        setup: fn(&mut store::MemoryStore, SystemTime) -> Vec<u8>,
        expected: fn(SystemTime) -> HashSet<String>,
    }

    #[test]
    fn broadcast() {
        let test_cases = vec![
            BroadcastCase {
                name: String::from("one"),
                setup: |s: &mut store::MemoryStore, expiration: SystemTime| -> Vec<u8> {
                    let cfg = config::Config::<config::MockTime>::new(config::MockTime {
                        now: time::UNIX_EPOCH + time::Duration::from_secs(1757680326),
                    })
                    .expect("failed to get config");

                    let mut n = node::Node::<store::MemoryStore, config::MockTime>::new(s, cfg);
                    n.init(String::from("n1"), vec![]);
                    n.neighborhood
                        .insert(String::from("n2"), node::Metadata { priority: 99 });
                    n.neighborhood
                        .insert(String::from("n3"), node::Metadata { priority: 99 });

                    let messages: Vec<String> = vec![
                        serde_json::to_string(&Payload {
                            src: String::from("c1"),
                            dest: String::from("n1"),
                            body: RequestBody::Topology {
                                msg_id: 1,
                                topology: HashMap::from([(
                                    String::from("n1"),
                                    vec![String::from("ignored")],
                                )]),
                            },
                        })
                        .expect("serializing topology message should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("c1"),
                            dest: String::from("n1"),
                            body: RequestBody::Broadcast {
                                msg_id: 2,
                                message: 222,
                                expiration: Some(expiration),
                                state: None,
                            },
                        })
                        .expect("serializing broadcast message should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("c1"),
                            dest: String::from("n1"),
                            body: RequestBody::Broadcast {
                                msg_id: 3,
                                message: 333,
                                expiration: Some(expiration),
                                state: None,
                            },
                        })
                        .expect("serializing broadcast message should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("c1"),
                            dest: String::from("n1"),
                            body: RequestBody::Read { msg_id: 5 },
                        })
                        .expect("serializing read message should work"),
                    ];

                    let mut sent = Cursor::new(Vec::<u8>::new());

                    for m in messages {
                        let read_cursor = Cursor::new(m);
                        listen(&mut n, read_cursor, &mut sent)
                            .expect("failed to listen to message");
                    }

                    sent.into_inner()
                },
                // We need to keep a list of messages that a node "sends".
                // To assert that it sends a re-broadcast message. Instead of checking node states.
                // "Your node should propagate values it sees from broadcast messages to the
                // other nodes in the cluster."
                // https://fly.io/dist-sys/3b/
                expected: |expiration: SystemTime| -> HashSet<String> {
                    HashSet::from([
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("c1"),
                            body: ResponseBody::<()> {
                                typ: String::from("topology_ok"),
                                in_reply_to: 1,
                                data: None,
                            },
                        })
                        .expect("serializing expected message 1 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("c1"),
                            body: ResponseBody::<()> {
                                typ: String::from("broadcast_ok"),
                                in_reply_to: 2,
                                data: None,
                            },
                        })
                        .expect("serializing expected message 2 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("n2"),
                            body: RequestBody::Broadcast {
                                msg_id: 2,
                                message: 222,
                                expiration: Some(expiration),
                                state: Some(MessageState {
                                    seen_by: HashSet::from([String::from("c1")]),
                                }),
                            },
                        })
                        .expect("serializing expected message 3 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("n3"),
                            body: RequestBody::Broadcast {
                                msg_id: 2,
                                message: 222,
                                expiration: Some(expiration),
                                state: Some(MessageState {
                                    seen_by: HashSet::from([String::from("c1")]),
                                }),
                            },
                        })
                        .expect("serializing expected message 4 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("c1"),
                            body: ResponseBody::<()> {
                                typ: String::from("broadcast_ok"),
                                in_reply_to: 3,
                                data: None,
                            },
                        })
                        .expect("serializing expected message 5 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("n2"),
                            body: RequestBody::Broadcast {
                                msg_id: 3,
                                message: 333,
                                expiration: Some(expiration),
                                state: Some(MessageState {
                                    seen_by: HashSet::from([String::from("c1")]),
                                }),
                            },
                        })
                        .expect("serializing expected message 6 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("n3"),
                            body: RequestBody::Broadcast {
                                msg_id: 3,
                                message: 333,
                                expiration: Some(expiration),
                                state: Some(MessageState {
                                    seen_by: HashSet::from([String::from("c1")]),
                                }),
                            },
                        })
                        .expect("serializing expected message 7 as json should work"),
                        serde_json::to_string(&Payload {
                            src: String::from("n1"),
                            dest: String::from("c1"),
                            body: ResponseBody::<ReadRespData> {
                                typ: String::from("read_ok"),
                                in_reply_to: 5,
                                data: Some(ReadRespData {
                                    messages: vec![2, 3],
                                }),
                            },
                        })
                        .expect("serializing expected message 8 as json should work"),
                    ])
                },
            },
            // TODO - include more sophisticated test case where priorities aren't "guaranteed" to
            // send to all nodes in a neighborhood.
            //
            // TODO - include test case where messages have expired
        ];

        for case in test_cases {
            info!("TEST: {:?}", case.name);

            let mut s =
                store::MemoryStore::new(Vec::<u8>::new()).expect("failed to create memory store");

            let expiration = MockTime {
                now: SystemTime::UNIX_EPOCH
                    + Duration::from_secs(1757680326)
                    + Duration::from_secs(300),
            }
            .now();

            let sent = (case.setup)(&mut s, expiration);

            let sent_str = String::from_utf8(sent).expect("Invalid UTF-8");
            let sent_lines: HashSet<String> =
                HashSet::from_iter(sent_str.lines().map(|s| s.to_string()));
            let expected = (case.expected)(expiration);

            assert_eq!(
                expected,
                sent_lines,
                "Sets don't match: \n\tactual has (expected doesn't) {:#?}, \n\texpected has (actual doesn't) {:#?}\n",
                sent_lines.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&sent_lines).collect::<Vec<_>>()
            );
        }
    }
}
