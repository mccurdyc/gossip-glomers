# Counter notes

- https://github.com/jepsen-io/maelstrom/blob/main/doc/workloads.md#workload-g-counter

```json
{"src": "1", "dest": "2", "body": {"type": "read", "msg_id": 1}}
2025-07-19T13:10:47.573005Z  INFO app::node: >> input: "{\"src\": \"1\", \"dest\": \"2\", \"body\": {\"type\": \"read\", \"msg_id\": 1}}"
2025-07-19T13:10:47.573098Z  INFO app::counter: >> input: Read(ReadPayload { src: "1", dest: "2", body: ReadReqBody { typ: "read", msg_id: 1 } })
2025-07-19T13:10:47.573123Z  INFO app::counter: << output: "{\"src\":\"2\",\"dest\":\"1\",\"body\":{\"type\":\"read_ok\",\"in_reply_to\":1,\"value\":0}}\n"
{"src":"2","dest":"1","body":{"type":"read_ok","in_reply_to":1,"value":0}}
```

Appears to work

```json
{"src": "1", "dest": "2", "body": {"type": "add", "delta": 2, "msg_id": 1}}
2025-07-19T13:12:40.756481Z  INFO app::node: >> input: "{\"src\": \"1\", \"dest\": \"2\", \"body\": {\"type\": \"add\", \"delta\": 2, \"msg_id\": 1}}"
2025-07-19T13:12:40.756592Z  INFO app::counter: >> input: Add(AddPayload { src: "1", dest: "2", body: AddReqBody { typ: "add", msg_id: 1, delta: 2 } })
2025-07-19T13:12:40.768405Z  INFO app::counter: << output: "{\"src\":\"2\",\"dest\":\"1\",\"body\":{\"type\":\"add_ok\",\"in_reply_to\":1}}\n"
{"src":"2","dest":"1","body":{"type":"add_ok","in_reply_to":1}}
```

```json
{"src": "1", "dest": "2", "body": {"type": "read", "msg_id": 1}}
# value = 0; WRONG! Should be 2
```

```txt
# counter.txt
   
```

This is the four bytes, the first byte is the "2" that we wanted to add and the
rest are "0"s to fill the rest of the array. So it's definitely doing something
semi-correct and we're just telling it the wrong way to store these values.

Also, noticing that using `to_ne_bytes()` wrote this in big-endian. I think we will
want little-endian since it will make it easier to "grow" the value. When I say
"easier", I mean also for me, as a human, to manually check easily.

Okay, now

```
      
```

It's appended to the file rather than overwriting it.

Actually, a subsequent `read` message properly returns `4`, but then another subsequent `read` returns `0`.

So appears `reads` actually mutate the file "cursor" location. It needs to be reset.
