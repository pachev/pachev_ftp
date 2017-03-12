//! FTP Client impolemented in rust
extern crate argparse; //argument parsing such as -h -d etc..
extern crate rpassword; //hidden passwords
extern crate ini;
extern crate rand;

//Reading from config files
use ini::Ini;

#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;

use std::io::prelude::*; //the standard io functions that come with rust
use std::process;
use std::path::Path;
use std::io::BufReader; //the standard io functions that come with rust
use std::net::{SocketAddrV4, Ipv4Addr, TcpStream};
use std::io;

use std::fs::OpenOptions;
use std::string::String;


use argparse::{ArgumentParser, Print, Store, StoreOption, StoreTrue, StoreFalse};

use slog::DrainExt;

//helper files for client functions
mod client;
mod utils;


use client::FtpMode;
use client::FtpType;


//This section here defines the arguements that the ftp_client will
//initally take when being called
#[derive(Debug, Clone)]
struct Arguements {
    hostname: String,
    ftp_port: String,
    ftp_mode: String,
    username: Option<String>,
    password: Option<String>,
    passive: bool,
    debug: bool,
    verbose: bool,
    data_port_range: String,
    run_test_file: String,
    config_file: String,
    run_default: bool,
    l_all: String,
    l_only: String,
    log_file: String,
}

//These are the defaults incase no arguements are provided
impl Arguements {
    fn new() -> Arguements {
        Arguements {
            hostname: "".to_string(),
            ftp_port: "21".to_string(),
            ftp_mode: "PASSIVE".to_string(),
            username: None,
            password: None,
            passive: true,
            debug: false,
            verbose: false,
            data_port_range: "".to_string(),
            run_test_file: "".to_string(),
            config_file: "".to_string(),
            run_default: false,
            l_all: "".to_string(),
            l_only: "logs/ftpclient.log".to_string(),
            log_file: "logs/ftpclient.log".to_string(),
        }
    }
}


