pub const COMMANDS_HELP: &'static str =
    "
Pachev Joseph - 5699044
!		dir		mdir		quit		size
append		disconnect	mget		recv		status
ascii		epsv4		mkdir		rstatus		system
binary		get		mls		rhelp		sunique
bye		help		mput		rename		type
cd		image		nlist		reset		user
cdup		lcd		open		restart		verbose
close		lpwd		passive		rmdir		?
delete		ls		put		runique	
debug		mdelete		pwd		send	
        ";

pub fn print_help(args: &str) {

    match args {
        "!" | "bye" | "quit" | "exit" => println!("bye - closes application"),
        "append" => println!("append[local] [remote] - Appends a file to exising file in remote"),
        "ascii" => println!("ascii- Sets transfer mode to ascii"),
        "binary" | "image" => println!("binary- Sets transfer mode to binary"),
        "cd" | "dir" => println!("cd [path]- Changes current remote directory"),
        "cdup" => println!("cdup - Changes current remote directory one directory up"),
        "close" | "disconnect" => println!("close - Closes current connection"),
        "dele" | "del" => println!("dele [file]- Deletes a file on remote connection"),
        "debug" => println!("debug- Toggles debug mode"),
        "get" | "recv" => println!("get[remote] [local] - retrieves a remote file to local path"),
        "verbose" => println!("debug- Toggles verbose mode"),
        "help" => println!("help [command]- Shows help for command or prints commands if empty"),
        "lcd" | "ldir" => println!("lcd [path]- Changes current local directory"),
        "lpwd" => println!("lpwd- Prints local current working directory"),
        "ls" | "list" => println!("ls [path]- List remote directory"),
        "lls" | "llist" => println!("lls [path]- List current local directory"),
        "mkdir" | "mkd" => println!("mkdir [path]- creates a remote directory"),
        "mdele" | "mdel" => {
            println!("dele [file1] [file2]...- Deletes multiple files on remote connection")
        }
        "mls" => {
            println!("mls [dir] [dir]...[file]- lists multiple directoriess on remote connection \
                      to a local file")
        }
        "mget" | "mrecv" => {
            println!("mget [file1] [file2]...- retrieves multiple files on remote connection")
        }
        "mput" | "msend" => {
            println!("mput [file1] [file2]...- sends multiple files on remote connection")
        }
        "put" | "send" => println!("get[local] [remote] - endss a local file to remote path"),
        "nls" | "nlist" => println!("nlist [path]- List simple names on remote connection"),
        "open" | "ftp" => println!("open [host] [port]- opens a remote connection"),
        "passive" => println!("passive- Sets transfer mode to passive"),
        "pwd" => println!("pwd- Prints remote current working directory"),
        "size" => println!("size [file]- Prints size of remote file"),
        "rhelp" => println!("rhelp- Retrieves remote server help file"),
        "reset" => println!("reset- Resets current connection"),
        "rstatus" => println!("rstatus- Retrieves remote server status"),
        "rmdir" | "rmd" => println!("rmdir [path]- deletes a remote directory"),
        "runique" => println!("runique- Toggles receive unique to not overwrite existing files"),
        "sunique" => println!("runique- Toggles store unique to not overwrite existing files"),
        "status" => println!("status- prints local status"),
        "system" => println!("system- prints remote system type"),
        "" => println!("{}", COMMANDS_HELP),
        _ => println!("This command is not supported"),
    }
}


//Utility operation to convert port in to two number per RFC
pub fn split_port(port: u16) -> (u16, u16) {
    let b1 = port / 256;
    let b2 = port % 256;
    (b1, b2)
}
