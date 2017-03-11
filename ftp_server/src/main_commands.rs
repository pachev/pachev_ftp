use rand::Rng;
use rand;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::string::String;
use std::net::{TcpStream, TcpListener, Shutdown};
use std::path::Path;
use std::fs;
use std::fs::File;


use user::User;
use server::FtpMode;
use server;

/// # The FTP List command
/// This function implements the list command server side
///
/// # Arguements
///
/// - client
/// - user
/// - mode
/// - args
/// - data_port
/// - listener
pub fn list(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            data_port: &i32,
            listener: &TcpListener) {



    match mode {
        FtpMode::Passive => {

            info!("{} in passive mode requesting LIST command", user.name);
            //getting a head start here in order to prvent slow connection
            let (stream, _) = listener.accept().expect("Could not accept connection");
            server::write_response(client,
                                   &format!("{} Openning ASCII mode data for file list\r\n",
                                            server::OPENNING_DATA_CONNECTION));

            let mut data_stream = stream;
            server::ftp_ls(&user, &mut data_stream, args);
            server::write_response(client,
                                   &format!("{} Transfer Complete\r\n",
                                            server::CLOSING_DATA_CONNECTION));
            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }

        FtpMode::Active(addr) => {
            info!("{} in passive mode requesting LIST command", user.name);
            server::write_response(client,
                                   &format!("{} Openning ASCII mode data for file list\r\n",
                                            server::OPENNING_DATA_CONNECTION));
            let mut stream = TcpStream::connect(addr).expect("Could not connect to addr");


            server::ftp_ls(&user, &mut stream, args);
            server::write_response(client,
                                   &format!("{} Transfer Complete\r\n",
                                            server::CLOSING_DATA_CONNECTION));

        }
    }


}


