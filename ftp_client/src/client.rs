use std::fs::File;
use std::env;
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufReader, Error as IoError};
use std::net::{TcpStream, TcpListener, Ipv4Addr, Shutdown, SocketAddrV4};

#[derive(Debug, Copy, Clone)]
pub enum FtpMode {
    Active(SocketAddrV4),
    Passive,
}


//TODO; Reformat code to resue functions  to use for active mode and passive mode
//Meaning add an extra function that does transfering based on modes
//TODO: figure out how to just have a class that I call functions from in rust
//Writes commands to the server
//
pub fn write_command(client: &mut BufReader<TcpStream>, cmd: &str) {
    client.get_mut()
        .write(cmd.to_string().as_bytes())
        .expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");

}

//reads the response back
pub fn read_message(client: &mut BufReader<TcpStream>) -> String {
    let mut response = String::new();
    client.read_line(&mut response).expect("Could not read message");
    println!("SERVER: {}", response);

    return response;

}

pub fn get_code_from_respone(line: &str) -> Result<i32, &'static str> {

    //Debug info can go in here, same as verbose
    // println!("response is: {}", line);
    //
    let number = match line[0..3].parse::<i32>() {
        Ok(code) => code,
        Err(_) => -1,
    };

    println!("code is: {}", number);
    Ok(number)
}

