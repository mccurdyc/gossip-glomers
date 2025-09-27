# Next

# Current State

```bash
just maelstrom-run broadcast true
# fails
```

## deduplucate received messages

It appears that nodes are receiving the same message many times i.e., deduplication of state in the
state files is not occuring.

The duplication appears to be related to two things:

1. State stores / files are not de-duplicated. (easy)

2. There is likely a race where nodes receive the same message at the same time from the maelstrom servers and
therefore in parallel share the same message.

Let's try to solve this problem a bit. Rather than just deduplicating state files. This is likely related
to the ["reducing communication"](#reducing-communication) problem highlighted below. 

## reduce communication

4. it looks like there is still WAY too much communication happening. It doesn't quite look like it's sharing
exhaustively, but pretty close.

TODO - I think we need lower the 63% random number threshold
