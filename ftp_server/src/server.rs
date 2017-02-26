use std::io::prelude::*; //the standard io functions that come with rust
use std::os::unix::fs::PermissionsExt;
use std::collections::HashMap;
use std::io::{BufWriter, BufReader};
use std::string::String;
use std::net::{TcpStream, TcpListener, Shutdown, SocketAddrV4};
use std::path::Path;
use std::fs;
use std::fs::File;

use user::User;


pub const OPENNING_DATA_CONNECTION: u32 = 150;
pub const OPERATION_SUCCESS: u32 = 200;
pub const SYSTEM_RECEIVED: u32 = 215;
pub const LOGGED_EXPECTED: u32 = 220;
pub const GOODBYE: u32 = 221;
pub const CLOSING_DATA_CONNECTION: u32 = 226;
pub const PASSIVE_MODE: u32 = 227;
pub const LOGGED_IN: u32 = 230;
pub const CWD_CONFIRMED: u32 = 250;
pub const PATHNAME_AVAILABLE: u32 = 257;
pub const PASSWORD_EXPECTED: u32 = 331;
pub const INVALID_USER_OR_PASS: u32 = 430;
pub const NOT_UNDERSTOOD: u32 = 500;
pub const ILLEGAL_PORT: u32 = 500;
pub const AUTHENTICATION_FAILED: u32 = 530;
pub const NO_ACCESS: u32 = 550;


#[derive(Debug, Copy, Clone)]
pub enum FtpMode {
    Active(SocketAddrV4),
    Passive,
}

pub fn write_response(client: &mut BufReader<TcpStream>, cmd: &str) {
    client.get_mut()
        .write(cmd.to_string().as_bytes())
        .expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");
}

pub fn read_message(client: &mut BufReader<TcpStream>) -> String {
    let mut response = String::new();
    client.read_line(&mut response).expect("Could not read message");
    println!("CLIENT: {}", response);

    return response;

}

pub fn handle_user(mut client: &mut BufReader<TcpStream>,
                   arg: &str,
                   map: &HashMap<String, User>)
                   -> bool {

    match map.get(arg) {
        Some(user) => {
            write_response(client,
                           &format!("{} Username okay, need password for {}\r\n",
                                    PASSWORD_EXPECTED,
                                    arg));
            let response = read_message(&mut client);

            let line = response.trim();

            let (cmd, password) = match line.find(' ') {
                Some(pos) => (&line[0..pos], &line[pos + 1..]),
                None => (line, "".as_ref()),
            };

            match cmd {
                "PASS" | "pass" => {
                    if password.trim() == user.pass {
                        write_response(client,
                                       &format!("{} Success Login for {}\r\n", LOGGED_IN, arg));
                        return true;
                    } else {

                        write_response(client,
                                       &format!("{} Invalid Password {}\r\n",
                                                INVALID_USER_OR_PASS,
                                                arg));
                        return false;
                    }
                }
                _ => {
                    write_response(client,
                                   &format!("{} {} not understood\r\n", NOT_UNDERSTOOD, cmd));
                    return false;
                }
            }

        }
        None => {

            info!("The user does not exist");
            write_response(client,
                           &format!("{} Invalid Username {}\r\n", INVALID_USER_OR_PASS, arg));
            return false;
        }
    }
}

//TODO: fixing here after implementing ls command
pub fn cwd(client: &mut BufReader<TcpStream>, args: &str, user: &mut User) {
    println!("user path: {}", user.path);
    println!("cur path: {}", user.cur_dir);

    let cur_dir = format!("{}", user.cur_dir).to_string();
    let arg_dir = format!("{}/{}", user.cur_dir, args).to_string();
    let user_dir = format!("{}", user.path);

    let mut temp_path = Path::new(&cur_dir);
    let user_root = Path::new(&user_dir);

    if args == ".." {
        temp_path = temp_path.parent().unwrap();
    } else if args == "." {
        //seriously? no sleep is affecting me
        temp_path = Path::new(&cur_dir);
    } else {
        temp_path = Path::new(&arg_dir);
    }

    let new_path = temp_path;
    println!("temp path: {}", temp_path.display());

    match new_path < user_root {
        true => {
            println!("new path is less then root");
            write_response(client,
                           &format!("{} CWD Command Success \r\n", CWD_CONFIRMED));
        }
        false => {
            if new_path.exists() && new_path.is_dir() {
                println!("New path exists");
                user.cur_dir = new_path.display().to_string();
                write_response(client,
                               &format!("{} CWD Command Success \r\n", CWD_CONFIRMED));
            } else {
                println!("New path doesn't  exists");
                write_response(client,
                               &format!("{} {} No Such File or Directory \r\n", NO_ACCESS, args));
            }
        }
    }

    println!("new cur path: {}", user.cur_dir);

}