pub fn make_dir(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = format!("MKD {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

pub fn change_dir(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = format!("CWD {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

pub fn change_dir_up(mut stream: &mut BufReader<TcpStream>) {
    let mut cmd = "CDUP\r\n".to_string();
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

//Remove a directory

pub fn remove_dir(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = format!("RMD {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

//Delete  a File

pub fn dele(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = format!("DELE {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

//Print working dir

pub fn print_working_dir(mut stream: &mut BufReader<TcpStream>) {
    let mut cmd = "PWD\r\n".to_string();
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

//QUIT
pub fn quit_server(mut stream: &mut BufReader<TcpStream>) {
    let mut cmd = "QUIT\r\n".to_string();
    let mut response = String::new();
    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);
}

//Put a file
pub fn put(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = "Type I\r\n".to_string();
    let mut response = String::new();

    let mut lpath = String::new();
    let mut rpath = String::new();

    match args.find(' ') {
        Some(pos) => {
            lpath = args[0..pos].to_string();
            rpath = args[pos + 1..].to_string();
        }
        None => {
            lpath = args.to_string();
            rpath = args.to_string();
        }
    }

    //Set transfer mode to binary
    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);


    cmd.clear();
    cmd = "PASV\r\n".to_string();
    response.clear();
    write_command(&mut stream, &cmd);

    //TODO: Match codes before continuing
    response = read_message(&mut stream);

    let start_pos = response.rfind('(').expect("Could not read response from server") + 1;
    let end_pos = response.rfind(')').expect("could not read response form server");
    let substr = response[start_pos..end_pos].to_string();
    let nums: Vec<u8> = substr.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
    let ip = Ipv4Addr::new(nums[0], nums[1], nums[2], nums[3]);
    let port = to_ftp_port(nums[4] as u16, nums[5] as u16);
    let addr = SocketAddrV4::new(ip, port);

    //Calling for the listing of directories
    cmd.clear();
    cmd = format!("STOR {}\r\n", rpath);
    write_command(&mut stream, &cmd);
    //TODO Spawn a therad here
    let mut stream2 = (TcpStream::connect(addr)).expect("could not read");
    response.clear();
    response = read_message(&mut stream);

    let mut file = File::open(lpath).expect("Could not open this file");
    write_to_stream(&mut file, &mut stream2);
    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");

    response.clear();
    response = read_message(&mut stream);

}

//Get a file
pub fn get(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = "Type I\r\n".to_string();
    let mut response = String::new();

    let mut lpath = String::new();
    let mut rpath = String::new();

    let home_dir = env::home_dir().unwrap();
    let cur_directory = match env::current_dir() {
        Ok(pwd) => format!("{}", pwd.display()).to_string(),
        Err(err) => format!("{}/{}", home_dir.display(), rpath).to_string(),

    };

    match args.find(' ') {
        Some(pos) => {
            rpath = args[0..pos].to_string();
            lpath = args[pos + 1..].to_string();
        }
        None => {
            rpath = args.to_string();
            lpath = format!("{}/{}", cur_directory, rpath).to_string();
        }
    }
    println!("current_dir:{}  rpath:{}  home_dir:{}",
             lpath,
             rpath,
             home_dir.display());

    //Set transfer mode to binary
    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);


    cmd.clear();
    cmd = "PASV\r\n".to_string();
    response.clear();
    write_command(&mut stream, &cmd);

    //TODO: Match codes before continuing
    response = read_message(&mut stream);

    let start_pos = response.rfind('(').expect("Could not read response from server") + 1;
    let end_pos = response.rfind(')').expect("could not read response form server");
    let substr = response[start_pos..end_pos].to_string();
    let nums: Vec<u8> = substr.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
    let ip = Ipv4Addr::new(nums[0], nums[1], nums[2], nums[3]);
    let port = to_ftp_port(nums[4] as u16, nums[5] as u16);
    let addr = SocketAddrV4::new(ip, port);

    //Calling for the listing of directories
    cmd.clear();
    cmd = format!("RETR {}\r\n", rpath);
    write_command(&mut stream, &cmd);
    //TODO Spawn a therad here
    let mut stream2 = (TcpStream::connect(addr)).expect("could not connect to data streamm");
    response.clear();
    response = read_message(&mut stream);

    let mut file = File::create(lpath).expect("Could not open this file");
    write_to_file(&mut file, &mut stream2);
    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");

    response.clear();
    response = read_message(&mut stream);

}

//List Command
pub fn list(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let mut cmd = "Type A\r\n".to_string();
    let mut response = String::new();

    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);

    //Passive connection mode
    cmd.clear();
    cmd = "PASV\r\n".to_string();
    response.clear();
    write_command(&mut stream, &cmd);
    response = read_message(&mut stream);

    let start_pos = response.rfind('(').expect("Could not read response from server") + 1;
    let end_pos = response.rfind(')').expect("could not read response form server");
    let substr = response[start_pos..end_pos].to_string();
    let nums: Vec<u8> = substr.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
    let ip = Ipv4Addr::new(nums[0], nums[1], nums[2], nums[3]);
    let port = to_ftp_port(nums[4] as u16, nums[5] as u16);

    let addr = SocketAddrV4::new(ip, port);

    //Calling for the listing of directories
    cmd.clear();
    cmd = format!("LIST {}\r\n", args);
    write_command(&mut stream, &cmd);
    response.clear();

    //TODO Spawn a therad here
    println!("remote address: {}:{}", ip, port);
    let mut stream2 = (TcpStream::connect(addr)).expect("could not connect to server");

    response.clear();
    response = read_message(&mut stream);

    let mut buf: Vec<u8> = Vec::new();
    (stream2.read_to_end(&mut buf)).expect("Could not read");
    let text = (String::from_utf8(buf)).expect("Could not read");

    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");

    println!("{}", text);
    response.clear();
    response = read_message(&mut stream);

    //Printing listing
}





//helper function to turn server port into valid tcp_stream port
fn to_ftp_port(b1: u16, b2: u16) -> u16 {
    b1 * 256 + b2
}

fn write_to_stream(file: &mut File, stream: &mut TcpStream) {
    let mut buf = vec![0; 4096];
    let mut done = false;
    while !done {
        let n = file.read(&mut buf).expect("Could not read local file");
        if n > 0 {
            stream.write_all(&buf[..n]).expect("Could not write to remote locatio");
        } else {
            done = true;
        }
    }
}

fn write_to_file(file: &mut File, stream: &mut TcpStream) {
    let mut buf = vec![0; 4096];
    let mut done = false;
    while !done {
        let n = stream.read(&mut buf).expect("Could not read remote file");
        if n > 0 {
            file.write_all(&buf[..n]).expect("Could not write to local locatio");
        } else {
            done = true;
        }
    }
}
