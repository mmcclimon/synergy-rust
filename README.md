# synergy-rust

This is [Synergy](//github.com/rjbs/Synergy), but in Rust. It's not useful
yet, and isn't likely to be.

## Architecture

The Perl version of Synergy is such that nearly everything has a reference to
everything else. That turns out to be really tedious and unidiomatic to
implement in Rust, because you have to think about lifetimes a lot and sending
references into threads is fraught. Plus, I only wanted to use Rust's built-in
concurrency primitives and not something else (like
[crossbeam](https://crates.io/crates/crossbeam)).

The Rust version leans heavily on the actor model. _Channels_ are how synergy
does I/O, there might be a slack channel, a twilio channel, or a console
channel. they produce events, which are responded to by _Reactors_. Here,
they're set up with mpsc::channels. In the lingo here, _Events_ flow from
channels through the hub to reactors, and _Replies_ flow from reactors through
the hub back to channels (where they are output). 

All the channels and reactors do their work in threads. Right now, the hub
does all the transmogrification of channels and events synchronously, but
this could move off-thread too, via another set of channels.


Here's a crappy sketch.


```

            World
              |
              v

      +--< Channels <--+
      |                |
      |                |
 ChannelEvents    ChannelReplies
      |                |
      |                |
      |                |
      |    +-----+     |
      +--> |     | >---+
           |     |
           | HUB |
           |     |
      +--< |     | <---+
      |    +-----+     |
      |                |
      |                |
 ReactorEvents    ReactorReplies
      |                |
      |                |
      +--> Reactors >--+
```

