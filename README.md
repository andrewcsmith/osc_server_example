Requires Rust + SuperCollider. Just a quick little example to demonstrate how
to pipe commands from SuperCollider to Rust using OSC.

Uses tokio, serde, serde-osc, and some dependent crates.

## How to

In Terminal: 

```
git clone https://github.com/andrewcsmith/osc_server_example.git
cd osc_server_example
cargo run
```

...once that's running, do this in SuperCollider:

```
~addr = NetAddr.new("127.0.0.1", 6667);
~addr.sendMsg("/freq", 440);
~addr.sendMsg("/freq", 441.1);
```

You should see "new_freq: 440" and "new_freq: 441.1".

