# Left Off

- Client Read Timeout
    - read requests failing

    ```
    >> input: Read(ReadPayload { src: "c6", dest: "n0", body: ReadReqBody { typ: "read", msg_id: 1 } })
    "listening: failed to fill whole buffer"
    ```

# Future

- [ ] Avoid struct repetition in modules (e.g., `Payload`, `ReqBody`, etc.)
- [ ] Avoid repetition in message matching
    - Could `listen` accept just a list of "matches" we wanted to add specifically for this test?