//TODO: implement logic for user permissions here
pub fn cdup(client: &mut BufReader<TcpStream>, user: &mut User) {
    println!("user path: {}", user.path);
    println!("cur path: {}", user.cur_dir);

    //REFRACTOR: PutUser directory and path in one line
    let user_dir = format!("{}", user.path);
    let user_root = Path::new(&user_dir);

    let cur_dir = format!("{}", user.cur_dir);
    let cur_path = Path::new(&cur_dir).parent().expect("No parent exists");


    match cur_path < user_root {
        true => {
            write_response(client,
                           &format!("{} CWD Command Success \r\n", CWD_CONFIRMED));
        }
        false => {
            user.cur_dir = cur_path.display().to_string();
            write_response(client,
                           &format!("{} CDUP Command Success \r\n", CWD_CONFIRMED));
        }
    }

    println!("NEW cur path: {}", user.cur_dir);


}



//TODO: Role check in main function instead of here
pub fn mkd(client: &mut BufReader<TcpStream>, args: &str, user: &mut User) {

    let new_dir = format!("{}/{}", user.cur_dir, args);
    let path = Path::new(&new_dir);

    if !path.exists() {
        fs::create_dir_all(&path).expect("Could not create directory");
    }

    write_response(client,
                   &format!("{} {} creation success\r\n", PATHNAME_AVAILABLE, args));

}

//TODO: Role check in main function instead of here
//REFRACTOR: Consider turning type into an ENUM
pub fn handle_type(client: &mut BufReader<TcpStream>, args: &str) -> String {
    match args {
        "i" | "I" => {
            write_response(client, &format!("{} Type set to I\r\n", OPERATION_SUCCESS));
            return "BINARY".to_string();
        }
        "a" | "A" => {

            write_response(client, &format!("{} Type set to A\r\n", OPERATION_SUCCESS));
            return "ASCII".to_string();
        }
        _ => return "".to_string(),
    }
}


//REFRACTOR: Redo this logic to for more succinct code
pub fn handle_mode(client: &mut BufReader<TcpStream>,
                   ftp_mode: FtpMode,
                   data_port: &i32,
                   args: &str) {

    match ftp_mode {
        FtpMode::Passive => {
            let ip = format!("{}", client.get_mut().local_addr().unwrap().ip()).replace(".", ",");
            let (port1, port2) = split_port(data_port.clone() as u16);

            write_response(client,
                           &format!("{} Entering Passive Mode ({},{},{}).\r\n",
                                    PASSIVE_MODE,
                                    ip,
                                    port1,
                                    port2));
        }

        FtpMode::Active(addr) => {
            match TcpStream::connect(addr) {
                Ok(stream) => {
                    write_response(client,
                                   &format!("{} Port command successful\r\n",
                                    OPERATION_SUCCESS,
                                    ));
                }
                Err(d) => {
                    info!("{}", d);
                    write_response(client,
                                   &format!("{} Illegal Port Command\r\n",
                                    ILLEGAL_PORT,
                                    ));
                }

            }
        }
        _ => {
            write_response(client,
                           &format!("{} Bad sequence of commands.\r\n", NOT_UNDERSTOOD));
        }
    }
}


//Handling list commmand
pub fn list(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            data_port: &i32,
            listener: &TcpListener) {

    //getting a head start here in order to prvent slow connection

    match mode {
        FtpMode::Passive => {

            let (stream, addr) = listener.accept().expect("Could not accept connection");
            write_response(client,
                           &format!("{} Openning ASCII mode data for file list\r\n",
                                    OPENNING_DATA_CONNECTION));

            let mut data_stream = stream;
            ftp_ls(&user, &mut data_stream, args, data_port);
            write_response(client,
                           &format!("{} Transfer Complete\r\n", CLOSING_DATA_CONNECTION));
            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }
        _ => println!("Mode not implemented"),
    }


}

//Utility operation to convert port in to two number per RFC
fn split_port(port: u16) -> (u16, u16) {
    let b1 = port / 256;
    let b2 = port % 256;
    (b1, b2)
}

//Refractor: Consider returning a result from here for global error handling
fn ftp_ls(user: &User, stream: &mut TcpStream, args: &str, port: &i32) {
    //HANDLE not a directory
    let mut cur_dir = String::new();

    if args.is_empty() {
        cur_dir = format!("{}", user.cur_dir);
    } else {
        cur_dir = format!("{}/{}", user.cur_dir, args);
    }

    let path = Path::new(&cur_dir);

    println!("cur_dir {}", path.display());
    let mut paths = fs::read_dir(path).expect("Could not read directory for listing {}");

    for path in paths {
        let path = path.unwrap().path();
        let shortpath = path.to_str().unwrap();
        let pos = shortpath.find("ftproot").unwrap(); //Possible improvement here(error checking)

        let meta = path.metadata().unwrap();
        let line = format!("{}\t{}B\t{}",
                           meta.permissions().mode(),
                           meta.len(),
                           &shortpath[pos + 7..]);

        stream.write_fmt(format_args!("{}\n", line));
    }

}

fn write_to_stream(file: &mut File, stream: &mut TcpStream) {
    let mut buf = vec![0; 1024];
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
    let mut buf = vec![0; 1024];
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

pub fn to_ftp_port(b1: u16, b2: u16) -> u16 {
    b1 * 256 + b2
}
