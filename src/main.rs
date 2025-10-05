use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("localhost:3000").expect("failed to bind to port");

    for conn in listener.incoming() {
        match conn {
            Ok(_) => println!("accepted connection"),
            Err(err) => println!("failled to accept connection: {:?}", err),
        }
    }
}
