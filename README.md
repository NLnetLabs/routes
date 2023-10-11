# BGP routing related tools

[![crates.io](https://img.shields.io/crates/v/routes.svg?color=brightgreen)](https://crates.io/crates/routes)


## `bmp-speaker`

`bmp-speaker` is a command line tool that simulates an [RFC 7854](https://datatracker.ietf.org/doc/rfc7854/) BMP (BGP Monitoring Protocol) monitored router. The tool connects to a BMP monitoring station and then offers a [REPL](https://www.digitalocean.com/community/tutorials/what-is-repl)-like interface from which you can instruct it to send BMP protocol messages to the monitoring station.

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

### Documentation

TO DO