use dns_parser::rdata::a::Record;
use dns_parser::{
    Builder,
    Error as DNSError,
    Packet,
    RData,
    ResponseCode
};
use dns_parser::{QueryClass, QueryType};
use log::*;
use simplelog::{
    Config,
    LevelFilter,
    TermLogger,
    TerminalMode,
    ColorChoice
};

use tokio::io::Result as ioResult;
use tokio::net::UdpSocket;

use std::error::Error;

use std::str;
use std::net::SocketAddr;

//use tokio::prelude::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logging();

    info!("Starting server, setting up listener for port 12345");

    let listener_socket: UdpSocket = create_udp_socket_receiver("0.0.0.0:12345").await?;
    let sender_socket: UdpSocket = create_udp_socket_sender().await?;

    start_listening_udp(listener_socket, sender_socket).await?;

    Ok(())
}


// Initializes the logging libary.
// Currently logging to console.
fn init_logging() {
    TermLogger::init(LevelFilter::Debug, Config::default()
        , TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

// Takes in a socket address and returns UDP Socket binded to the address
async fn create_udp_socket_receiver(host: &str) -> ioResult<UdpSocket> {
    debug!("Initializing listener udp socket on {host}");
    let sock = UdpSocket::bind(host).await?;
    return Ok(sock)
}

// Returns a UDP socket to be used fowarding UDP request
async fn create_udp_socket_sender() -> ioResult<UdpSocket> {
    let local_address = "0.0.0.0:0";
    let socket = UdpSocket::bind(local_address).await?;
    let socket_address: SocketAddr = "0.0.0.0.53"
        .parse::<SocketAddr>()
        .expect("Invalid fowarding address specified");
    socket.connect(&socket_address).await?;
    debug!("Initializing listener UDP socket on {local_address}");
    return Ok(socket)
}

// Takes in two UDP sockets, listener and sender, and asychronously runs the handling of
// incoming messages. Use the UDP listener socket to receive/send data from/to client.
// The UDP sender_socket forwards message, as a DNS query, and logs it to the output.
async fn start_listening_udp(mut listener_socket: UdpSocket, mut sender_socket: UdpSocket) -> ioResult<()> {
    loop {
        // Wait for request from a DNS client
        let (request,peer) = receive_request(&mut listener_socket).await?;
        // Forward the request to a remote DNS server
        let forward_response = forward_request(&mut sender_socket, &request[..]).await?;
        // The response from the remote DNS server is then send back to the
        // initial client.
        listener_socket.send_to(&forward_response[..], &peer).await?;
    }
}

// Takes in a mutable reference to a UDP socket we listen to.
// Returns a Vec and a response address.
async fn receive_request(listener_socket: &mut UdpSocket) -> ioResult<(Vec<u8>,SocketAddr)> {
    let mut buf = [0;4096];

    let (amt, peer) = listener_socket.recv_from(&mut buf).await?;
    let filled_vec = Vec::from(&mut buf[..amt]);

    return Ok((filled_vec, peer))
}

// Forward request to the provided UDP socket
async fn forward_request(sender_socket: &mut UdpSocket, request: &[u8]) -> ioResult<Vec<u8>> {
    let mut buff = [0;4096];
    debug!("Forwarding to target DNS");
    sender_socket.send(request).await?;

    let (amt,_) = sender_socket.recv_from(&mut buff).await?;
    let filled_vec = Vec::from(&mut buff[..amt]);

    return Ok(filled_vec)
}






