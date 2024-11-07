#[cfg(test)]
mod tests {
    use app::{broadcast, config, counter, echo, node, store, unique};
    use once_cell::sync::Lazy;
    use std::io::{Cursor, Write};
    use std::vec::Vec;
    use tempfile::NamedTempFile;

    struct MockTime;
    impl app::config::TimeSource for MockTime {
        fn now(&self) -> std::time::SystemTime {
            std::time::SystemTime::UNIX_EPOCH
        }
    }

    // Ensure that the `tracing` stack is only initialised once using `once_cell`
    static TRACING: Lazy<()> = Lazy::new(|| {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr) // all debug logs have to go to stderr
            .with_max_level(tracing::Level::DEBUG)
            .init();
    });

    fn setup(starting_value: u32) -> NamedTempFile {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        Lazy::force(&TRACING);

        let mut f = NamedTempFile::new().expect("Failed to create test tempfile");
        let buf = u32::to_be_bytes(starting_value);
        f.write_all(&buf)
            .expect("Failed to writing starting value to file");
        f
    }

    fn cleanup(f: NamedTempFile) {
        f.into_temp_path()
            .close() // closes and removes
            .expect("Failed to cleanup test tempfile")
    }

    #[test]
    fn echo() {
        let f = setup(0);

        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1", "n2", "n3"]}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"echo","msg_id":1,"echo":"Please echo 35"}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"echo_ok","msg_id":1,"in_reply_to":1,"echo":"Please echo 35"}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"echo","msg_id":99,"echo":"Please echo 99"}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"echo_ok","msg_id":99,"in_reply_to":99,"echo":"Please echo 99"}}
"#,
            ),
        ];

        let s = store::MemoryStore::new().expect("failed to create store");
        let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
            .expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            echo::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
        cleanup(f);
    }

    #[test]
    fn unique() {
        let f = setup(0);
        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1", "n2", "n3"]}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"generate","msg_id":1}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"generate_ok","msg_id":1,"in_reply_to":1,"id":"979f89fa9ea19c49f86ff60ea893db2d66df54d8bba01bd024ca2b837d731c6a"}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"generate","msg_id":99}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"generate_ok","msg_id":99,"in_reply_to":99,"id":"575302209a4a1459a01354f6791242f5cf469f6f0a407788f61bb4c2bf3299d0"}}
"#,
            ),
        ];

        let s = store::MemoryStore::new().expect("failed to create store");
        let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
            .expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            unique::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
        cleanup(f);
    }

    #[test]
    fn broadcast() {
        let f = setup(0);

        let test_cases = vec![
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1", "n2", "n3"]}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":1, "message": 1000}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":1}}
"#,
            ),
            (
                r#"{"src":"f11","dest":"z10","body":{"type":"broadcast","msg_id":99, "message": 42}}
"#,
                r#"{"src":"z10","dest":"f11","body":{"type":"broadcast_ok","in_reply_to":99}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"messages":[1000,42]}}
"#,
            ),
            (
                r#"{"src":"c1","dest":"n1","body":{"type":"topology","msg_id":101,"topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"topology_ok","in_reply_to":101}}
"#,
            ),
        ];

        let s = store::MemoryStore::new().expect("failed to create store");
        let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
            .expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            broadcast::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
        cleanup(f);
    }

    #[test]
    fn counter() {
        let test_cases = vec![
            (
                0,
                r#"{"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1", "n2", "n3"]}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
            ),
            (
                0,
                r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
"#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"value":0}}
"#,
            ),
        ];

        let s = store::MemoryStore::new().expect("failed to create store");
        let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
            .expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

        for (starting_value, input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            let f = setup(starting_value);

            counter::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);

            cleanup(f);
        }
    }
}
