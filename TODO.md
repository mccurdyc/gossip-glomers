# Left Off

- [ ] Testing unique-ids.

    - It fails, but it's unclear why. I don't believe there are
    duplicate IDs. I think it fails to respond for some requests at the end after the
    network partition.

    ```bash
    :availability {:valid? false, :ok-fraction 0.9770115},
    ```

    97% availability

    ```txt
    [2m2024-09-01T15:37:29.414033Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1379,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":255}}"
    [2m2024-09-01T15:37:29.414043Z[0m [32m INFO[0m [2mapp::unique[0m[2m:[0m >> input: Unique(Payload { src: "c8", dest: "n1", body: ReqBody { typ: "generate", msg_id: 255 } })
    [2m2024-09-01T15:37:29.414050Z[0m [32m INFO[0m [2mapp::unique[0m[2m:[0m << output: "{\"src\":\"n1\",\"dest\":\"c8\",\"body\":{\"type\":\"generate_ok\",\"msg_id\":255,\"in_reply_to\":255,\"id\":\"55d4eebbc9481af1916275b29a9c90601f4aa19f8b86676ef12b0c482930695d\"}}\n"
    ... partition
    [2m2024-09-01T15:37:29.418059Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1385,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":256}}"
    [2m2024-09-01T15:37:34.420425Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1540,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":257}}"
    [2m2024-09-01T15:37:39.420676Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1543,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":258}}"
    [2m2024-09-01T15:37:44.422355Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1546,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":259}}"
    [2m2024-09-01T15:37:49.424752Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1549,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":260}}"
    [2m2024-09-01T15:37:54.426617Z[0m [32m INFO[0m [2mapp::run[0m[2m:[0m line: "{\"id\":1552,\"src\":\"c8\",\"dest\":\"n1\",\"body\":{\"type\":\"generate\",\"msg_id\":261}}"
    ```


    Things to try

    - Maybe don't ackowledge a message until we respond?
