use std::fs::File;
use rand::Rng;
use rand;
use std::os::unix::fs::PermissionsExt;
use std::env;
use std::thread; //For threads
use std::fs;
use std::path::Path;
use std::error::Error;
use std::io;
use std::time::Duration;
use std::io::Write;
use std::io::prelude::*;
use std::io::{BufReader, Error as IoError};
use std::net::{TcpStream, TcpListener, Ipv4Addr, Shutdown, SocketAddrV4};

use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;


#[derive(Debug, Copy, Clone)]
pub enum FtpMode {
    Active(SocketAddrV4),
    Passive,
}

#[derive(Debug, Copy, Clone)]
pub enum FtpType {
    Binary,
    ASCII,
}




//Writes commands to the server
//
pub fn write_command(client: &mut BufReader<TcpStream>, cmd: &str, debug: bool) {
    client.get_mut()
        .write(cmd.to_string().as_bytes())
        .expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");
    info!("----> {}", cmd);
    if debug {

        println!("----> {}", cmd);
    }

}

//reads the response back
pub fn read_message(client: &mut BufReader<TcpStream>, verbose: bool) -> String {
    let mut response = String::new();
    client.read_line(&mut response).expect("Could not read message");
    info!("SERVER: {}", response);

    if verbose {
        println!("SERVER: {}", response);
    }

    return response;

}

//reads multi line message
pub fn read_multi_message(client: &mut BufReader<TcpStream>, verbose: bool) -> String {
    let mut response = "end of transmission".to_string();

    client.get_mut().set_read_timeout(Some(Duration::from_millis(500))).expect("Could set timeout");

    for line in client.lines() {
        match line {
            Ok(res) => {
                println!("Server {}", res);
            }
            Err(_) => {
                break;
            }
        }
    }
    return response;

}

//This is used for verifying message received fromt the sever
pub fn get_code_from_respone(line: &str) -> Result<i32, &'static str> {

    let number = match line[0..3].parse::<i32>() {
        Ok(code) => code,
        Err(_) => -1,
    };

    println!("code is: {}", number);
    info!("code from server is: {}", number);
    Ok(number)
}

