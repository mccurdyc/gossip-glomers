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

- [ ] Replace `ubuntu` with `alpine` in the dev image
    - Will need to fix dynamic linking

    ```
    apk add libgcc musl-dev gcc lld clang build-base
    ```

    ```
    export RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=lld"
    ```
    - https://numbersandreality.com/computers/linux/rust-lang-on-alpine/


    ```
    ld.lld: error: undefined symbol: fstat64
    ```

    https://www.perplexity.ai/search/cannot-find-scrt1-o-alpine-lin-MHV7NXXsRWOSqBCrVeUePw

    <blockquote>
    The error "undefined symbol: fstat64" typically arises when compiling or linking applications on Alpine Linux, which uses musl libc instead of glibc. The fstat64 function is not available in musl libc, as it has removed compatibility symbols with the 64 suffix (like fstat64, ftello64, etc.) in favor of standard POSIX functions.
    ...
    Libraries or code that reference fstat64 are likely designed for glibc
    </blockquote>

    - https://users.rust-lang.org/t/dynamic-linking-with-musl-target/27380/2
    - https://users.rust-lang.org/t/link-the-rust-standard-library-dynamically/29175

