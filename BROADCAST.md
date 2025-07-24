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

1. Consume the Topology message; don't consume the proposed topology, just the full list of nodes in the network
