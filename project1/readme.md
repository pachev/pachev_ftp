# FTP Server and Client In Rust

This is an implementation of both an FTP server and client in Rust. It follows the [RFC 959][1] as best as possible.


* [Installation of Rust](#installation)
* [Compiling](#compiling)
* [Setting Up Configurations](#setting-up-configurations)
* [Usage](#usage)
* [A Case For Extra Credit](#a-case-for-extra-credit)

## Installation

Follow the instructions at [www.rustup.rs][2] to install all the dependencies needed.

`rustup` installs `rustc`, `cargo`, `rustup` and other standard tools
to Cargo's `bin` directory. On Unix it is located at
`$HOME/.cargo/bin` and on Windows at `%USERPROFILE%\.cargo\bin`. This
is the same directory that `cargo install` will install Rust programs
and Cargo plugins.

This directory will be in your `$PATH` environment variable, which
means you can run them from the shell without further
configuration. Open a *new* shell and type the following:

```
rustc --version
```

If you see something like `rustc 1.7.0 (a5d1e7a59 2016-02-29)` then
you are ready to Rust. 


## Compiling

The two projects are separated into two different folders. Both will need to be compiled individually

Since cargo is already installed you can simply run:

```
cd ftp_client && cargo build --release
cd ../ftp_server && cargo build --release
```

The compiled binaries will then be placed under  `[project_folder]/target/release`.

Optionally, you can run `cargo test` to run unit tests for each application


## Setting Up Configurations

### FTP Client

For a successful run, the ftp client needs several starter files/directories in the directory that the binary is executed in. 
```
*ftp_client
│   fclient.cfg
│
└───tests
│   │   sunny.txt
│   │   sunny2.txt
│   │   rainy.txt
│   │   rainy2.txt
│   
└───logs
    │   ftpclient.log
        
```
fclient.cfg has the following structure

```
#this is a comment

[default]
data_port_max = 27500
data_port_min = 27999
default_ftp_port = 21
default_mode = Passive 
default_debug_mode  = true
default_verbose_mode = false
default_test_file = test/test.txt
default_log_file = logs/ftpclient.log
```
A test has the following structure
  
```
open localhost 2115
classftp
micarock520
mkdir rustypachev
runique
sunique
debug
verbose
ls
cd rustypachev
ls
put test.txt dititwork.txt
cdup
rmdir rustypachev
ls
quit
```
 

### FTP Server

For a successful run, the ftp server needs several starter files/directories in the directory that the binary is executed in. 
```
*ftp_server
└───conf
│   │   fsys.cfg
│   │   users.cfg
│   │
│   
└───ftproot
│   └───user1
│   │       │   ...
│   │       │   ...
│   │       │   ...
│   └───user2
│   │       │   ...
│   │       │   ...
│   │       │   ...
│   ...
└───logs
    │   fserver.log
```

The fsys.cfg has the following structure
```
#This is a comment
[default]
FTP_ROOT = ftproot/
USER_DATA_FILE = conf/users.cfg 
#ftp_mode supports ACTIVE PASSIVE BOTH 
FTP_MODE = PASSIVE 
#this applies for PASSIVE ONLY 
DATA_PORT_RANGE_MIN = 27500
DATA_PORT_RANGE_MAX = 27999
DATA_PORT_FTP_SERVER = 2115 # 21 is the common port 
FTP_ENABLED = 1 
MAX_USER_SUPPORT = 200
WELCOME_MSG = "Welcome to FTP Server Spring 2017" 
FTP_LOG = logs/fserver.log
SERVICE_PORT = 2116
MAX_USERS = 200
MAX_ATTEMPTS = 3
#

```

The users.cfg file has the following structure

```
#this is the users file
# Starting anything with a hashtag makes it a comment
# users are specified as followe
#[name] [password] [role]

pachev  admin
user1 dummy user
user2 dummy blocked
francisco dummy notallowed
```
The rest of the files are created but the directories must be in the main folder

## Usage

As long as the [configurations](setting-up-configurations) were setup correctly, both the client and server will 
run with no arguments given. However adding a `--help` or `--info` will print out usage of the particular application.

### FTP Client

basic usage of the ftp client is as followed: `./ftp_client [host] [port] [options]`. An example of this would be 

`./ftp_client cnt4713.cs.fiu.edu -u classftp --pass secret -D -V`

This command above would login to cnt4713.cs.fiu.edu on default port 21 with the username classftp, password secret and 
turns on both Debug mode and Verbose mode. More options can be found by running `./ftp_client --help`.

### FTP Server

basic usage of the ftp client is as followed: `./ftp_server [options]`. An example of this would be 

`./ftp_server -u path/to/custom/users`

This command above would start the ftp server using a custom path to a group of users. For more options,
run the command `./ftp_server --help`. The ftp server also comes with a __service port__ that is used
to stop, start, and pause the application. This is a seperate port that is only available to admin users. 
This port can be setup in the configuartions of the project.. 

A folder named `project1` is already setup with the structure that the files need to be ran in.



## A Case For Extra Credit

In general, this project was due in Python, yet I decided to do it in a new programming language: Rust. I am the only
student in class who did not benefit from a starter file as everyone else was provided with a sample client and server
file written in python. Further, python as a more mature language has better support and libraries. 

Although I did this project in a different language, I still was able to help other students through the forum by 
providing general strategies for handling argument parsing and configuration file parsing in python. In addition, I 
online below the extra steps which I believe can earn me more era credit. 

### FTP Client

1. Client is multithread for sending and receiving files which was not a requirement
2. m-commands were implemented although not required
    1. mdele
    2. mput
    3. mget
    4. mlist
3. Test files work through Linux redirection: for instance `./ftp_client < test.txt | less` will work just fine

### FTP Server

1. Hidden password inputs 
2. The server supports both Active and Passive modes
3. unit tests


[1]: https://www.ietf.org/rfc/rfc959.txt
[2]: https://www.rustup.rs
