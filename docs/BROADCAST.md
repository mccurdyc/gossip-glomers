# Multi-Node Broadcast

Gossip is a fun topic. The name gets you immediately thinking about HUMAN interactions
or how information or diseases spread through communities of humans.

I think the driving principle for my design will be to stay as simple as possible
as it seems that most often a simple solution is "good enough".

This is also the first task where I feel like I could try multiple approaches.
This was the motivation behind me wanting to build a distributed system test
framework. I wanted to try and evaluation different approaches. I wanted to
take a research-y approach. For example, how well does random neighbor sampling work?
What is the minimal percentage to have a high confidence or probability that all
nodes received a message? I've read 63%, but have yet to read this in a paper.

I've gone down a rabbit hole of trying to find research papers on the topic of
epidemiology that are applicable to computer science networks. What is the seminal
paper on this topic? I've also keen to have my own ideas and proposal BEFORE I
read the research in this area. I don't "just want the answer". I want to practice
thinking on my own. This isn't a homework assignment, this is for fun and for
learning. There's no time limit.

From my minimal reading so far, the "answer" tends to be "it depends on the network".
But I'm not looking for the ideal approach, I'm looking for the "golden path"
"good enough" / "where to start" approach.

I've got the following papers queued:

- _Gossip and Epidemic Protocols_ - Alberto Montresor
- _Epidemic Algorithms for Replicated Database Maintenance_ - Demers et. al.
- _Epidemic process in complex networks_ - Pastor-Satorras et. al.
- _Networks and the Epidemiology of Infectious Disease_ - Danon et. al.

I've read the Paxos, "Simple Paxos" and Raft papers a couple years ago.

I've also got a few blog posts queued:
- _Consensus Algorithms at Scale_ - PlanetScale

But before I read the above, I want to make my own proposal:

# My Proposals

I don't think any of these approaches are novel. These are just the approaches
that immediately come to mind for me.

We have a couple groups: we have nodes and we have messages. We want to have a
high confidence that all messages get to all nodes, eventually.

I do plan on over-engineering my solution a bit for the Maelstrom context.

Mostly because I want to have my solutions "written down" somewhere as a reference
to test in the future.

## Hierarchical Gossip

In an organization there are leaders (or management, or tech leaders). Not all information (or messages)
is disseminated down to all employees (or nodes) all the time, immediately.
Information has various weights and necessary speed. "There's a fire" needs to
disseminate everywhere, fast. There's no "vetting" or planning before sending
this message to anyone; you see someone, you tell them "get out now". But most
information is disseminated slow. It goes through chains of review,
starts in small bubbles, then is slowly disseminated across leadership and then across
employees, based on considerations of the network.

Now, at this point, we've got the following concepts:

- nodes
- neighborhoods (or teams)
- local leaders of neighborhoods (leadership)
- hierarchy of neighborhoods
- messages
- message weights (priority)
- message sensitivity
- message "flow" through the network

## Random Sampling

The last thought that I have is around "random sampling". And maybe this is just
a simpler alternative to hierarchy (that may honestly be better). I learned in
my college research days that simple random sampling is actually really good
[_mrstudyr: Retrospectively Studying the Effectiveness of Mutant Reduction Techniques_](https://philmcminn.com/publications/mccurdy2016.pdf).

We could randomly sample across a few dimensions:

- nodes
- messages
- how often the neighborhoods changes

Recalling back to COVID-19 days, there were two key bits of advise:

- "Stay at home, unless necessary" or "minimize interaction"
- Get vaccinated

If we negate these two statements to guarantee spread of information we get the following:

- travel or move between neighborhoods often
- try to accept as many messages as possible (don't deny messages)

# Implementation

## Using a Niave, Simple, Filesystem File Lock

System File locking on every single message leads to severe congestion in a 5-node, 10RPS system.

TODO: read LSPI p233, 1117

We should prove that this is actually the point of congestion. I mean I'm pretty confident it is, but we
should get some practice with Flamegraphs and (possibly) tracing instrumentation in Rust.

Right now, we are doing the most niave way which would guarantee consistenty, but at an obvious cost to
it's ability to process messages. We need to "dial back" consistency in order to support the rates of requests.

At any time, a node can go offline or the network can go down. Instead of using a single, shared, store, each
node could get a unique file. Then, we can drop locking. But then _we_ need to handle ensuring consistency
instead of relying on the operating system file locking for consistency.

For ensuring constency, we likely can't just rely on a single broadcast message being forwarded.
We will have to some consensus. "What do you have?" / "Here's what I have.". But let's keep it simple first.
Let's try just forwarding a single message with unique store files.

Okay, another really cool thing to experience! We are learning from first principles! We see how a message in
indefinitely gossiped. A single `message: 0` is sent to a node, it sends it to it's neighbors which in turn
send it to theirs ad infinitum. We need a base case i.e., a way to say "I've seen this" and/or for nodes to share
their states.

Maybe a node keeps track of "share state" of a message in its store also:

0: [] - i've seen
0: [1] - i've seen and shared with node 1
0: [1,2] - i've seen and shared with node 1 and node 2

We don't need to guarantee that they've actually recorded the message, just that we've sent it.

Was thinking of bloom filters for the "seen states" then we could compare hashes across nodes.

## Messages have a "depth"

What if state is on the message instead of storing "share state"

Messages would be "heavier" than necessary.
