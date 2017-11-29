//! FTP Server implemented in rust for CNT4713 Net Centric at
//! Florida International University

extern crate argparse; //argument parsing such as -h -d etc..
extern crate rand; // unique string names to handle collisions
extern crate ini; // configuration file parser

// External logging library for pretty logging
#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;

use ini::Ini;

use std::io::prelude::*; //the standard io functions that come with rust
use std::io::{Write, BufReader};
use std::io;
use std::thread::spawn; //For threads
use std::thread; //For threads

use std::string::String;
use std::net::{Ipv4Addr, TcpStream, TcpListener, Shutdown, SocketAddrV4};

use std::path::Path;
use std::fs;
use std::fs::OpenOptions;
use std::fs::File;
use std::process;

use std::env;
use std::iter::Iterator;
use std::collections::HashMap;

use argparse::{ArgumentParser, Print, Store, StoreTrue, StoreFalse};
use slog::DrainExt;



// Local Modules
mod server;
mod tests;
mod user;
mod main_commands;

use user::User;
use server::FtpMode;
use main_commands as mc;

#[derive(Debug, Clone)]
struct Settings {
    ftp_port: String,
    ftp_mode: String,
    service_port: String,
    ftp_root: String,
    users_path: String,
    welcome: String,
    passive: bool,
    debug: bool,
    verbose: bool,
    data_port_range: String,
    run_test_file: String,
    config_file: String,
    log_file: String,
    max_users: String,
    max_attempts: String,
}

//These are the defaults incase no arguements are provided
impl Settings {
    /// A struct that handles all the command line arguements
    ///
    /// # Otions supported supported
    /// - `-h` for hostname
    /// - `-p` for port
    /// - `--pasive` to enable passive mode
    ///
    /// # Remarks
    ///
    /// It was easier to use a tested crate rather than parsing argument myself
    fn new() -> Settings {
        Settings {
            ftp_port: "2115".to_string(),
            ftp_root: "ftproot".to_string(),
            ftp_mode: "PASSIVE".to_string(),
            service_port: "2185".to_string(),
            users_path: "conf/users.cfg".to_string(),
            welcome: "Welcome To Pachev's FTP".to_string(),
            passive: true,
            debug: false,
            verbose: false,
            data_port_range: "27500-27999".to_string(),
            run_test_file: "".to_string(),
            config_file: "".to_string(),
            log_file: "logs/fserver.log".to_string(),
            max_users: "200".to_string(),
            max_attempts: "3".to_string(),
        }
    }
}

