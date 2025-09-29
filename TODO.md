# Current State

```bash
just maelstrom-run broadcast true
# fails
```

## Handle Node being offline longer than the timeout and/or message expiration.

One thought was a node announces, "hey, I'm back"

This really boils down to message state vs neighborhood state. Where should state me stored?

Generally, you probably don't want ever-growing state on the message that you are sending over the wire (more expensive).
Keep the largest data close.

This is where having a leader that has global, committed state, is helpful. Otherwise, you
are left asking "how do I recover from being offline?".

Maybe you ask a few random (all?) neighbors, "hey what all have you seen?". Maybe there are checkpoints
too so that you don't ask them about the same thing again if you go offline twice. "What
all have you seen since last time we spoke?".

## Async Flush to disk

We could use a memorystore which is just a hashset and then have some background flush that runs on
some flush_interval. To avoid writing to a file on every message.

    1. Store message Ids in a HashSet (HashMap if we want to keep the seen_by list also)
    2. Periodically write the HashSet to a file. Keep it simple to start i.e., write on every message.
    3. Restoring state is calling HashSet::from(vec_read_from_file) on recovery.

Let's try to solve this problem a bit. Rather than just deduplicating state files. This is likely related
to the ["reducing communication"](#reducing-communication) problem highlighted below.

## Reduce Communication

4. it looks like there is still WAY too much communication happening. It doesn't quite look like it's sharing
exhaustively, but pretty close.

TODO - I think we need lower the 63% random number threshold