pub fn stor(mut client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {


    match mode {
        FtpMode::Passive => {
            info!("{} in passive mode requesting STOR command", user.name);
            let (stream, _) = listener.accept().expect("Could not accept connection");
            let mut data_stream = stream;

            stor_file(&mut client, user, &mut data_stream, args);

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }

        FtpMode::Active(addr) => {

            info!("{} in active mode requesting STOR command", user.name);
            let mut data_stream = TcpStream::connect(addr).expect("Could not connect to addr");
            stor_file(&mut client, user, &mut data_stream, args);
            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }
    }

}

pub fn retr(mut client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {

    //getting a head start here in order to prvent slow connection

    match mode {
        FtpMode::Passive => {

            info!("{} in passive mode requesting RETR command", user.name);
            let (stream, _) = listener.accept().expect("Could not accept connection");
            let mut data_stream = stream;

            retr_file(&mut client, user, &mut data_stream, args);
            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }

        FtpMode::Active(addr) => {
            info!("{} in active mode requesting RETR command", user.name);
            let mut data_stream = TcpStream::connect(addr).expect("Could not connect to addr");
            retr_file(&mut client, user, &mut data_stream, args);
            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }
    }


}


pub fn stou(mut client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {

    //This is in case the file name is not unique
    let mut rng = rand::thread_rng();

    let full_path = format!("{}/{}", user.cur_dir, args);
    let s = rng.gen_ascii_chars().take(8).collect::<String>();

    let remote = Path::new(&full_path);

    match mode {

        FtpMode::Passive => {

            info!("{} in passive mode requesting STOU command", user.name);
            let (stream, _) = listener.accept().expect("Could not accept connection");

            let mut data_stream = stream;

            if remote.exists() {
                stor_file(&mut client, user, &mut data_stream, &s);
            } else {
                stor_file(&mut client, user, &mut data_stream, args);
            }

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stream");

        }

        FtpMode::Active(addr) => {

            info!("{} in active mode requesting STOU command", user.name);
            let mut data_stream = TcpStream::connect(addr).expect("Could not connect to addr");
            if remote.exists() {
                stor_file(&mut client, user, &mut data_stream, &s);
            } else {
                stor_file(&mut client, user, &mut data_stream, args);
            }

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stream");

        }
    }

}

pub fn appe(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {


    match mode {
        FtpMode::Passive => {

            //Waits for clinet to connect to data port
            let (stream, _) = listener.accept().expect("Could not accept connection");

            let mut data_stream = stream;
            let full_path = format!("{}/{}", user.cur_dir, args);
            let remote = Path::new(&full_path);


            if !remote.is_dir() {

                let mut file = match OpenOptions::new().append(true).open(remote) {
                    Ok(file) => file,
                    Err(_) => {
                        let file = File::create(remote)
                            .expect("Could not create remote file for append");
                        file
                    }
                };


                server::write_to_file(&mut file, &mut data_stream);

                //TODO: Add how long it took to transfer file
                server::write_response(client,
                                       &format!("{} Transfer Complete\r\n",
                                                server::CLOSING_DATA_CONNECTION));

            } else {
                server::write_response(client,
                                       &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
            }

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");




        }

        FtpMode::Active(_) => {
            println!("mode not yet implemented");

        }
    }

}

pub fn rnfr(mut client: &mut BufReader<TcpStream>, user: &User, args: &str) {
    let full_path = format!("{}/{}", user.cur_dir, args);
    let remote = Path::new(&full_path);

    if remote.exists() {
        server::write_response(client,
                               &format!("{} File or Directory Exists, Ready for Desitination\r\n",
                                        server::ITEM_EXISTS));

        //REFRACTOR: Consider adding a function that reads a message and parses cmd/args
        let response = server::read_message(&mut client);
        let line = response.trim();
        let (cmd, new_name) = match line.find(' ') {
            Some(pos) => (&line[0..pos], &line[pos + 1..]),
            None => (line, "".as_ref()),
        };

        match cmd.to_lowercase().as_ref() {
            "rnto" => {

                let from_path = format!("{}/{}", user.cur_dir, args);
                let to_path = format!("{}/{}", user.cur_dir, new_name);
                let from = Path::new(&from_path);
                let to = Path::new(&to_path);

                println!("Curr {}\nTo: {}", from_path, to_path);
                match fs::rename(from, to) {
                    Ok(_) => {
                        server::write_response(client,
                                               &format!("{} Success Renaming\r\n",
                                                        server::CWD_CONFIRMED));

                    }
                    Err(_) => {

                        server::write_response(client,
                                               &format!("{} Could Not Rename File\r\n",
                                                        server::BAD_SEQUENCE));
                    }

                }
            }
            _ => {
                server::write_response(client,
                                       &format!("{} {} Bad Sequence of Commands \r\n",
                                                server::BAD_SEQUENCE,
                                                cmd));
            }
        }

    } else {
        server::write_response(client,
                               &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
    }

}

pub fn dele(mut client: &mut BufReader<TcpStream>, user: &User, args: &str) {
    let full_path = format!("{}/{}", user.cur_dir, args);
    let remote = Path::new(&full_path);

    info!("{} being deleted form serve", args);
    if remote.exists() && !remote.is_dir() {
        match fs::remove_file(remote) {
            Ok(_) => {
                server::write_response(client,
                                       &format!("{} Success Deleting Filer\n",
                                                server::OPERATION_SUCCESS));
            }
            Err(_) => {

                server::write_response(client,
                                       &format!("{} File could not be deleted\r\n",
                                                server::NO_ACCESS));
            }
        }

    } else {
        server::write_response(client,
                               &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
    }
}


pub fn rmd(mut client: &mut BufReader<TcpStream>, user: &User, args: &str) {
    let full_path = format!("{}/{}", user.cur_dir, args);
    let mut remote = Path::new(&full_path);

    if remote.exists() && remote.is_dir() {
        match fs::remove_dir(remote) {
            Ok(_) => {
                server::write_response(client,
                                       &format!("{} Success Deleting Directory\r\n",
                                                server::CWD_CONFIRMED));
            }
            Err(_) => {

                server::write_response(client,
                                       &format!("{} Directory is not empty\r\n",
                                                server::NO_ACCESS));
            }
        }


    } else {
        server::write_response(client,
                               &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
    }
}


fn stor_file(client: &mut BufReader<TcpStream>, user: &User, stream: &mut TcpStream, args: &str) {

    server::write_response(client,
                           &format!("{} Opening binary mode to receive {}\r\n",
                                    server::OPENNING_DATA_CONNECTION,
                                    args));
    let mut data_stream = stream;
    let full_path = format!("{}/{}", user.cur_dir, args);

    let remote = Path::new(&full_path);

    if !remote.is_dir() {
        let mut file = File::create(remote).expect("Could not create file to store");
        server::write_to_file(&mut file, &mut data_stream);
        //TODO: Add how long it took to transfer file
        server::write_response(client,
                               &format!("{} Transfer Complete\r\n",
                                        server::CLOSING_DATA_CONNECTION));

    } else {
        server::write_response(client,
                               &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
    }
}

fn retr_file(client: &mut BufReader<TcpStream>, user: &User, stream: &mut TcpStream, args: &str) {

    server::write_response(client,
                           &format!("{} Openning binary mode to transfer {}\r\n",
                                    server::OPENNING_DATA_CONNECTION,
                                    args));

    let full_path = format!("{}/{}", user.cur_dir, args);
    println!("{} requested file", full_path);

    let mut data_stream = stream;
    let local = Path::new(&full_path);

    if !local.is_dir() && local.exists() {
        let mut file = File::open(local).expect("Could not create file to store");

        server::write_to_stream(&mut file, &mut data_stream);

        server::write_response(client,
                               &format!("{} Transfer Complete\r\n",
                                        server::CLOSING_DATA_CONNECTION));

    } else {
        server::write_response(client,
                               &format!("{} No Such File or Dir\r\n", server::NO_ACCESS));
    }
}
