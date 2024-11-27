#[cfg(test)]
mod tests {
    use app::{broadcast, config, counter, echo, node, store, unique};
    use std::io::{Cursor, Write};
    use std::vec::Vec;
    use tempfile::NamedTempFile;
    use tracing::info;

    struct MockTime;
    impl app::config::TimeSource for MockTime {
        fn now(&self) -> std::time::SystemTime {
            std::time::SystemTime::UNIX_EPOCH
        }
    }

    #[test]
    fn run() {
        let test_cases = vec![(
            r#"{"id":42,"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"32","node_ids":["n1","n2","n3"]}}
"#,
            r#"{"src":"n1","dest":"c1","body":{"type":"init_ok","in_reply_to":1}}
"#,
        )];

        for (input, expected) in test_cases {
            let buf: Vec<u8> = Vec::new();
            let s = store::MemoryStore::new(buf).expect("failed to create store");
            let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                .expect("failed to get config");
            let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            n.run(read_cursor, write_cursor, echo::listen, cfg)
                .expect("Node did NOT run");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }

    #[test]
    fn echo() {
        let test_cases = vec![
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

        let buf: Vec<u8> = Vec::new();
        let s = store::MemoryStore::new(buf).expect("failed to create store");
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
    }

    #[test]
    fn unique() {
        let test_cases = vec![
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

        let buf: Vec<u8> = Vec::new();
        let s = store::MemoryStore::new(buf).expect("failed to create store");
        let cfg = config::Config::<MockTime>::new(&MockTime {}).expect("failed to get config");
        let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            unique::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }

    #[test]
    fn broadcast() {
        let test_cases = vec![
            (
                "one",
                Box::new(|| -> store::MemoryStore {
                    let buf: Vec<u8> = Vec::new();
                    store::MemoryStore::new(buf).expect("failed to create store")
                }) as Box<dyn Fn() -> store::MemoryStore>,
                r#"{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":1, "message": 42}}
        "#,
                r#"{"src":"n1","dest":"c1","body":{"type":"broadcast_ok","in_reply_to":1}}
"#,
            ),
            (
                "two",
                Box::new(|| -> store::MemoryStore {
                    let buf = String::from("1\n2\n3\n");
                    let s = store::MemoryStore::new(buf.as_bytes().to_vec())
                        .expect("failed to create store");

                    info!("store: {:?}", s);
                    s
                }) as Box<dyn Fn() -> store::MemoryStore>,
                r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
        "#,
                r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"messages":[1,2,3]}}
"#,
            ),
            (
                "three",
                Box::new(|| -> store::MemoryStore {
                    let buf: Vec<u8> = Vec::new();
                    store::MemoryStore::new(buf).expect("failed to create store")
                }) as Box<dyn Fn() -> store::MemoryStore>,
                r#"{"src":"c1","dest":"n1","body":{"type":"topology","msg_id":101,"topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}
        "#,
                r#"{"src":"n1","dest":"c1","body":{"type":"topology_ok","in_reply_to":101}}
"#,
            ),
        ];

        for (name, setup_fn, input, expected) in test_cases {
            info!("TEST: {:?}", name);
            let s = setup_fn();
            let cfg = config::Config::<config::SystemTime>::new(&config::SystemTime {})
                .expect("failed to get config");
            let mut n: node::Node<store::MemoryStore> = node::Node::new(s);

            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            broadcast::listen(&mut n, read_cursor, &mut write_cursor, &cfg).expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }

    #[test]
    fn counter() {
        // setup closure
        let setup = |starting_value: u32| {
            let mut f = NamedTempFile::new().expect("Failed to create test tempfile");
            let buf = u32::to_be_bytes(starting_value);
            f.write_all(&buf)
                .expect("Failed to writing starting value to file");
            f
        };

        let cleanup = |f: NamedTempFile| {
            f.into_temp_path()
                .close() // closes and removes
                .expect("Failed to cleanup test tempfile")
        };

        let test_cases = vec![(
            0,
            r#"{"src":"c1","dest":"n1","body":{"type":"read","msg_id":100}}
"#,
            r#"{"src":"n1","dest":"c1","body":{"type":"read_ok","in_reply_to":100,"value":0}}
"#,
        )];

        let buf: Vec<u8> = Vec::new();
        let s = store::MemoryStore::new(buf).expect("failed to create store");
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
