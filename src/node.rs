use crate::payload::{Payload, RequestBody, ResponseBody};
use crate::{config, store};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use tokio::sync::mpsc;
use tracing::{error, info};

#[derive(Debug)]
pub struct Metadata {
    pub priority: u8,
}

pub struct Node<'a, S: store::Store, T: config::TimeSource> {
    pub id: String, // include it as the src of any message it sends.
    pub msg_id: u32,
    pub world: HashMap<String, Metadata>,
    pub neighborhood: HashMap<String, Metadata>,
    // Do we use a HashSet (empty values) or HashMap with a value of `seen_by`?
    // Do we persist `seen_by`? - Initial thought is no, because we aren't going to be replaying
    //
    // old messages at this point, so we don't really care about those old messages. If we see an
    // old message again, I think it's safe to assume our neighborhood hasn't seen it until we
    // rebuild the "seen_by" state.
    pub(crate) seen: HashSet<u32>,
    pub store: &'a mut S,
    pub config: config::Config<T>,
}

impl<'a, S: store::Store, T: config::TimeSource> Node<'a, S, T> {
    pub fn new(s: &'a mut S, config: config::Config<T>) -> Self
    where
        S: store::Store,
    {
        Self {
            id: std::default::Default::default(),
            msg_id: std::default::Default::default(),
            world: std::default::Default::default(),
            neighborhood: std::default::Default::default(),
            seen: std::default::Default::default(),
            store: s,
            config,
        }
    }

    pub fn init(&mut self, node_id: String, node_ids: Vec<String>) {
        self.msg_id = 1;
        self.id = node_id;
        self.neighborhood = HashMap::new();
        self.world = HashMap::new();

        for n in node_ids {
            if n == self.id {
                continue;
            }

            // Let's pretend (for now, just use a random number generator)
            // the init message included weights
            //
            // NOTE: We don't filter the neighborhood here because filtering where
            // messages go will be up to the send/response implementation.
            // In other words, there may be cases where we want to send a message
            // to everyone and other places where we want to be selective.
            let priority: u8 = rand::random_range(0..=100);
            self.world.insert(n.clone(), Metadata { priority });

            if priority > 33 {
                self.neighborhood.insert(n.clone(), Metadata { priority });
            }
        }
    }
}

/// read reads lines from the reader. The reader in this case will be stdin which is not closed
/// until maelstrom is done with analysis.
pub(crate) async fn read<R, T>(
    reader: R,
    tx: mpsc::UnboundedSender<Payload<RequestBody<T>>>,
) -> anyhow::Result<()>
where
    R: BufRead + Send,
    T: DeserializeOwned + Send,
{
    info!("starting listener...");

    for line in reader.lines() {
        if let Ok(l) = line {
            info!(">> input: {:?}", l);
            let msg: Payload<RequestBody<T>> = serde_json::from_str(&l)?;
            if let Err(e) = tx.send(msg) {
                error!("failed while gossiping message: {}", e);
            };
            todo!("respond appropriately with a maelstrom response");
        } else {
            error!("error reading line: {:?}", line);
        }
    }
    Ok(())
}

pub(crate) async fn write<W, T>(
    _writer: W,
    _rx: mpsc::UnboundedReceiver<Payload<ResponseBody<T>>>,
) -> anyhow::Result<()>
where
    W: Write,
    T: Serialize + Send,
{
    todo!("implement node write");
}

// #[cfg(test)]
// mod tests {
//     use crate::{config, echo, node, store};
//     use once_cell::sync::Lazy;
//     use std::io::Cursor;
//     use std::time;
//     use std::vec::Vec;
//
//     // Ensure that the `tracing` stack is only initialised once using `once_cell`
//     pub static TRACING: Lazy<()> = Lazy::new(|| {
//         tracing_subscriber::fmt()
//             .with_writer(std::io::stderr) // all debug logs have to go to stderr
//             .with_max_level(tracing::Level::DEBUG)
//             .init();
//     });
//
//     #[test]
//     fn run() {
//         // The first time `initialize` is invoked the code in `TRACING` is executed.
//         // All other invocations will instead skip execution.
//         Lazy::force(&TRACING);
//
//         let test_cases = vec![
//             (
//                 r#"{"id":42,"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"32","node_ids":["n1","n2","n3"]}}
// "#,
//                 r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
// "#,
//             ),
//             (
//                 r#"{"src":"f11","dest":"z10","body":{"type":"echo","msg_id":99,"echo":"Please echo 99"}}
// "#,
//                 r#"{"src":"z10","dest":"f11","body":{"type":"echo_ok","in_reply_to":99,"echo":"Please echo 99"}}
// "#,
//             ),
//         ];
//
//         for (input, expected) in test_cases {
//             let buf: Vec<u8> = Vec::new();
//             let mut s = store::MemoryStore::new(buf).expect("failed to create store");
//             let cfg = config::Config::<config::MockTime>::new(config::MockTime {
//                 now: time::SystemTime::UNIX_EPOCH + time::Duration::from_secs(1757680326),
//             })
//             .expect("failed to get config");
//             let mut n: node::Node<store::MemoryStore, config::MockTime> =
//                 node::Node::new(&mut s, cfg);
//
//             // Necessary to implement Read trait on BufReader for bytes
//             let mut vec: Vec<u8> = Vec::new();
//             let write_cursor = Cursor::new(&mut vec);
//             let read_cursor = Cursor::new(input.as_bytes());
//
//             n.run(read_cursor, write_cursor, echo::listen)
//                 .expect("Node did NOT run");
//
//             assert_eq!(String::from_utf8(vec).unwrap().trim(), expected.trim());
//         }
//     }
// }