fn main() {

    //Using argparse to make cmd line parsing manageable
    let mut arguements = Arguements::new();
    let conf = Ini::load_from_file("fclient.cfg").unwrap();

    //Loading default setting from conf file
    load_defaults(&mut arguements, &conf);

    //This is due to borrowing issue I'm setting a default mode of true
    //but use the argparser to allow the user to set the transfer mode
    let mut passive = true;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Pachev's FTP client");

        ap.refer(&mut arguements.hostname)
            .add_argument("hostname", Store, "Server hostname");

        ap.refer(&mut arguements.ftp_port)
            .add_argument("port", Store, "Server Port");

        ap.add_option(&["--info", "-i", "--list-commands"],
                      Print(utils::COMMANDS_HELP.to_string()),
                      "List supported commands");
        ap.add_option(&["--version", "-v"],
                      Print("v0.1.0".to_string()),
                      "Prints version");

        ap.refer(&mut arguements.username)
            .add_option(&["-u", "--user"], StoreOption, "Username");

        ap.refer(&mut arguements.password)
            .add_option(&["-w", "--pass"], StoreOption, "Password");

        ap.refer(&mut arguements.passive)
            .add_option(&["--pasive"],
                        StoreTrue,
                        "Use passive mode and 
                                listen on \
                         provided address for data transfers");
        ap.refer(&mut passive)
            .add_option(&["--active"],
                        StoreFalse,
                        "Use active mode and 
                                listen on provided \
                         address for data transfers");

        ap.refer(&mut arguements.debug)
            .add_option(&["-D", "--debug"], StoreTrue, "Sets debug mode on");

        ap.refer(&mut arguements.verbose)
            .add_option(&["-V", "--verbose"], StoreTrue, "Sets verbose  mode on");

        ap.refer(&mut arguements.data_port_range)
            .add_option(&["--dpr"], Store, "Sets a range of ports for data");

        ap.refer(&mut arguements.config_file)
            .add_option(&["-c", "--config"], Store, "location of configuration file");

        ap.refer(&mut arguements.run_test_file)
            .add_option(&["-t", "--test-file"], Store, "location of test file");

        ap.refer(&mut arguements.run_default)
            .add_option(&["-T"], StoreTrue, "Runs default test file");

        ap.refer(&mut arguements.l_all)
            .add_option(&["--LALL"], Store, "Location to store all log output");

        ap.refer(&mut arguements.l_only)
            .add_option(&["--LONLY"], Store, "Location to store all log output");

        ap.parse_args_or_exit();
    }
    arguements.passive = passive;

    //Uses either the parsed info or defaults to determiner server


    start_ftp_client(&mut arguements);
}

fn start_ftp_client(mut arguements: &mut Arguements) -> BufReader<TcpStream> {

    let temp_path = format!("{}", arguements.log_file);
    let log_path = Path::new(&temp_path);
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();
    let drain = slog_stream::stream(log_file, MyFormat).fuse();
    let logger = slog::Logger::root(drain, o!());
    slog_stdlog::set_logger(logger).unwrap();

    info!("Global File Logger for FTP CLIENT");
    //this will serve as a holder
    let mut myclient: TcpStream;

    /*
     * Here is the loop for starting the program
     * this handles the cases where no localhost is provided and
     * where an empty hostname is provided. The loop will continue
     * as long as we are not able to connec to a socket. The only command
     * available during the loop are open, quit, help
     */
    loop {


        //TODO: put the connection into a function to reduce repetition of code
        if !arguements.hostname.is_empty() {
            let server = format!("{}:{}", arguements.hostname, arguements.ftp_port);
            match TcpStream::connect(server.as_str()) {
                Ok(stream) => {
                    info!("Success Connecting to server {}",
                          stream.peer_addr().unwrap().ip());
                    arguements.hostname = "".to_string();
                    arguements.ftp_port = "".to_string();
                    myclient = stream;
                    let mut stream = BufReader::new(myclient);
                    println!("Success Connecting to server");
                    let response = client::read_message(&mut stream, arguements.verbose);
                    cmd_loop(&mut stream, &mut arguements);
                }
                Err(_) => {
                    arguements.hostname = "".to_string();
                    arguements.ftp_port = "".to_string();
                    println!("Could not connect to host");
                    debug!("Could not connect to host {}", arguements.hostname);
                }
            }
        } else {

            let (mut cmd, mut args) = get_commands();

            match cmd.to_lowercase().as_ref() {
                "open" | "ftp" => {
                    let (host, port) = match args.find(' ') {
                        Some(pos) => (&args[0..pos], &args[pos + 1..]),
                        None => (args.as_ref(), "21".as_ref()),
                    };

                    let server = format!("{}:{}", host, port);
                    match TcpStream::connect(server.as_str()) {
                        Ok(stream) => {
                            arguements.hostname = "".to_string();
                            arguements.ftp_port = "".to_string();
                            myclient = stream;
                            let mut stream = BufReader::new(myclient);
                            println!("Success Connecting to server");
                            let response = client::read_message(&mut stream, arguements.verbose);
                            cmd_loop(&mut stream, &mut arguements);
                        }
                        Err(_) => {
                            println!("Could not connect to host");
                            info!("Could not connect to host");
                        }

                    }
                }
                "!" | "bye" | "quit" | "exit" => {
                    println!("Goodbye");
                    process::exit(1);
                }
                "close" | "disconnect" => {
                    println!("Not Connected");
                }
                "debug" => {
                    toggle_debug(&mut arguements);
                }
                "verbose" => {
                    toggle_verbose(&mut arguements);
                }
                "help" | "?" | "usage" => utils::print_help(&args),
                _ => {
                    println!("Not Connected");
                }
            }
        }
    }

}


fn login(mut client: &mut BufReader<TcpStream>, arguements: &Arguements) -> bool {
    let mut logged_in: bool = false;
    let os_user = std::env::var("USER").unwrap_or(String::new());

    let user = match arguements.username {
        Some(ref usr) => usr.to_string(),
        None => {
            print!("User ({}) ", os_user);
            io::stdout().flush().expect("Something went wrong flushing");
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Err(_) => "".to_string(),
                Ok(_) => {
                    match line.trim().is_empty() {
                        true => os_user.to_string(),
                        false => line.trim().to_string(),
                    }
                }
            }
        }
    };

    //hidden passwords removed for turning in assignment. Necessary for test file to work
    let password = match arguements.password {
        Some(ref pass) => pass.to_string(),
        None => {
            print!("Password:");
            io::stdout().flush().expect("Something went wrong flushing");
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Err(_) => "".to_string(),
                Ok(_) => {
                    match line.trim().is_empty() {
                        true => "".to_string(),
                        false => line.trim().to_string(),
                    }
                }
            }
        }

    };
    let mut line = String::new();
    let mut cmd = format!("USER {}\r\n", user);
    let mut response = String::new();

    client::write_command(&mut client, &cmd, arguements.debug);
    response = client::read_message(&mut client, arguements.verbose);

    response.clear();
    cmd = format!("PASS {}\r\n", password);

    client::write_command(&mut client, &cmd, arguements.debug);
    response = client::read_message(&mut client, arguements.verbose);

    match client::get_code_from_respone(&response) {
        Ok(230) => {
            println!("Success Logging In");
            info!("Success Logging In {}", user);
            logged_in = true;
        }
        Ok(_) => {
            println!("Login Failed");
            info!("Error Logging In {}", user);
            logged_in = false;
        }
        Err(e) => {
            println!("Something went wrong");
            debug!("Something went wrong logging user");
        } 
    }

    logged_in

}



