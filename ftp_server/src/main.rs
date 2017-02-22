extern crate argparse; //argument parsing such as -h -d etc..

use std::io::prelude::*; //the standard io functions that come with rust
use std::io::{BufReader, Error as IoError};
use std::thread::spawn; //For threads

use std::string::String;
use std::str::FromStr;
use std::net::{TcpStream, TcpListener, Ipv4Addr, Shutdown,  SocketAddrV4};

use std::env; //To collect arguements and variables
use std::process::exit; //Gracefully exiting
use std::iter::Iterator;
use std::collections::HashMap;

use argparse::{ArgumentParser, Print, Store, StoreOption, StoreTrue};

#[derive(Debug, Clone)]
struct Arguements {
    ftp_port:       String,
    passive:        bool,
    active:         bool,
    debug:          bool,
    verbose:        bool,
    data_port_range:Option<String>,
    run_test_file:  Option<String>,
    config_file:    Option<String>,
    log_file:       Option<String>,
}

//These are the defaults incase no arguements are provided
impl Arguements {
    fn new() -> Arguements {
        Arguements {
            ftp_port:       "5000".to_string(),
            passive:        false,
            active:         false,
            debug:          false,
            verbose:        false,
            data_port_range:None,
            run_test_file:  None,
            config_file:    None,
            log_file:       None,
        }
    }
}


fn main() {

    let mut arguements = Arguements::new();

    {

        let mut ap = ArgumentParser::new();
        ap.set_description("Pachev's FTP client");

        ap.refer(&mut arguements.ftp_port)
            .add_option(&["--port", "-p"],Store, "Server Port");

        ap.add_option(&["--info", "-i", "--list-commands"], Print(COMMANDS_HELP.to_string()), "List supported commands");
        ap.add_option(&["--version", "-v"], Print("v0.1.0".to_string()), "Prints version");


        ap.refer(&mut arguements.passive)
            .add_option(&["--pasive"], StoreTrue, "Use passive mode and 
                                listen on provided address for data transfers");
        ap.refer(&mut arguements.active)
            .add_option(&["--active"], StoreTrue, "Use active mode and 
                                listen on provided address for data transfers");

        ap.refer(&mut arguements.debug)
            .add_option(&["-D", "--debug"], StoreTrue, "Sets debug mode on");

        ap.refer(&mut arguements.verbose)
            .add_option(&["-V", "--verbose"], StoreTrue, "Sets verbose  mode on");

        ap.refer(&mut arguements.data_port_range)
            .add_option(&["--dpr"], StoreOption, "Sets a range of ports for data");

        ap.refer(&mut arguements.config_file)
            .add_option(&["-c", "--config"], StoreOption, "location of configuration file");

        ap.refer(&mut arguements.run_test_file)
            .add_option(&["-t", "--test-file"], StoreOption, "location of test file");


        ap.parse_args_or_exit();
    }
    
    let server = format!("127.0.1.1:{}",arguements.ftp_port);
    let listener = TcpListener::bind(server.as_str()).expect("Could not bind to socket");

    println!("Success staring server");
    for stream in listener.incoming() {
        spawn(|| {
            let mut stream = stream.expect("Could not create TCP Stream");
            handle_client(&mut stream);
        });
    }
    println!("Hello, world!");
}

fn handle_client(client: & mut TcpStream) {
    client.write(b"Hello World\r\n").unwrap();
    client.shutdown(Shutdown::Both).expect("couldn't close server");
}



const COMMANDS_HELP: &'static str =
"
Pachev Joseph - 5699044
Commands: 
        user - Sends the username
        pass - Send the password
        cwd - Changes working directory
        cdup - Changes to parent directory
        logout - Terminates session
        retr - Retrieves a file
        stor - Stores a file
        stou - Stores a file uniquely
        appe - Appends to a file
        type - Stes tranfer type to Active or Passive
        rnrf - Rename From
        rnto - Rename To
        abor - Aborts a transfer
        dele - Deletes a file
        rmd - Removes a directory
        mkd - Makes a directory
        pwd - Prints working directory
        list - Lists files
        noop - Does nothing
        help - Prints Help Menu
        size - Prints size of file
        nlist - Name list of diretory
        ";
