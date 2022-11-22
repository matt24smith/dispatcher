# Network Dispatcher
Client/proxy/server network socket dispatcher. Streams files and raw socket 
data over the network.

### Features
- [X] Stream arbitrary data over the network
- [X] Complete UDP networking stack
  - Send, proxy, reverse-proxy, and receive to/from multiple endpoints simultaneously
  - Stream multiplexing and aggregation
  - [Multicast](https://en.wikipedia.org/wiki/Multicast) reverse-proxy IP routing
  - Hostname resolution
- [X] Fast
  - 500+ Mbps read/transfer/write speed via UDP
- [X] Minimal 
  - Compiled binaries ~350Kb each
  - Tiny memory footprint
  - Stateless: no shared resources between threads. Communication between threads are routed via UDP multicast

### Compatible with
- [X] UDP
- [X] TCP (partial support via `proxy` or `reverse_proxy`)
- [ ] SSL (planned feature)
- [X] IPv4
- [X] IPv6
- [X] Unix/Linux/Mac
- [X] Windows


## Install
Install utils from source using cargo, e.g.
```
git clone https://github.com/matt24smith/dispatcher.git
cargo install dispatcher/reverse_proxy
```


## Operation
Use `--help`/`-h` to view help messages.
The `--tee`/`-t` flag may be used to copy input to stdout.

### Client

Stream data from the client to logging servers. The `--server_addr` option may 
be repeated for multiple server hosts. To accept input from stdin, use `--path "-"`

```
client --path '/dev/random' --server_addr 'localhost:9921'
```

### Proxy

Forward UDP packets from listening port to downstream hosts. 
Options `--listen_addr` and `--downstream_addr` may be repeated for multiple 
endpoints.

```
proxy --listen_addr '0.0.0.0:9921' --downstream_addr 'localhost:9922'
```

### Reverse-Proxy

Forward UDP packets from upstream to new incoming TCP client connections.
UDP packets will be routed via the multicast channel to listeners on each TCP 
client handler.

```
reverse_proxy --udp_listen_addr '0.0.0.0:9921' --tcp_output_addr '0.0.0.0:9921' --multicast_addr '224.0.0.1:9922'
```

### Server

Start the logging server. The `--listen_addr` option may be repeated to listen 
for incoming messages from multiple sockets.

```
server --path logfile.log --listen_addr '0.0.0.0:9920' --listen_addr '[::]:9921'
```


## Motivation

- Complete yet barebones distributed networks framework for e.g. telemetry or sensor data
- Zero-configuration, simple operation and deployment
- Leverage benefits of UDP protocol:
  - Ability to merge data streams from many sources
  - Stream multiplexing and redistribution
  - UDP multicasting enables stateless, scaleable reverse-proxy
- Prioritizing cross-compatability, simplicity, security, and performance

## Alternatives

- cURL
- [Netcat](https://en.wikipedia.org/wiki/Netcat) Point-to-point communications with complete feature set 
- [Nginx](https://en.wikipedia.org/wiki/Nginx) Feature-rich proxy server with static file serving, file caching, and load balancing
- [Websocat](https://github.com/vi/websocat) Command-line client for websockets

