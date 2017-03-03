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


//Utility operation to convert port in to two number per RFC
pub fn split_port(port: u16) -> (u16, u16) {
    let b1 = port / 256;
    let b2 = port % 256;
    (b1, b2)
}
