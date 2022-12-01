use std::fs::OpenOptions;
use std::io::{stdout, BufWriter, Result as ioResult, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::path::PathBuf;
use std::thread::{Builder, JoinHandle};

use mproxy_socket_dispatch::{bind_socket, new_socket, BUFSIZE};

/// Client socket handler.
/// Binds a new UDP socket to the network multicast channel
fn join_multicast(addr: SocketAddr) -> ioResult<UdpSocket> {
    // https://bluejekyll.github.io/blog/posts/multicasting-in-rust/
    #[cfg(debug_assertions)]
    println!("server broadcasting to: {}", addr.ip());
    match addr.ip() {
        IpAddr::V4(ref mdns_v4) => {
            let socket = new_socket(&addr)?;
            // join multicast channel on all interfaces
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
            let bind_result = bind_socket(&socket, &addr);
            if bind_result.is_err() {
                panic!("binding to {:?}  {:?}", addr, bind_result);
            }

            Ok(socket.into())
        }
        IpAddr::V6(ref mdns_v6) => {
            let socket = match new_socket(&addr) {
                Ok(s) => s,
                Err(e) => panic!("creating new socket {}", e),
            };
            // bind to all interfaces
            //assert!(socket.set_multicast_if_v6(0).is_ok());

            // disable ipv4->ipv6 multicast rerouting
            assert!(socket.set_only_v6(true).is_ok());

            /*
            #[cfg(target_os = "macos")]
            if socket.set_multicast_if_v6(0).is_err() {
                //panic!();
            }
            */

            // join multicast channel
            if let Err(e) = socket.join_multicast_v6(mdns_v6, 0) {
                panic!("joining ipv6 multicast channel: {} {}", mdns_v6, e);
            }
            //socket.join_multicast_v6(&Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), addr.port().into())?;

            // enable broadcasting
            //socket.set_broadcast(true)?;

            let listenaddr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)),
                addr.port(),
            );
            let bind_result = bind_socket(&socket, &listenaddr);
            if bind_result.is_err() {
                panic!("binding to {:?}: {:?}", listenaddr, bind_result);
            }

            Ok(socket.into())
        }
    }
}

fn join_unicast(addr: SocketAddr) -> ioResult<UdpSocket> {
    let socket = new_socket(&addr)?;
    bind_socket(&socket, &addr)?;
    Ok(socket.into())
}

/// Create a new UDP socket and bind to upstream socket address
pub fn upstream_socket_interface(addr: String) -> ioResult<(SocketAddr, UdpSocket)> {
    let addr = addr
        .to_socket_addrs()
        .unwrap()
        .next()
        .expect("parsing socket address");
    let listen_socket = match addr.ip().is_multicast() {
        false => join_unicast(addr).unwrap_or_else(|_| panic!("failed to create unicast socket listener! {}", addr)),
        true => {match join_multicast(addr) {
            Ok(s) => s,
            Err(e) => panic!("failed to create multicast listener on address {}! are you sure this is a valid multicast channel?\n{:?}", addr, e),
        }},
    };
    Ok((addr, listen_socket))
}

/// Server UDP socket listener.
/// Binds to UDP socket address `addr`, and logs input to `logfile`.
/// Can optionally copy input to stdout if `tee` is true.
/// `logfile` may be a filepath, file descriptor/handle, etc.
pub fn listener(addr: String, logfile: PathBuf, tee: bool) -> JoinHandle<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&logfile);
    let mut writer = BufWriter::new(file.unwrap());
    let mut output_buffer = BufWriter::new(stdout());

    let (addr, listen_socket) = upstream_socket_interface(addr).unwrap();
    Builder::new()
        .name(format!("{}:server", addr))
        .spawn(move || {
            let mut buf = [0u8; BUFSIZE]; // receive buffer
            loop {
                match listen_socket.recv_from(&mut buf[0..]) {
                    Ok((c, _remote_addr)) => {
                        if tee {
                            let _o = output_buffer
                                .write(&buf[0..c])
                                .expect("writing to output buffer");
                            #[cfg(debug_assertions)]
                            assert!(c == _o);
                        }
                        let _ = writer
                            .write(&buf[0..c])
                            .unwrap_or_else(|_| panic!("writing to {:?}", &logfile));
                    }
                    Err(err) => {
                        writer.flush().unwrap();
                        eprintln!("{}:server: got an error: {}", addr, err);
                        #[cfg(debug_assertions)]
                        panic!("{}:server: got an error: {}", addr, err);
                    }
                }
                writer.flush().unwrap();
                if tee {
                    output_buffer.flush().unwrap();
                }
            }
        })
        .unwrap()
}