fn cmd_loop(mut client: &mut BufReader<TcpStream>, mut arguements: &mut Arguements) {

    let mut actv_socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 27598);
    let mut ftp_type = FtpType::Binary;

    let mut ftp_mode = match arguements.passive {
        true => {
            info!("Running in passive mode");
            FtpMode::Passive
        }
        false => {
            info!("Running in active mode");
            FtpMode::Active(actv_socket_addr)
        }
    };
    let mut logged_in = login(&mut client, &arguements);
    let auth_mesg = "You need to be logged in";
    let mut runique = false;
    let mut sunique = false;

    loop {
        let (cmd, args) = get_commands();
        let (debug, verbose) = (arguements.debug, arguements.verbose);
        if logged_in {
            match cmd.to_lowercase().as_ref() {
                "appe" | "append" => client::appe(&mut client, &args, ftp_mode, debug, verbose),
                "ascii" => {
                    ftp_type = FtpType::ASCII;
                    println!("Type set to A- Ascii");
                    info!("Type set to A- Ascii");
                }
                "binary" | "image" => {
                    ftp_type = FtpType::Binary;
                    println!("Type set Binary");
                    info!("Type set Binary");
                }
                "close" | "disconnect" => {
                    println!("Closing connection");
                    info!("Closing connection");
                    break;
                }
                "cd" | "cwd" | "dir" => client::change_dir(&mut client, &args, debug, verbose),
                "cdup" | "cdu" => client::change_dir_up(&mut client, debug, verbose),
                "dele" | "del" => client::dele(&mut client, &args, debug, verbose),
                "get" | "retr| recv" => {
                    match runique {
                        true => {
                            client::get_u(&mut client, &args, ftp_mode, ftp_type, debug, verbose)
                        }
                        false => {
                            client::get(&mut client, &args, ftp_mode, ftp_type, debug, verbose)
                        }
                    }
                }
                "ls" | "list" | "dir" => client::list(&mut client, &args, ftp_mode, debug, verbose),
                "lls" | "llist" | "ldir" => client::list_local(&args),
                "lpwd" => client::print_locoal_dir(),
                "lcd" | "lcwd" => client::change_local_dir(&args),
                "mkdir" | "mkd" => client::make_dir(&mut client, &args, debug, verbose),
                "mdele" | "mdel" => client::mdele(&mut client, &args, debug, verbose),
                "mlist" | "mls" | "mdir" => {
                    client::mlist(&mut client, &args, ftp_mode, debug, verbose)
                }
                "mget" | "mretr| mrecv" => {
                    client::mget(&mut client, &args, ftp_mode, ftp_type, debug, verbose)
                }
                "mput" | "mstor" => {
                    client::mput(&mut client, &args, ftp_mode, ftp_type, debug, verbose)
                }
                "pwd" => client::print_working_dir(&mut client, debug, verbose),
                "put" | "stor" => {
                    client::put(&mut client,
                                &args,
                                ftp_mode,
                                ftp_type,
                                debug,
                                verbose,
                                sunique)
                }
                "rm" | "rmd" | "rmdir" => client::remove_dir(&mut client, &args, debug, verbose),
                "rstatus" => client::rstatus(&mut client, &args, debug, verbose),
                "reset" => continue,
                "rename" | "rename" => client::rename(&mut client, &args, debug, verbose),
                "rhelp" => client::r_help(&mut client, debug, verbose),
                "runique" => {
                    runique = !runique;
                    println!("Receive Unqiue= {}", runique);
                    info!("Receive Unqiue= {}", runique);
                }
                "sunique" => {
                    sunique = !sunique;
                    println!("Put Unqiue= {}", sunique);
                    info!("Put Unqiue= {}", sunique);
                }
                "status" => {
                    client::status(&mut client,
                                   debug,
                                   verbose,
                                   ftp_type,
                                   ftp_mode,
                                   runique,
                                   sunique)
                }
                "system" => client::system(&mut client, &args, debug, verbose),
                "size" => client::size(&mut client, &args, debug, verbose),
                "type" => {
                    match ftp_type {
                        FtpType::Binary => {
                            println!("Using Binary Mode For Transfers");
                            info!("Using Binary Mode For Transfers");
                        }
                        FtpType::ASCII => {
                            println!("Using ASCII Mode for transfers");
                            info!("Using ASCII Mode for transfers");
                        }
                    }
                }
                "debug" => {
                    toggle_debug(&mut arguements);
                }
                "verbose" => {
                    toggle_verbose(&mut arguements);
                }
                "!" | "bye" | "quit" | "exit" => {
                    println!("Goodbye");
                    client::quit_server(&mut client, debug, verbose);
                    process::exit(1);
                }
                "help" | "?" | "usage" => utils::print_help(&args),
                "user" => println!("Already connected"),
                _ => {
                    println!("Invalid Command");
                }
            }

        } else {
            match cmd.to_lowercase().as_ref() { 
                "!" | "bye" | "quit" | "exit" => {
                    println!("Goodbye");
                    client::quit_server(&mut client, debug, verbose);
                    process::exit(1);
                }
                "help" | "?" | "usage" => utils::print_help(&args),
                "user" => logged_in = login(&mut client, &arguements),
                "open" | "ftp" => {
                    println!("Already connected, use close to end connection");
                }
                "debug" => {
                    toggle_debug(&mut arguements);
                }
                "verbose" => {
                    toggle_verbose(&mut arguements);
                }

                "close" | "disconnect" => {
                    println!("Closing connection");
                    break;
                }
                _ => {
                    println!("You need to be logged in for this command");
                }

            }
        }


    }

}

