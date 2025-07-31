#[cfg(test)]
mod tests {
    use app::{config, echo, node, store};
    use once_cell::sync::Lazy;
    use std::io::Cursor;
    use std::vec::Vec;

    // Ensure that the `tracing` stack is only initialised once using `once_cell`
    pub static TRACING: Lazy<()> = Lazy::new(|| {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr) // all debug logs have to go to stderr
            .with_max_level(tracing::Level::DEBUG)
            .init();
    });

    #[test]
    fn run() {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        Lazy::force(&TRACING);

        let test_cases = vec![(
            r#"{"id":42,"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"32","node_ids":["n1","n2","n3"]}}
"#,
            r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
        )];

        for (input, expected) in test_cases {
            let buf: Vec<u8> = Vec::new();
            let mut s = store::MemoryStore::new(buf).expect("failed to create store");
            let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                .expect("failed to get config");
            let mut n: node::Node<store::MemoryStore> = node::Node::new(&mut s);

            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            n.run(read_cursor, write_cursor, echo::listen, cfg)
                .expect("Node did NOT run");

            assert_eq!(String::from_utf8(vec).unwrap().trim(), expected.trim());
        }
    }
}
