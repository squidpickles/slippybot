SlippyBot
=========

This is a very simple Slack bot written in Rust.

It channels the essence of our pal, Slippy.

Running
-------
You'll need a pretty recent version of Rust, and you'll need a Slack API key.

Create a `slippybot.conf` file containing your API key, and you should be ready to go.

Installing
----------
There's a systemd service file that will let you run SlippyBot as a managed service. The `Makefile` has an `install` target that will enable it, once it's compiled.

Extending
---------
To extend SlippyBot, you can add a command to the `commands` module. Have a look at a simple one like `hello.rs` as an example. Basically, you need to implement the `Command` trait.
