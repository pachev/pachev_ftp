extern crate argparse; //argument parsing such as -h -d etc..
extern crate rpassword; //hidden passwords

use std::io::prelude::*; //the standard io functions that come with rust
use std::net::{TcpListener, TcpStream}; //TcP stream and listeners
use std::thread::spawn; //For threads

use std::string::String;
use std::str::FromStr;
use std::net::ToSocketAddrs;

use std::env; //To collect arguements and variables
use std::process::exit; //Gracefully exiting
use std::iter::Iterator;
use std::collections::HashMap;

use argparse::{ArgumentParser, Print, Store, StoreOption, StoreTrue};
use rpassword::{read_password};


//This section here defines the arguements that the ftp_client will
//initally take when being called
#[derive(Debug, Clone)]
struct Arguements {
    hostname:       String,
    ftp_port:       String,
    username:       Option<String>,
    password:       Option<String>,
    passive:        bool,
    active:         bool,
    debug:          bool,
    verbose:        bool,
    data_port_range:Option<String>,
    run_test_file:  Option<String>,
    config_file:    Option<String>,
    run_default:    bool,
    l_all:          Option<String>,
    l_only:         Option<String>,
}

//These are the defaults incase no arguements are provided
impl Arguements {
    fn new() -> Arguements {
        Arguements {
            hostname:       "cnt4713.cs.fiu.edu".to_string(),
            ftp_port:       "21".to_string(),
            username:       None,
            password:       None,
            passive:        false,
            active:         false,
            debug:          false,
            verbose:        false,
            data_port_range:None,
            run_test_file:  None,
            config_file:    None,
            run_default:    false,
            l_all:          None,
            l_only:         None,
        }
    }
}


fn main() {
    
    let mut arguements = Arguements::new();
    
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Pachev's FTP client");

        ap.refer(&mut arguements.hostname)
            .add_option(&["--host", "-h"],Store, "Server hostname");

        ap.refer(&mut arguements.ftp_port)
            .add_option(&["--port", "-p"],Store, "Server Port");

        ap.add_option(&["--info", "-i", "--list-commands"], Print(COMMANDS_HELP.to_string()), "List supported commands");
        ap.add_option(&["--version", "-v"], Print("v0.1.0".to_string()), "Prints version");

        ap.refer(&mut arguements.username)
            .add_option(&["-u", "--user"], StoreOption, "Username");

        ap.refer(&mut arguements.password)
            .add_option(&["-w", "--pass"], StoreOption, "Password");

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

        ap.refer(&mut arguements.run_default)
            .add_option(&["-T"], StoreTrue, "Runs default test file");

        ap.refer(&mut arguements.l_all)
            .add_option(&["--LALL"], StoreOption, "Location to store all log output");

        ap.refer(&mut arguements.l_only)
            .add_option(&["--LONLY"], StoreOption, "Location to store all log output");

        ap.parse_args_or_exit();
    }
}


const COMMANDS_HELP: &'static str =
"
Pachev Joseph - 5699044
Commands: 
        login - User login command
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
