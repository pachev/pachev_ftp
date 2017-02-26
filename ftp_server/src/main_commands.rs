use rand::Rng;
use rand;
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
use server::FtpMode;
use server;

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
            server::write_response(client,
                                   &format!("{} Openning ASCII mode data for file list\r\n",
                                            server::OPENNING_DATA_CONNECTION));
            let mut stream = TcpStream::connect(addr).expect("Could not connect to addr");

            server::ftp_ls(&user, &mut stream, args);
            server::write_response(client,
                                   &format!("{} Transfer Complete\r\n",
                                            server::CLOSING_DATA_CONNECTION));

        }
        _ => println!("Mode not implemented"),
    }


}


pub fn stor(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {


    match mode {
        FtpMode::Passive => {
            let (stream, addr) = listener.accept().expect("Could not accept connection");
            server::write_response(client,
                                   &format!("{} Openning binary mode to receive{}\r\n",
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

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }

        FtpMode::Active(addr) => {
            println!("mode not yet implemented");

        }
        _ => println!("Mode not implemented"),
    }

}

pub fn retr(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {

    //getting a head start here in order to prvent slow connection

    match mode {
        FtpMode::Passive => {

            let (stream, addr) = listener.accept().expect("Could not accept connection");
            server::write_response(client,
                                   &format!("{} Openning binary mode to transfer {}\r\n",
                                            server::OPENNING_DATA_CONNECTION,
                                            args));
            let mut data_stream = stream;
            let full_path = format!("{}/{}", user.cur_dir, args);
            println!("{} requested file", full_path);

            let local = Path::new(&full_path);

            if !local.is_dir() && local.exists() {
                let mut file = File::open(local).expect("Could not create file to store");

                server::write_to_stream(&mut file, &mut data_stream);

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

        FtpMode::Active(addr) => {
            println!("mode not yet implemented");

        }
        _ => println!("Mode not implemented"),
    }


}


pub fn stou(client: &mut BufReader<TcpStream>,
            user: &User,
            mode: FtpMode,
            args: &str,
            listener: &TcpListener) {


    match mode {
        FtpMode::Passive => {
            let (stream, addr) = listener.accept().expect("Could not accept connection");

            let mut rng = rand::thread_rng();
            let mut data_stream = stream;
            let full_path = format!("{}/{}", user.cur_dir, args);

            let s = rng.gen_ascii_chars().take(8).collect::<String>();

            let unique_path = format!("{}/{}", user.cur_dir, s);

            let mut remote = Path::new(&full_path);

            if remote.exists() {
                remote = Path::new(&unique_path);
            }

            server::write_response(client,
                                   &format!("{} Openning binary mode to receive {}\r\n",
                                            server::OPENNING_DATA_CONNECTION,
                                            s));

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

            data_stream.shutdown(Shutdown::Both).expect("Could not shutdownd data stram");

        }

        FtpMode::Active(addr) => {
            println!("mode not yet implemented");

        }
        _ => println!("Mode not implemented"),
    }

}
