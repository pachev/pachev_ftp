use std::io::prelude::*; //the standard io functions that come with rust
use std::io::{BufReader, Error as IoError};
use std::thread::spawn; //For threads

use std::string::String;
use std::net::{TcpStream, TcpListener, Ipv4Addr, Shutdown, SocketAddrV4};

pub const OPENNING_DATA_CONNECTION: u32 = 150;
pub const OPERATION_SUCCESS: u32 = 200;
pub const SYSTEM_RECEIVED: u32 = 215;
pub const LOGGED_EXPECTED: u32 = 220;
pub const CLOSING_DATA_CONNECTION: u32 = 226;
pub const PASSIVE_MODE: u32 = 227;
pub const LOGGED_IN: u32 = 230;
pub const CWD_CONFIRMED: u32 = 250;
pub const PATHNAME_AVAILABLE: u32 = 257;
pub const PASSWORD_EXPECTED: u32 = 331;
pub const AUTHENTICATION_FAILED: u32 = 530;
pub const GOODBYE: u32 = 221;

pub fn write_response(client: &mut BufReader<TcpStream>, cmd: &str) {
    client.get_mut()
        .write(cmd.to_string().as_bytes())
        .expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");
}

pub fn handle_user(client: &mut BufReader<TcpStream>, arg: &str) {}
