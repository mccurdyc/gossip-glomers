# TODOs

```bash
just maelstrom-run echo
...
/home/mccurdyc/src/github.com/mccurdyc/gossip-glomers/src/node.rs:106:43:              
expected init request body
/home/mccurdyc/src/github.com/mccurdyc/gossip-glomers/store/echo/20250801T074719.553-0400/node-logs/n0.log

2025-08-01T11:47:20.836014Z  INFO app::node: starting listener...
2025-08-01T11:47:20.843630Z  INFO app::node: >> input: "{\"id\":0,\"src\":\"c0\",\"dest\":\"n0\",\"body\":{\"type\":\"init\",\"node_id\":\"n0\",\"node_ids\":[\"n0\"],\"msg_id\":1}}"
2025-08-01T11:47:20.843655Z  INFO app::io: << output: Payload { src: "n0", dest: "c0", body: ResponseBody { typ: "init_ok", in_reply_to: 1, data: None } }
2025-08-01T11:47:20.862022Z  INFO app::node: >> input: "{\"id\":2,\"src\":\"c2\",\"dest\":\"n0\",\"body\":{\"echo\":\"Please echo 4\",\"type\":\"echo\",\"msg_id\":1}}"
```

Actually appears to be panicking while trying to deserialize an echo message as init. So seems like `echo` messages
aren't properly falling into the `Other` branch to be deserialized by the listener functions.

Broadcasts message, but need to write tests.
    - Even single node broadcast is failing?
