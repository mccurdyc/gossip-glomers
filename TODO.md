# Left Off

```
2024-12-14T15:37:26.465874Z  INFO app::node: line: "{\"id\":55,\"src\":\"c2\",\"dest\":\"n0\",\"body\":{\"type\":\"broadcast\",\"message\":12,\"msg_id\":28}}"
2024-12-14T15:37:26.465951Z  INFO app::node: >> input: Other({"id": Number(55), "src": String("c2"), "body": Object {"message": Number(12), "msg_id": Number(28), "type": String("broadcast")}, "dest": String("n0")})
2024-12-14T15:37:26.466090Z  INFO app::broadcast: << output: "{\"src\":\"n0\",\"dest\":\"c2\",\"body\":{\"type\":\"broadcast_ok\",\"in_reply_to\":28}}\n"
2024-12-14T15:37:26.547696Z  INFO app::node: line: "{\"id\":57,\"src\":\"c2\",\"dest\":\"n0\",\"body\":{\"type\":\"read\",\"msg_id\":29}}"
2024-12-14T15:37:26.547794Z  INFO app::node: >> input: Other({"id": Number(57), "dest": String("n0"), "body": Object {"msg_id": Number(29), "type": String("read")}, "src": String("c2")})
2024-12-14T15:37:26.547924Z ERROR app::node: error listening: cannot parse integer from empty string
2024-12-14T15:37:41.599888Z  INFO app::node: line: "{\"id\":58,\"src\":\"c2\",\"dest\":\"n0\",\"body\":{\"type\":\"read\",\"msg_id\":30}}"
2024-12-14T15:37:41.599924Z  INFO app::node: >> input: Other({"src": String("c2"), "id": Number(58), "dest": String("n0"), "body": Object {"msg_id": Number(30), "type": String("read")}})
2024-12-14T15:37:41.599959Z ERROR app::node: error listening: cannot parse integer from empty string
```

# Future

- [ ] Avoid struct repetition in modules (e.g., `Payload`, `ReqBody`, etc.)
- [ ] Avoid repetition in message matching
    - Could `listen` accept just a list of "matches" we wanted to add specifically for this test?
