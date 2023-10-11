# BGP routing related tools

## `bmp-speaker`

`bmp-speaker` is a command line tool that simulates an [RFC 7854](https://datatracker.ietf.org/doc/rfc7854/) BMP (BGP Monitoring Protocol) monitored router. The tool connects to a BMP monitoring station and then offers a [REPL](https://www.digitalocean.com/community/tutorials/what-is-repl)-like interface from which you can instruct it to send BMP protocol messages to the monitoring station.

The tool was created to aid with testing of BMP monitoring stations and assumes detailed knowledge of BMP message structure and content as well as the correct sequence in which the various BMP message types should be sent. It thus also allows sending BMP messages in the incorrect order and with logically inconsistent content. For example, sending a Peer Down Notification message without having first sent the corresponding Peer Up Notification message.

### Disclaimer

The functionality availability is the subset that was needed by the authors until now. No guarantees are made about correctness of the messages produced or that RFC 7854 is fully implemented.

### Contributions

Contributions and requests are welcome via GitHub pull requests and issues.

This version of the tool uses some functionality from the NLnet Labs [routecore](https://github.com/NLnetLabs/routecore/) crate with the intention that all BMP and BGP message construction is done by that crate. Some contributions may therefore be better suited as contributions to the routecore repository rather than to this repository.

### Installation

As `bmp-speaker` is not yet published to `crates.io` installation via `cargo install` requires that you point `cargo` to the `bmp-speaker` Git repository:

```
cargo install routes --bin bmp-speaker --version 0.1.0-dev --git https://github.com/NLnetLabs/routes
```

### Usage

```
bmp-speaker --server <BMP monitoring station ip or hostname>[:<port>]
```

Executing this command will drop you into the REPL from where you can instruct `bmp-speaker` to send BMP messages, for example:

```
> initiation my-sys-name "my-sys-desc long description"
> peer_up_notification global 0 10.0.0.1 12345 127.0.0.1 80 81 888 999 0 0
> route_monitoring global 0 10.0.0.1 12345 0 none "e [123,456,789] 10.0.0.1 BLACKHOLE,123:44 127.0.0.1/32"
```

One can also use the tool in a batch-like mode by storing the commands to send in a text file and piping them into the tool. Beware however that the tool exits when the input pipe is closed.

### Documentation

TO DO