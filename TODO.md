# Left Off

# Future

- [ ] panic

```
{"src":"c1","dest":"n1","body":{"type":"broadcast","msg_id":1, "message": 00}}
2024-12-14T16:27:42.645114Z  INFO app::node: >> input: "{\"src\":\"c1\",\"dest\":\"n1\",\"body\":{\"type\":\"broadcast\",\"msg_id\":1, \"message\": 00}}"
thread 'main' panicked at src/bin/broadcast/main.rs:25:6:
failed to start: invalid number at line 1 column 76
```

- [ ] Avoid struct repetition in modules (e.g., `Payload`, `ReqBody`, etc.)
- [ ] Avoid repetition in message matching
    - Could `listen` accept just a list of "matches" we wanted to add specifically for this test?
