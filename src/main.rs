extern crate theseus;

fn invalid_tqbn() {
    eprintln!("invalid tqbn");
    std::process::exit(2);
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: theseus [tqbn]");
        std::process::exit(1);
    }
    let tqbn: Vec<_> = args[1].chars().collect();
    if tqbn.len() != 73 {
        invalid_tqbn();
    }
}
