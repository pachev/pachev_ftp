use std::fs::File;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufReader, Error as IoError};
use std::net::{TcpStream, TcpListener, Ipv4Addr, SocketAddrV4};

#[derive(Debug, Copy, Clone)]
pub enum FtpMode {
    Active(SocketAddrV4),
    Passive
}

pub struct Client {
    stream: BufReader<TcpStream>,
    mode: FtpMode
}

mod status {
    pub const OPEN_DATA_CONNECTION : i32 = 150;
    pub const SUCCESS : i32 = 200;
    pub const READY_FOR_NEW_USER : i32 = 220;
    pub const ENTERING_PASSIVE_MODE : i32 = 227;
    pub const CLOSING_DATA_CONNECTION : i32 = 226;
    pub const LOGIN_SUCCESSFUL : i32 = 230;
    pub const FILE_ACTION_OK : i32 = 250;
    pub const PATHNAME_CREATED : i32 = 257;
    pub const USERNAME_OK_NEED_PASSWORD : i32 = 331;
    pub const INVALID_USERNAME_OR_PASSWORD : i32 = 430;
    pub const NOT_LOGGED_IN : i32 = 530;
    pub const OPERATION_FAILED : i32 = 550;
}


//TODO; Reformat code to resue functions  to use for active mode and passive mode
//Meaning add an extra function that does transfering based on modes
impl Client {

    //TODO: figure out how to just have a class that I call functions from in rust
    //Writes commands to the server
    pub fn write_command(client: &mut BufReader<TcpStream>, cmd: &str) {
        client.get_mut().write(cmd.to_string().as_bytes()).expect("Something went wrong writing command");
        client.get_mut().flush().expect("Something went wrong flushing stream");
        
    }

    //reads the response back
    pub fn read_message(client: &mut BufReader<TcpStream>) -> String {
        let mut response = String::new();
        client.read_line(&mut response).expect("Could not read message");
        println!("SERVER: {}", response);

        return response;

    }

    pub fn get_code_from_respone(line: &str)->Result<i32, &'static str> {

        //Debug info can go in here, same as verbose
        // println!("response is: {}", line);
        //
        let number = match line[0..3].parse::<i32>() {
            Ok(code) => code,
            Err(_) => -1
        };

        println!("code is: {}", number);
        Ok(number)
    }

    pub fn make_dir (stream: &mut BufReader<TcpStream>, args: &str) {

    }

    pub fn list(mut stream: &mut BufReader<TcpStream>, args: &str) {
        let mut cmd = "Type A\n".to_string();
        let mut response = String::new();

        //Set transfer mode
        write_command(&mut stream, &cmd);
        response = read_message(&mut stream);
        println!("SERVER: {}", response);

        //Passive connection mode
        cmd.clear();
        cmd = "PASV\n".to_string();
        response.clear();

        write_command(&mut stream, &cmd);
        response = read_message(&mut stream);
        println!("SERVER: {}", response);

        let start_pos = response.rfind('(').unwrap() +1;
        let end_pos = response.rfind(')').unwrap();
        let substr = response[start_pos..end_pos].to_string();
        let nums : Vec<u8> = substr.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
        let ip = Ipv4Addr::new(nums[0],nums[1],nums[2],nums[3]);
        let port = to_ftp_port(nums[4] as u16, nums[5] as u16);

        let addr = SocketAddrV4::new(ip,port);

        cmd.clear();
        cmd = format!("LIST {}\n", args);
        write_command(&mut stream, &cmd);
        response.clear();

        let mut stream2 = (TcpStream::connect(addr)).expect("could not read");

        response = read_message(&mut stream);
        println!("SERVER: {}", response);
        let mut buf :Vec<u8> = Vec::new();
        (stream2.read_to_end(&mut buf)).expect("Could not read");
        let text = (String::from_utf8(buf)).expect("Could not read");

        println!("{}", text);



    }



}



//Writes commands to the server
fn write_command(client: &mut BufReader<TcpStream>, cmd: &str) {
    client.get_mut().write(cmd.to_string().as_bytes()).expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");

}

//reads the response back
fn read_message(client: &mut BufReader<TcpStream>) -> String {
    let mut response = String::new();
    client.read_line(&mut response).expect("Could not read message");
    println!("SERVER: {}", response);

    return response;

}

//helper function to turn server port into valid tcp_stream port
fn to_ftp_port(b1: u16, b2: u16) -> u16 {
        b1 *256 + b2
}




