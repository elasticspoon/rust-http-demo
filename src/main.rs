use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::{env, net::TcpListener, process};

fn main() {
    // let port = env::var("PORT").map_or_else(|_| 3000, |port| port.parse::<i32>().unwrap_or(3000));
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<i32>().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 3000));
    let listener = TcpListener::bind(addr).unwrap_or_else(|err| {
        eprintln!("Failed to bind to localhost:{}: {}", port, err);
        process::exit(1);
    });

    println!("Listening on {}", listener.local_addr().unwrap());
    for conn in listener.incoming() {
        match conn {
            Ok(_) => println!("Accepted connection"),
            Err(err) => println!("failled to accept connection: {:?}", err),
        }
    }
}