pub fn make_dir(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("MKD {}\r\n", args);
    let mut response = String::new();
    info!("Sending MKD command");

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

pub fn change_dir(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("CWD {}\r\n", args);
    let mut response = String::new();
    info!("Sending CWD command");

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

pub fn change_dir_up(mut stream: &mut BufReader<TcpStream>, debug: bool, verbose: bool) {
    let mut cmd = "CDUP\r\n".to_string();
    let mut response = String::new();

    info!("Sending CUP command");
    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}


pub fn change_local_dir(mut stream: &mut BufReader<TcpStream>, args: &str) {
    let l_cur_dir = env::current_dir().unwrap();
    println!("cur path: {}", l_cur_dir.display());

    let cur_dir = format!("{}", l_cur_dir.display()).to_string();
    let arg_dir = format!("{}/{}", l_cur_dir.display(), args).to_string();

    let mut temp_path = Path::new(&cur_dir);

    if args == ".." {
        temp_path = temp_path.parent().unwrap();
    } else if args == "." {
        temp_path = Path::new(&l_cur_dir);
    } else {
        temp_path = Path::new(&arg_dir);
    }

    //Similar to try catch
    if env::set_current_dir(&temp_path).is_ok() {
        println!("new cur path: {}", &temp_path.display());
        info!("new cur path: {}", &temp_path.display());
    } else {

        println!("Error changing local directory");
        info!("Error changing local directory");
    }


}

//List local directory
pub fn list_local(mut stream: &mut BufReader<TcpStream>, args: &str) {

    let l_cur_dir = env::current_dir().unwrap();

    let mut cur_dir = format!("{}", l_cur_dir.display());

    if !args.is_empty() {
        cur_dir = format!("{}/{}", l_cur_dir.display(), args);
    }

    let path = Path::new(&cur_dir);

    println!("cur_dir {}", path.display());
    info!("cur_dir {}", path.display());
    let mut paths = fs::read_dir(path).expect("Could not read directory for listing {}");

    for path in paths {
        let path = path.unwrap().path();
        let meta = path.metadata().unwrap();
        let line = format!("{}\t{}B\t{}",
                           meta.permissions().mode(),
                           meta.len(),
                           path.display());

        println!("{}\r\n", line);
    }

    println!("List sucessful");
    info!("List sucessful");
}

//Print local
pub fn print_locoal_dir(mut stream: &mut BufReader<TcpStream>) {

    let l_cur_dir = env::current_dir().expect("Something went wrong obtaining local directory");

    println!("local: {}", l_cur_dir.display());
    info!("printing local directory : {}", l_cur_dir.display());
}

//Remove a directory

pub fn remove_dir(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("RMD {}\r\n", args);
    let mut response = String::new();
    info!("SENDING CMD command");

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

//Rhelp
pub fn r_help(mut stream: &mut BufReader<TcpStream>, debug: bool, verbose: bool) {
    let mut cmd = "HELP\r\n".to_string();
    let mut response = String::new();

    info!("SENDING help command");
    write_command(&mut stream, &cmd, debug);
    read_multi_message(&mut stream, verbose);
}

//Delete  a File

pub fn dele(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    info!("SENDING DELE command");
    let mut cmd = format!("DELE {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

//Print working dir

pub fn print_working_dir(mut stream: &mut BufReader<TcpStream>, debug: bool, verbose: bool) {
    let mut cmd = "PWD\r\n".to_string();
    let mut response = String::new();
    info!("SENDING PWD command");

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

//QUIT
pub fn quit_server(mut stream: &mut BufReader<TcpStream>, debug: bool, verbose: bool) {
    let mut cmd = "QUIT\r\n".to_string();
    let mut response = String::new();
    info!("EXITING CLIENT");
    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
}

//Put a file
pub fn put(mut stream: &mut BufReader<TcpStream>,
           args: &str,
           ftp_mode: FtpMode,
           ftp_type: FtpType,
           debug: bool,
           verbose: bool,
           sunique: bool) {

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

    let mut response = String::new();
    set_type(&mut stream, ftp_type, debug);
    response = read_message(&mut stream, verbose);
    response.clear();

    match ftp_mode {
        FtpMode::Passive => {

            info!("Seding {} in passive mode to be stored as {} ",
                  lpath,
                  rpath);
            write_command(&mut stream, "PASV \r\n", debug);
            response = read_message(&mut stream, verbose);
            let addr = get_pasv_address(&response);
            match sunique {
                true => write_command(&mut stream, &format!("STOR {} \r\n", rpath), debug),
                false => write_command(&mut stream, &format!("STOU {} \r\n", rpath), debug),
            }


            stor_file(&addr, &lpath, &mut stream, debug);

            response.clear();
            response = read_message(&mut stream, verbose);

        }
        FtpMode::Active(addr) => {

            info!("Seding {} in active mode to be stored as {} ", lpath, rpath);
        }
    }



}

//Get a file
pub fn get(mut stream: &mut BufReader<TcpStream>,
           args: &str,
           ftp_mode: FtpMode,
           ftp_type: FtpType,
           debug: bool,
           verbose: bool) {
    let mut response = String::new();
    let mut lpath = String::new();
    let mut rpath = String::new();

    //TODO: replce tilde with home dir
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

    set_type(&mut stream, ftp_type, debug);
    response = read_message(&mut stream, verbose);
    response.clear();

    match ftp_mode {
        FtpMode::Passive => {
            info!("Retrieving {} in passive mode to be stored as {} ",
                  rpath,
                  lpath);
            write_command(&mut stream, "PASV\r\n", debug);
            response = read_message(&mut stream, verbose);

            let addr = get_pasv_address(&response);
            write_command(&mut stream, &format!("RETR {}\r\n", rpath), debug);
            get_file(&addr, &lpath, &mut stream, verbose);
            response.clear();
            response = read_message(&mut stream, verbose);
        }
        FtpMode::Active(addr) => {

            info!("Retrieving {} in active mode to be stored as {} ",
                  rpath,
                  lpath);
        }
    }



}

//List Command
pub fn list(mut stream: &mut BufReader<TcpStream>,
            args: &str,
            ftp_mode: FtpMode,
            debug: bool,
            verbose: bool) {

    let mut response = String::new();
    set_type(&mut stream, FtpType::ASCII, debug);

    response = read_message(&mut stream, verbose);
    response.clear();

    match ftp_mode {
        FtpMode::Passive => {
            info!("Retrieving LIST command in passive mode");

            write_command(&mut stream, "PASV \r\n", debug);
            response = read_message(&mut stream, verbose);
            let addr = get_pasv_address(&response);
            write_command(&mut stream, &format!("LIST {}\r\n", args), debug);
            println!("args: {}", args);

            list_file(&addr, args, &mut stream, verbose);
            response.clear();
            response = read_message(&mut stream, verbose);

        }
        FtpMode::Active(addr) => {

            info!("Retrieving LIST command in passive mode");
        }
    }

}

//mdele for deleting multiple files on the server
pub fn mdele(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let arg_list: Vec<&str> = args.split(' ').collect();

    info!("Deleting multiple files {}", args);
    for file in arg_list {
        let mut cmd = format!("DELE {}\r\n", file);
        let mut response = String::new();
        write_command(&mut stream, &cmd, debug);
        response = read_message(&mut stream, verbose);
        response.clear();
    }


}

//mget for retrieving multiple files at once
pub fn mget(mut stream: &mut BufReader<TcpStream>,
            args: &str,
            ftp_mode: FtpMode,
            ftp_type: FtpType,
            debug: bool,
            verbose: bool) {
    let mut threads = vec![];

    let arg_list: Vec<&str> = args.split(' ').collect();
    let mut response = String::new();

    set_type(&mut stream, ftp_type, debug);

    //Creating a clone of the stream that will be protected in a mutex
    let mut temp_stream = stream.get_mut().try_clone().expect("Could not clone stream");
    let mut shared_stream = Arc::new(Mutex::new(BufReader::new(temp_stream)));

    response = read_message(&mut stream, verbose);
    response.clear();
    info!("retrieving multiple files {}", args);

    for file in arg_list {
        let arg = format!("{}", file);
        let mut_stream = shared_stream.clone();

        let mut response = String::new();
        let t_debug = debug.clone();
        let t_verbose = verbose.clone();

        let thread = thread::spawn(move || match ftp_mode {
            FtpMode::Passive => {

                let mut buf_stream = mut_stream.lock().expect("could not lock main streamm");
                write_command(&mut buf_stream, "PASV\r\n", t_debug);
                response = read_message(&mut buf_stream, t_verbose);
                let addr = get_pasv_address(&response);
                write_command(&mut buf_stream, &format!("RETR {}\r\n", arg), t_debug);
                get_file(&addr, &arg, &mut buf_stream, t_verbose);
                response.clear();
                response = read_message(&mut buf_stream, t_verbose);
            }
            FtpMode::Active(addr) => {}
        });

        threads.push(thread);
    }

    for t in threads {
        info!("Joining all threads");
        let _ = t.join().unwrap();
    }



}

//mput for storing multiple files
pub fn mput(mut stream: &mut BufReader<TcpStream>,
            args: &str,
            ftp_mode: FtpMode,
            ftp_type: FtpType,
            debug: bool,
            verbose: bool) {
    let mut threads = vec![];

    let arg_list: Vec<&str> = args.split(' ').collect();
    let mut response = String::new();

    info!("storing multiple files {}", args);
    set_type(&mut stream, ftp_type, debug);

    //Creating a clone of the stream that will be protected in a mutex
    let mut temp_stream = stream.get_mut().try_clone().expect("Could not clone stream");

    //Creating a new BufReader of the main stream protected inside a mutex
    let mut shared_stream = Arc::new(Mutex::new(BufReader::new(temp_stream)));

    response = read_message(&mut stream, verbose);
    response.clear();

    for file in arg_list {
        let arg = format!("{}", file);
        //cloning the mutex inside of each thread
        let mut_stream = shared_stream.clone();

        let mut response = String::new();
        let t_debug = debug.clone();
        let t_verbose = verbose.clone();

        let thread = thread::spawn(move || match ftp_mode {
            FtpMode::Passive => {

                let mut buf_stream = mut_stream.lock().expect("could not lock main streamm");
                write_command(&mut buf_stream, "PASV\r\n", t_debug);
                response = read_message(&mut buf_stream, t_verbose);
                let addr = get_pasv_address(&response);
                write_command(&mut buf_stream, &format!("STOR {}\r\n", arg), t_debug);
                stor_file(&addr, &arg, &mut buf_stream, t_debug);
                response.clear();
                response = read_message(&mut buf_stream, t_verbose);
            }
            FtpMode::Active(addr) => {}
        });

        threads.push(thread);
    }

    //Joining threads before end of function
    for t in threads {
        info!("Joining all threads");
        let _ = t.join().unwrap();
    }



}

//mlist Command for listing multiple directories
pub fn mlist(mut stream: &mut BufReader<TcpStream>,
             args: &str,
             ftp_mode: FtpMode,
             debug: bool,
             verbose: bool) {

    let mut arg_list: Vec<&str> = args.split(' ').collect();
    let save_to = arg_list.pop().expect("nothing in the vector");
    println!("Would you like to save to {}?", save_to);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let ans = buf.trim();

    match ans.to_lowercase().as_ref() {
        "y" => {
            let mut local_file = File::create(&save_to).expect("Could not save to local file");

            info!("Saving MLIST of {} to {}", args, save_to);
            for file in arg_list {
                let mut response = String::new();
                set_type(&mut stream, FtpType::ASCII, debug);

                response = read_message(&mut stream, verbose);
                response.clear();

                match ftp_mode {
                    FtpMode::Passive => {

                        write_command(&mut stream, "PASV \r\n", debug);
                        response = read_message(&mut stream, verbose);
                        let addr = get_pasv_address(&response);
                        write_command(&mut stream, &format!("LIST {}\r\n", file), debug);
                        println!("args: {}", file);

                        let mut stream2 = TcpStream::connect(addr)
                            .expect("could not read connect address");

                        response = read_message(&mut stream, verbose);
                        let mut buf: Vec<u8> = Vec::new();
                        stream2.read_to_end(&mut buf).expect("Could not read second stream");
                        let text = (String::from_utf8(buf))
                            .expect("Could not read text from streamm");
                        stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
                        write!(local_file, "{}", text);
                        response.clear();
                        response = read_message(&mut stream, verbose);

                    }
                    FtpMode::Active(addr) => {}
                }
            }
        }
        _ => {
            return;
        }
    }


}

pub fn rstatus(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("STAT {}\r\n", args);
    info!("Sending STAT command to server");
    let mut response = String::new();

    write_command(&mut stream, &cmd, debug);
    response = read_multi_message(&mut stream, verbose);
}


pub fn appe(mut stream: &mut BufReader<TcpStream>,
            args: &str,
            ftp_mode: FtpMode,
            debug: bool,
            verbose: bool) {

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

    let mut response = String::new();
    set_type(&mut stream, FtpType::ASCII, debug);
    response = read_message(&mut stream, verbose);
    response.clear();

    match ftp_mode {
        FtpMode::Passive => {

            info!("Appending to file {} in passive mode", args);
            write_command(&mut stream, "PASV \r\n", debug);
            response = read_message(&mut stream, verbose);
            let addr = get_pasv_address(&response);
            write_command(&mut stream, &format!("APPE {} \r\n", rpath), debug);
            stor_file(&addr, &lpath, &mut stream, verbose);

            response.clear();
            response = read_message(&mut stream, verbose);
        }

        FtpMode::Active(addr) => {

            info!("Appending to file {} in passive mode", args);
        }
    }
}


pub fn get_u(mut stream: &mut BufReader<TcpStream>,
             args: &str,
             ftp_mode: FtpMode,
             ftp_type: FtpType,
             debug: bool,
             verbose: bool) {

    //This is in case the file name is not unique
    let mut rng = rand::thread_rng();
    let s = rng.gen_ascii_chars().take(8).collect::<String>();
    let mut response = String::new();
    let mut lpath = String::new();
    let mut rpath = String::new();

    match args.find(' ') {
        Some(pos) => {
            rpath = args[0..pos].to_string();
            lpath = args[pos + 1..].to_string();
        }
        None => {
            rpath = args.to_string();
            lpath = rpath.clone();
        }
    }

    let mut local = Path::new(&lpath);

    match ftp_mode {

        FtpMode::Passive => {

            write_command(&mut stream, "PASV \r\n", debug);
            response = read_message(&mut stream, verbose);

            let addr = get_pasv_address(&response);

            write_command(&mut stream, &format!("RETR {}\r\n", rpath), debug);

            if local.exists() {
                println!("Local file exits, replacing with {}", s);
                info!("Local file exits, replacing with {}", s);
                get_file(&addr, &s, &mut stream, verbose);
            } else {
                info!("Storing file {}", rpath);
                get_file(&addr, &lpath, &mut stream, verbose);
            }
            response.clear();
            response = read_message(&mut stream, verbose);

        }

        FtpMode::Active(addr) => {}
    }
}

//Retrieves the size of a file

pub fn size(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("SIZE {}\r\n", args);
    info!("Sending SIZE command to server");
    let mut response = String::new();
    set_type(&mut stream, FtpType::Binary, debug);
    response = read_message(&mut stream, verbose);
    response.clear();

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
    let pos = response.find(' ').unwrap();
    println!("{} b", &response[pos + 1..].trim());
}

// Status of local staus
pub fn status(mut stream: &mut BufReader<TcpStream>,
              debug: bool,
              verbose: bool,
              ftp_type: FtpType,
              ftp_mode: FtpMode,
              sunique: bool,
              runique: bool) {

    let mode = match ftp_mode {
        FtpMode::Passive => "Passive Mode",
        FtpMode::Active(_) => "Active Mode",
    };

    let t_type = match ftp_type {
        FtpType::Binary => "Binary mode on for transfers",
        FtpType::ASCII => "ASCII mode is on for transfers",
    };

    let con_to = stream.get_mut().peer_addr().unwrap();

    println!("Connected to {}", con_to);
    println!("Mode is set  to {}", mode);
    println!("Transfer Type is set  to {}", t_type);
    println!("Debug is set  to {}", debug);
    println!("Verbose is set  to {}", verbose);
}


// System call of remote
pub fn system(mut stream: &mut BufReader<TcpStream>, args: &str, debug: bool, verbose: bool) {
    let mut cmd = format!("SYST {}\r\n", args);
    let mut response = String::new();

    write_command(&mut stream, &cmd, debug);
    response = read_message(&mut stream, verbose);
    println!("{}", response);
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

fn set_type(mut stream: &mut BufReader<TcpStream>, ftp_type: FtpType, debug: bool) {
    match ftp_type {
        FtpType::Binary => {
            let mut cmd = "Type I\r\n".to_string();
            //Set transfer mode to binary
            write_command(&mut stream, &cmd, debug);

        }
        FtpType::ASCII => {
            let mut cmd = "Type A\r\n".to_string();
            //Set transfer mode to Ascii
            write_command(&mut stream, &cmd, debug);

        }
    }
}

fn get_pasv_address(response: &str) -> SocketAddrV4 {
    let start_pos = response.rfind('(').expect("Could not read response from server") + 1;
    let end_pos = response.rfind(')').expect("could not read response form server");
    let substr = response[start_pos..end_pos].to_string();
    let nums: Vec<u8> = substr.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
    let ip = Ipv4Addr::new(nums[0], nums[1], nums[2], nums[3]);
    let port = to_ftp_port(nums[4] as u16, nums[5] as u16);
    let addr = SocketAddrV4::new(ip, port);
    addr

}

fn stor_file(addr: &SocketAddrV4,
             lpath: &str,
             mut stream: &mut BufReader<TcpStream>,
             verbose: bool) {

    //TODO Spawn a therad here
    let mut stream2 = TcpStream::connect(addr).expect("could not read connect address");
    let response = read_message(&mut stream, verbose);

    let mut file = match File::open(lpath) {
        Ok(file) => file,
        Err(_) => {
            println!("Error opening file on local");
            stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
            return;
        }
    };
    write_to_stream(&mut file, &mut stream2);
    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
}


fn get_file(addr: &SocketAddrV4,
            rpath: &str,
            mut stream: &mut BufReader<TcpStream>,
            verbose: bool) {

    //TODO Spawn a therad here
    let mut stream2 = TcpStream::connect(addr).expect("could not read connect address");
    let response = read_message(&mut stream, verbose);

    let mut file = match File::create(rpath) {
        Ok(file) => file,
        Err(_) => {
            println!("Error opening file on local");
            stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
            return;
        }
    };
    write_to_file(&mut file, &mut stream2);
    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
}

fn list_file(addr: &SocketAddrV4,
             rpath: &str,
             mut stream: &mut BufReader<TcpStream>,
             verbose: bool) {

    //TODO Spawn a therad here
    let mut stream2 = TcpStream::connect(addr).expect("could not read connect address");
    let response = read_message(&mut stream, verbose);

    let mut buf: Vec<u8> = Vec::new();
    stream2.read_to_end(&mut buf).expect("Could not read second stream");
    let text = (String::from_utf8(buf)).expect("Could not read text from streamm");
    stream2.shutdown(Shutdown::Both).expect("Failed to close data stream");
    println!("{}", text);
}
