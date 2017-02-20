extern crate argparse; //argument parsing such as -h -d etc..
extern crate rpassword; //hidden passwords

use std::io::prelude::*; //the standard io functions that come with rust
use std::io::BufReader; //the standard io functions that come with rust
use std::net::TcpStream;
use std::net::SocketAddrV4;
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
use rpassword::prompt_password_stdout;

//helper file for client functions
mod client;
use client::{FtpMode, Client};
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
    
    //Using argparse to make cmd line parsing manageable
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

    //Uses either the parsed info or defaults to determiner server
    let mut server = format!("{}:{}", arguements.hostname, arguements.ftp_port);
    let mut myclient = TcpStream::connect(server.as_str()).expect("Error Connecting to server");
    let mut stream = BufReader::new(myclient);
    let mut line = String::new();
    println!("Success Connecting to server");
    stream.read_line(&mut line).expect("Something wwent wrong reading from server");

    login(&mut stream, &arguements);
    cmd_loop(&mut stream);
}

fn login(mut client: &mut BufReader<TcpStream>, arguements: &Arguements) {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut logged_in: bool = false;
    let os_user = std::env::var("USER").unwrap_or(String::new());

    while !logged_in {
        let user = match arguements.username {
            Some(ref usr) => usr.to_string(),
            None => {
                print!("User ({}) ", os_user);
                stdout.flush().expect("Something went wrong flushing");
                let mut line = String::new();
                match stdin.read_line(&mut line) {
                    Err(_) => return,
                    Ok(_) => {
                        match line.trim().is_empty() {
                            true => os_user.to_string(),
                            false=> line.trim().to_string()
                        }
                    }
                }
            }
        };

        let password = match arguements.password {
            Some(ref pass) => pass.to_string(),
            None => {
                match prompt_password_stdout("Password: ") {
                    Ok(pwd) => pwd.to_string(),
                    Err(_) => return,
                }
            }
        };
        let mut line = String::new();
        let mut cmd = format!("USER {}\n", user);
        let mut response = String::new();

        Client::write_command(&mut client, &cmd);
        response = Client::read_message(&mut client);


        response.clear();
        cmd = format!("PASS {}\n", password);

        Client::write_command(&mut client, &cmd);
        response = Client::read_message(&mut client);
        
        match Client::get_code_from_respone(&response) {
            Ok(230) => {
                println!("Success Logging In");
                logged_in = true;
            },
            Ok(n) => {
                println!("Uncessfull, try again");
                continue;
            },
            Err(e) => break
        }


        

    }

}



fn cmd_loop (mut client: &mut BufReader<TcpStream>) {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    'looper: loop {
        print!("ftp>");
        stdout.flush().unwrap();

        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        let line = buf.trim();

        let (cmd,args) = match line.find(' ') {
            Some(pos) =>(&line[0..pos], &line[pos+1..]),
            None => (line, "".as_ref())
        };

        match cmd {
            "ls" | "list" => Client::list(&mut client, &args),
            "mkdir" | "mkd" => Client::make_dir(&mut client, &args),
            "cd" | "cwd" => Client::change_dir(&mut client, &args),
            "dele" | "del" => Client::dele(&mut client, &args),
            "cdup" | "cdu" => Client::change_dir_up(&mut client),
            "pwd" => Client::print_working_dir(&mut client),
            "put" | "stor" => Client::put(&mut client, args),
            "get"| "retr" => Client::get(&mut client, args),
            "rm" | "rmd" => Client::remove_dir(&mut client, &args),
            "quit" | "exit" => { 
                println!("Goodbye");
                Client::quit_server(&mut client);
                break 'looper;
            },
            "help"=> println!("{}", COMMANDS_HELP),
            _ => {
                println!("Invalid Command");
            }
        }

    }


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