fn main() {
    let mut settings = Settings::new();
    let conf = Ini::load_from_file("conf/fsys.cfg").unwrap();

    //Loading default setting from conf file
    load_defaults(&mut settings, &conf);

    /*This is due to borrowing issue I'm setting a default mode of true
     * but use the argparser to allow the user to set the transfer mode
     */

    let mut passive = true;

    {

        let mut ap = ArgumentParser::new();
        ap.set_description("Pachev's FTP client");

        ap.add_option(&["--info", "-i", "--list-commands"],
                      Print(COMMANDS_HELP.to_string()),
                      "List supported commands");
        ap.add_option(&["--version", "-v"],
                      Print("v0.1.0".to_string()),
                      "Prints version");

        ap.refer(&mut settings.ftp_port)
            .add_option(&["--port", "-p"], Store, "Server Port");

        ap.refer(&mut settings.passive)
            .add_option(&["--passive"],
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
        ap.refer(&mut settings.debug)
            .add_option(&["-D", "--debug"], StoreTrue, "Sets debug mode on");

        ap.refer(&mut settings.verbose)
            .add_option(&["-V", "--verbose"], StoreTrue, "Sets verbose  mode on");

        ap.refer(&mut settings.data_port_range)
            .add_option(&["--dpr"], Store, "Sets a range of ports for data");

        ap.refer(&mut settings.config_file)
            .add_option(&["-c", "--config"], Store, "location of configuration file");

        ap.refer(&mut settings.run_test_file)
            .add_option(&["-t", "--test-file"], Store, "location of test file");

        ap.refer(&mut settings.users_path)
            .add_option(&["-u", "--userdb"], Store, "location of User DB file");

        ap.parse_args_or_exit();
    }
    settings.passive = passive;


    //Creating the database of users
    let mut users: HashMap<String, user::User> = HashMap::new();
    users = get_user_list(&settings);

    let service_port = format!("{}", settings.service_port);
    let mut map = users.clone(); //Cloning users for service port usage
    let mut serv_settings = settings.clone();

    // # This is the service port for the FTP server
    // It will be in the background stoping everything and starting everything
    let thread = thread::spawn(move || {

        let server = format!("127.0.0.1:{}", service_port);
        let listener = TcpListener::bind(server.as_str()).expect("Could not bind to main port");
        let (stream, _) = listener.accept().expect("Could not connect to service prot");
        let mut serv_client = BufReader::new(stream);
        let mut logged_in = false;
        let mut started = true;
        server::write_response(&mut serv_client,
                               "Welcome to Sevice Port please login with admin user 
                               using 'USER [username] Followed by 'PASS [password]'\r\n");


        loop {

            let mut response = String::new();
            serv_client.read_line(&mut response).expect("Could not read stream message");

            let line = response.trim();

            let (cmd, args) = match line.find(' ') {
                Some(pos) => (&line[0..pos], &line[pos + 1..]),
                None => (line, "".as_ref()),
            };

            if logged_in {

                match cmd.to_lowercase().as_ref() {
                    "server_stop" => {
                        process::exit(1);
                    }
                    "server_start" => {
                        match started {
                            true => {
                                server::write_response(&mut serv_client,
                                                       "Server is already running \r\n")
                            }
                            false => {
                                start_server(&mut serv_settings, &map);
                                server::write_response(&mut serv_client, "Server has started\r\n");
                            }

                        }
                    }
                    "server_pause" => {
                        started = pause_server();
                    }
                    _ => {
                        println!("Bad Command");
                    }
                }
            } else {
                logged_in = server::handle_user(&mut serv_client, &args, &map);
            }
        }

    });

    start_server(&mut settings, &users);

    thread.join().expect("Could not join service thread");

}
fn pause_server() -> bool {
    false
}

fn start_server(settings: &mut Settings, users: &HashMap<String, user::User>) {
    use std::collections::HashSet;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    let mut threads = HashMap::new();

    let log_path = Path::new(&settings.log_file);
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .unwrap();

    let drain = slog_stream::stream(log_file, MyFormat).fuse();
    let logger = slog::Logger::root(drain, o!());
    slog_stdlog::set_logger(logger).unwrap();

    info!("Global file logger for FTP Server");

    //Sets FTP ROOT
    create_root(&settings);

    let server = format!("127.0.0.1:{}", settings.ftp_port);
    let listener = TcpListener::bind(server.as_str()).expect("Could not bind to main port");
    let data_port_range = get_data_ports(format!("{}", settings.data_port_range));

    let hash_set: HashSet<i32> = HashSet::new();
    let hash_set_done: HashSet<i32> = HashSet::new();

    let mut used_ports = Arc::new(Mutex::new(hash_set));
    let mut used_ports_done = Arc::new(Mutex::new(hash_set_done));

    let mut port_count = 0;


    println!("Welcome to Pachev's Famous Rusty FTP Server");


    for stream in listener.incoming() {
        port_count = 0;
        while used_ports.lock().unwrap().contains(&data_port_range[port_count]) && port_count < 200 {
            port_count += 1;
        }
        if port_count >= 200 {
            info!("Reached client threshold");
            continue;
        }
	println!("Handling client number {}", port_count);
        let data_port = data_port_range[port_count];
        let stream = stream.expect("Could not create TCP Stream");

        if (used_ports_done.lock().unwrap().contains(&data_port)) {
          let t: std::thread::JoinHandle<_> = threads.remove(&data_port).unwrap();
          let _ = t.join();
          used_ports_done.lock().unwrap().remove(&data_port);
          println!("Cleaned up the data from previous run...");
        }
        used_ports.lock().unwrap().insert(data_port);


        debug!("client {} has started and given data port {}",
               stream.peer_addr().unwrap().ip(),
               data_port);

        let mut map = users.clone();
        let settings = settings.clone();
        let mut used_ports_client_copy = used_ports.clone();
	let mut used_ports_done_client_copy = used_ports_done.clone();

        threads.insert(data_port, spawn(move || {
            let mut b_stream = BufReader::new(stream);
            handle_client(&mut b_stream, &data_port, &settings, &mut map);
            used_ports_client_copy.lock().unwrap().remove(&data_port);
	    used_ports_done_client_copy.lock().unwrap().insert(data_port);
        }));
    }

    for (p, t) in threads {
        info!("Stopping all threads");
        let _ = t.join();
    }

}

/// # handle_client
///
/// This is the main function that handles the client thread
///
/// # Arguments
///
/// - client
/// - data_port
/// - map
fn handle_client(mut client: &mut BufReader<TcpStream>,
                 data_port: &i32,
                 settings: &Settings,
                 map: &HashMap<String, user::User>) {

    let data_server = format!("{}:{}",
                              client.get_mut().local_addr().unwrap().ip(),
                              data_port);

    let mut actv_socket_addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 27598);

    let data_listener = TcpListener::bind(data_server.as_str()).expect("Could not open data serve");

    let mut ftp_mode = match settings.passive {
        true => {
            info!("Running in passive mode");
            FtpMode::Passive
        }
        false => {
            info!("Running in active mode");
            FtpMode::Active(actv_socket_addr)
        }
    };

    let mut logged_in = false;
    let mut limit = settings.max_attempts.parse::<i32>().unwrap_or(3);
    let mut user = User::new();

    let msg = format!("{} {} {}\r\n",
                      server::LOGGED_EXPECTED,
                      settings.welcome,
                      client.get_mut().local_addr().unwrap().ip());


    server::write_response(&mut client, &msg);

    loop {

        let mut response = String::new();
        client.read_line(&mut response).expect("Could not read stream message");

        let line = response.trim();

        let (cmd, args) = match line.find(' ') {
            Some(pos) => (&line[0..pos], &line[pos + 1..]),
            None => (line, "".as_ref()),
        };

        println!("CLIENT: {} {}", cmd, args);
        info!("CLIENT: {} {}", cmd, args);


        if logged_in {
            match cmd.to_lowercase().as_ref() {
                "appe" => {
                    mc::stor(&mut client, &user, ftp_mode, &args, &data_listener);

                }
                "cdup" => {
                    server::cdup(&mut client, &mut user);
                }

                "cwd" | "cd" => {
                    server::cwd(&mut client, &args, &mut user);
                }
                "dele" => {
                    mc::dele(&mut client, &user, &args);
                }
                "list" => {
                    mc::list(&mut client,
                             &user,
                             ftp_mode,
                             &args,
                             &data_port,
                             &data_listener);
                }
                "mkd" | "mkdir" => {
                    server::mkd(&mut client, &args, &mut user);
                }
                "noop" => {
                    server::write_response(&mut client,
                                           &format!("{} NOOP successfull\r\n",
                                                    server::OPERATION_SUCCESS));
                }
                "pasv" => {
                    ftp_mode = FtpMode::Passive;
                    server::handle_mode(&mut client, ftp_mode, &data_port);

                }
                "port" => {
                    actv_socket_addr = port_addr(args);
                    ftp_mode = FtpMode::Active(actv_socket_addr);

                    server::handle_mode(&mut client, ftp_mode, &data_port);
                }
                "pwd" => {
                    let shortpath = format!("{}", user.cur_dir);
                    let pos = shortpath.find("ftproot").unwrap(); //Possible improvement here(error checking)
                    server::write_response(&mut client,
                                           &format!("{} {} is the current directory\r\n",
                                                    server::PATHNAME_AVAILABLE,
                                                    &shortpath[pos + 7..]));

                }
                "retr" => {
                    mc::retr(&mut client, &user, ftp_mode, &args, &data_listener);
                }
                "rmd" => {
                    mc::rmd(&mut client, &user, &args);
                }
                "rnfr" => {
                    mc::rnfr(&mut client, &user, &args);
                }
                "stor" => {
                    mc::stor(&mut client, &user, ftp_mode, &args, &data_listener);
                }
                "stou" => {
                    mc::stou(&mut client, &user, ftp_mode, &args, &data_listener);
                }
                "type" => {
                    server::handle_type(&mut client, &args);
                }
                "quit" | "exit" | "logout" => {
                    server::write_response(&mut client,
                                           &format!("{} GOODBYE\r\n", server::GOODBYE));
                    break;
                }
                "syst" => {
                    server::write_response(&mut client,
                                           &format!("{} UNIX Type: L8\r\n",
                                                    server::SYSTEM_RECEIVED));
                }
                "help" | "?" => {
                    write!(client.get_mut(), "{}\r\n", COMMANDS_HELP)
                        .expect("Could not write to client");
                }
                "user" => {
                    server::write_response(client,
                                           &format!("{} Badd sequence of commands\r\n",
                                                    server::NOT_UNDERSTOOD));

                }
                _ => server::write_response(&mut client, &format!("500 Invalid Command\r\n")),
            }

        } else {

            match cmd.to_lowercase().as_ref() {
                "user" => {
                    match server::handle_user(&mut client, &args, &map) {
                        true => {
                            logged_in = true;
                            user = map.get(args).unwrap().clone();
                        }
                        false => {
                            logged_in = false;
                            limit -= 1;
                            if limit <= 0 {
                                info!("{} reached logged limit", args);
                                break;
                            }
                        }
                    }
                }
                _ => {
                    server::write_response(&mut client,
                                           &format!("{} Not Logged In\r\n",
                                                    server::AUTHENTICATION_FAILED))
                }

            }
        }

    }

    client.get_mut().shutdown(Shutdown::Both).expect("couldn't close server");
    println!("Client {} has closed connection", data_port - 27500);
    info!("Client {} has closed connection", data_port - 27500);
}


/// # Users are initialized here
///
/// # Arguements
/// - name
/// - pass
/// - role
///
/// # example
/// ```
/// initialize_user("pachev", "dummy", "admin");
/// ```
fn initialize_user(name: &str, pass: &str, role: &str, root: &str) -> User {
    //Figuring out the current dirrectory
    let cur_directory = match env::current_dir() {
        Ok(pwd) => format!("{}", pwd.display()).to_string(),
        //Assigns to tmp if it doesn't exist
        Err(_) => format!("/tmp/").to_string(),

    };

    info!("Initializing users from user database");

    let mut user = User::new();

    //The starting path of the user will be determined by the user role
    match role {

        "admin" => {
            info!("This user {} is an admin", name);
            let admin_path = format!("{}/ftproot", cur_directory);
            user.path = format!("{}", admin_path).to_string();

        }
        _ => {
            info!("This user {} is regular", name);
            let user_path = format!("{}/{}/{}", cur_directory, root, name);
            user.path = user_path;
        }

    }

    let temp = format!("{}/ftproot/{}", cur_directory, name);
    let user_path = Path::new(&temp);

    if !user_path.exists() {
        fs::create_dir_all(&temp).expect("Could not create user directory");
    }
    user.name = format!("{}", name).to_string();
    user.pass = format!("{}", pass).to_string();
    user.role = format!("{}", role).to_string();
    user.cur_dir = format!("{}", user.path).to_string();

    return user;
}


//takes the command line argument in the form of 1-5 and returns array of ports
fn get_data_ports(ports: String) -> Vec<i32> {
    //Split the range in order to have an array of ports to issue
    let port_str_range: Vec<&str> = ports.trim().split('-').collect();
    let init_port: i32 = port_str_range[0].parse::<i32>().expect("could not parse ports");
    let last_port: i32 = port_str_range[1].parse::<i32>().expect("could not parse ports");

    let mut port_int_range: Vec<i32> = Vec::new();

    info!("Data ports: {} min {} max", init_port, last_port);

    for i in init_port..last_port + 1 {
        port_int_range.push(i);
    }

    return port_int_range;

}

fn get_user_list(settings: &Settings) -> HashMap<String, user::User> {

    let mut map: HashMap<String, user::User> = HashMap::new();

    let user_list = format!("{}", settings.users_path);
    let f = File::open(user_list).unwrap_or(File::open("conf/users.cfg").unwrap());
    let file = BufReader::new(f);
    // let mut users: Vec<&str> = Vec::new(); //May still user as alternative
    let mut user = user::User::new();

    for line in file.lines() {
        let line = line.expect("Could not read line");
        let things = match line.find('#') {
            Some(pos) => (line[0..pos].to_string()),
            None => line,
        };

        if things.is_empty() {
            continue;
        }

        let split = things.split(' ');
        let tokens: Vec<&str> = split.collect();
        user = initialize_user(&tokens[0].to_string(),
                               &tokens[1].to_string(),
                               &tokens[2].to_string(),
                               &settings.ftp_root);
        let name = tokens[0].to_string();
        info!("name: {}, role {}", name, tokens[2]);
        map.insert(name, user);
    }

    map

}

//Converts port command arguements into a socket address
fn port_addr(args: &str) -> SocketAddrV4 {
    let nums: Vec<u8> = args.split(',').map(|x| x.parse::<u8>().unwrap()).collect();
    let ip = Ipv4Addr::new(nums[0], nums[1], nums[2], nums[3]);
    let port = server::to_ftp_port(nums[4] as u16, nums[5] as u16);
    let addr = SocketAddrV4::new(ip, port);
    addr

}

//create ftproot folder if it does not exist
fn create_root(settings: &Settings) {
    let path = Path::new(&settings.ftp_root);

    if !path.exists() {
        fs::create_dir_all(path).expect("Root was not created");
    }
}

fn load_defaults(settings: &mut Settings, conf: &Ini) {

    info!("Loading defaults from Setting File");
    let defaults = conf.section(Some("default".to_owned())).unwrap();

    settings.ftp_port = format!("{}",
                                defaults.get("DATA_PORT_FTP_SERVER")
                                    .unwrap_or(&settings.ftp_port));

    settings.service_port = format!("{}",
                                    defaults.get("SERVICE_PORT")
                                        .unwrap_or(&settings.service_port));

    settings.ftp_root = format!("{}", defaults.get("FTP_ROOT").unwrap_or(&settings.ftp_root));
    settings.users_path = format!("{}",
                                  defaults.get("USER_DATA_FILE").unwrap_or(&settings.users_path));

    settings.welcome = format!("{}",
                               defaults.get("WELCOME_MSG").unwrap_or(&settings.welcome));

    settings.data_port_range = format!("{}-{}",
                                       defaults.get("DATA_PORT_RANGE_MIN")
                                           .unwrap_or(&"27500".to_string()),
                                       defaults.get("DATA_PORT_RANGE_MAX")
                                           .unwrap_or(&"2799".to_string()));

    settings.log_file = format!("{}", defaults.get("FTP_LOG").unwrap_or(&settings.log_file));
    settings.max_users = format!("{}",
                                 defaults.get("MAX_USERS").unwrap_or(&settings.max_users));
    settings.max_attempts = format!("{}",
                                    defaults.get("MAX_ATTEMPTS").unwrap_or(&settings.max_attempts));

    settings.ftp_mode = format!("{}",
                                defaults.get("FTP_MODE").unwrap_or(&"PASSIVE".to_string()));

    match settings.ftp_mode.to_lowercase().as_ref() {
        "passive" => {
            settings.passive = true;
        }
        _ => {
            settings.passive = false;
        }
    }

}




const COMMANDS_HELP: &'static str =
    "214-   \r\n
214-Pachev Joseph - 5699044 \r\n
214-FTP Server- V0.1.0
214-use --help for help on starting client\r\n
214-Commands: \r\n
214-        user - Sends the username\r\n
214-        pass - Send the password\r\n
214-        cwd - Changes working directory\r\n
214-        cdup - Changes to parent directory\r\n
214-        logout - Terminates session
214-        retr - Retrieves a file\r\n
214-        stor - Stores a file\r\n
214-        stou - Stores a file uniquely\r\n
214-        appe - Appends to a file\r\n
214-        type - Stes tranfer type to Active or Passive\r\n
214-        rnrf - Rename From\r\n
214-        rnto - Rename To\r\n
214-        abor - Aborts a transfer\r\n
214-        dele - Deletes a file\r\n
214-        rmd - Removes a directory\r\n
214-        mkd - Makes a directory\r\n
214-        pwd - Prints working directory\r\n
214-        list - Lists files\r\n
214-        noop - Does nothing\r\n
214-        help - Prints Help Menu\r\n
214-        size - Prints size of file\r\n
214-        nlist - Name list of direcotry\r\n
214 \r\n     
";

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
