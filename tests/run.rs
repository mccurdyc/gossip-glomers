#[cfg(test)]
mod tests {
    use app::config::{Config, SystemTime};
    use app::{broadcast, echo, node, unique};
    use once_cell::sync::Lazy;
    use std::io::Cursor;
    use std::path::Path;
    use std::vec::Vec;

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

    #[test]
    fn setup() {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        Lazy::force(&TRACING);
    }

    #[test]
    fn echo() {
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

        let mut node: node::Node = Default::default();

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            echo::listen(
                &mut node,
                read_cursor,
                &mut write_cursor,
                &mut Config::<SystemTime>::new(&SystemTime {}, Path::new("empty"))
                    .expect("failed to create config"),
            )
            .expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }

    #[test]
    fn unique() {
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

        let mut node: node::Node = Default::default();

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            unique::listen(
                &mut node,
                read_cursor,
                &mut write_cursor,
                &mut Config::<MockTime>::new(&MockTime {}, Path::new("empty"))
                    .expect("failed to create config"),
            )
            .expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }

    #[test]
    fn broadcast() {
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

        let mut node: node::Node = Default::default();

        for (input, expected) in test_cases {
            // Necessary to implement Read trait on BufReader for bytes
            let mut vec: Vec<u8> = Vec::new();
            let mut write_cursor = Cursor::new(&mut vec);
            let read_cursor = Cursor::new(input.as_bytes());

            broadcast::listen(
                &mut node,
                read_cursor,
                &mut write_cursor,
                &mut Config::<SystemTime>::new(&SystemTime {}, Path::new("empty"))
                    .expect("failed to create config"),
            )
            .expect("listen failed");

            assert_eq!(String::from_utf8(vec).unwrap(), expected);
        }
    }
}