fn get_commands() -> (String, String) {

    print!("ftp> ");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let line = buf.trim();
    let (cmd, args) = match line.find(' ') {
        Some(pos) => (&line[0..pos], &line[pos + 1..]),
        None => (line, "".as_ref()),
    };

    let s1 = format!("{}", cmd);
    let s2 = format!("{}", args);
    debug!("Retrieving commands {} {}", cmd, args);
    (s1, s2)
}

fn load_defaults(settings: &mut Arguements, conf: &Ini) {
    info!("Loading default settings");
    let defaults = conf.section(Some("default".to_owned())).unwrap();

    settings.ftp_port = format!("{}",
                                defaults.get("default_ftp_port")
                                    .unwrap_or(&settings.ftp_port));
    settings.data_port_range = format!("{}-{}",
                                       defaults.get("data_port_min")
                                           .unwrap_or(&"27500".to_string()),
                                       defaults.get("data_port_max")
                                           .unwrap_or(&"2799".to_string()));

    settings.log_file = format!("{}",
                                defaults.get("default_log_file").unwrap_or(&settings.log_file));
    settings.ftp_mode = format!("{}",
                                defaults.get("default_mode").unwrap_or(&"PASSIVE".to_string()));
    match settings.ftp_mode.to_lowercase().as_ref() {
        "passive" => {
            settings.passive = true;
        }
        _ => {
            settings.passive = false;
        }
    }
    let debug = format!("{}",
                        defaults.get("default_debug_mode").unwrap_or(&"true".to_string()));

    let verbose = format!("{}",
                          defaults.get("default_verbose_mode").unwrap_or(&"true".to_string()));

    settings.debug = debug.parse::<bool>().unwrap_or(true);
    settings.debug = verbose.parse::<bool>().unwrap_or(false);

}

fn toggle_debug(settings: &mut Arguements) {
    match settings.debug {
        true => {
            settings.debug = false;
            println!("Debugging off (Debug=0)");
            info!("Debugging off (Debug=0)");
        }
        false => {
            settings.debug = true;
            println!("Debugging on (Debug=1)");
            info!("Debugging on (Debug=1)");
        }
    }
}

fn toggle_verbose(settings: &mut Arguements) {
    match settings.verbose {
        true => {
            settings.verbose = false;
            println!("Verbose off (Verbose=0)");
            info!("Verbose off (Verbose=0)");
        }
        false => {
            settings.verbose = true;
            println!("Verbose on (Verbose=1)");
            info!("Verbose on (Verbose=1)");
        }
    }
}


struct MyFormat;

impl slog_stream::Format for MyFormat {
    fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("{} - {}\n", rinfo.level(), rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}
