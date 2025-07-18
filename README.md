# gossip-glomers

## Project Inspiration

1. I've been wanted to write Rust again.
1. I knew I wanted to do something distributed-systems related.
1. I figured I'd actually implement https://github.com/mccurdyc/lodiseval in Rust

 - But I figured I'd need a better starting place to first re-learn the basics --- with some hand-holding --- of Rust before trying
 to solve an actual problem. That's when I came across [Jon Gjengset's recorded stream](https://youtu.be/gboGyccRVXI?si=h51BZDIr1LPFWxFU).

1. Then I realized that lodiseval is actually kind of a re-implementation of [maelstrom](https://github.com/jepsen-io/maelstrom) so the Gossip Glomers
challenge was perfect. Everything kind of came full-circle because Jepsen was partially an inspiration for the lodiseval concept in the first place.

## (Re-)Learning Rust

While doing this challenge, I've been reading a book titled [_Zero to Production in Rust_](https://www.amazon.com/Zero-Production-Rust-introduction-development/dp/B0BHLDMFDQ/ref=sr_1_1?crid=1TA1X83N68E12&dib=eyJ2IjoiMSJ9.3PXaR8G-D0CsuTHFajbMtqgg2dshDRkAfoGeqSf-uMYpDEqs3nQVzkzhLvWAMfJjtDGOgjhYHtvn1KkN_hMmvnAeqo5wb1QwwsEz__O_OVO0zaXUTf2hEYza_Af1BnawPhCiSMm30-UqSPqpPFms54BUggOR8lW7_hAaqo1pNHpXHWsPiGLg3-v3jX7oSXsElR8yXD1fu5EFSZM4i0iPlMn9cJ73q3lB6hhNA9J_z-U.BWPWQtikOFIqn-pKOPxEHapd4Bm3o-HYf7pglObXCJs&dib_tag=se&keywords=production+rust&qid=1724622092&sprefix=production+rust%2Caps%2C103&sr=8-1)
which actually contains many of the best practices that I follow generally. Therefore,
I am using this as the first implementation of what I will end up codifying in my
[nix-templates](https://github.com/mccurdyc/nix-templates) repository for Rust projects.

## Usage

1. `nix run '.#echo'`

### Docker

```bash
docker run -it echo
```

## Run a Maelstrom Test

```bash
nix build '.#checks.x86_64-linux.echo' --print-build-logs --keep-failed
nix build '.#checks.x86_64-linux.unique' --print-build-logs --keep-failed
nix build '.#checks.x86_64-linux.broadcast' --print-build-logs --keep-failed
nix build '.#checks.x86_64-linux.counter' --print-build-logs --keep-failed
```

### Debugging Failures

Note the `--keep-failed` flag we use above, this keeps the sandbox build directory at `/tmp/nix-build-.../build/<hash>-source/`.
Then, we can copy the maelstrom `store/` output from that directory to local and then
run maelstrom's `serve` for further analysis

```bash
java -Djava.awt.headless=true -jar "./maelstrom.jar" serve
```

Alternatively, you can use `breakpointHook` to be dropped into the sandbox container directly.
However, running maelstrom's `serve` command will NOT work from within the sandboxed
container. It runs but the ports are not accessible by the host machine. You would
have to set up some kind of bridge interface to bridge to the build containers.
