extern crate clap;

use clap::{App, Arg, SubCommand};

pub fn cli() {
    let mut app = App::new("theseus")
        .author("Richard Schneider <richard@schneiderbox.net>")
        .about("Quoridor AI engine")
        .subcommand(
            SubCommand::with_name("move")
                .about("Makes a move from a board state")
                .arg(
                    Arg::with_name("tqbn")
                        .help("Board in TQBN notation")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("api")
                .about("Runs HTTP API server")
//                .arg(
//                    Arg::with_name("port")
//                        .short("p")
//                        .long("port")
//                        .help("Listen on this port")
//                        .takes_value(true),
//                ),
        );

    let app_m = app.clone().get_matches();
    match app_m.subcommand() {
        ("move", Some(sub_m)) => {
            let tqbn = sub_m.value_of("tqbn").unwrap();
            eprintln!("input: {}", tqbn);
            let board = crate::Board::from_tqbn(tqbn);
            board.print();

            let mut log = String::new();
            let child = crate::ai::mcts(&board, &mut log);
            eprintln!("{}", log);
            let move_string = board.move_string_to(&child);
            eprintln!("output: {}", move_string);
            child.print();

            println!("{}", move_string);
        }
        ("api", Some(_sub_m)) => {
            crate::api();
        }
        _ => {
            app.print_help().unwrap();
            println!();
        }
    };

    /*
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: theseus [tqbn]");
        std::process::exit(1);
    }

    let tqbn = &args[1].to_string().to_ascii_lowercase();

    */
}
