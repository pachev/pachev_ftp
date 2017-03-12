
#[cfg(test)]
// Running Some unit tests here
mod tests {
    use std::net::{Ipv4Addr, SocketAddr, TcpStream, TcpListener, Shutdown, SocketAddrV4};

    //Testing that listener wworks

    //TODO: Write more unit tests
    #[test]
    fn test_server() {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        assert_eq!(listener.local_addr().unwrap(),
                   SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080)));

    }
}
