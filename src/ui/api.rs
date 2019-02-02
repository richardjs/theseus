use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};

struct AllowOrigin {
    origin: String,
}

impl AllowOrigin {
    fn new(origin: String) -> AllowOrigin {
        AllowOrigin { origin }
    }
}

impl Fairing for AllowOrigin {
    fn info(&self) -> Info {
        Info {
            name: "Allow origin",
            kind: Kind::Response,
        }
    }

    fn on_response(&self, _request: &Request, response: &mut Response) {
        response.set_raw_header("Access-Control-Allow-Origin", self.origin.clone());
    }
}

#[get("/?<id>&<tqbn>")]
fn index(id: u64, tqbn: String) -> String {
    let _id = id;

    let mut log = String::new();

    log.push_str(&format!("input: {}\\n", tqbn));

    let board = crate::Board::from_tqbn(&tqbn.to_string());
    log.push_str(&board.to_string().replace("\n", "\\n"));

    let child = crate::ai::minimax(&board);
    log.push_str(&child.to_string().replace("\n", "\\n"));

    let move_string = board.move_string_to(&child);
    log.push_str(&format!("output: {}\\n", move_string));

    String::from(format!(
        "
    {{
	\"move\": \"{}\", 
	\"log\": \"{}\"
    }}",
        move_string, log
    ))
}

pub fn api() {
    rocket::ignite()
        .attach(AllowOrigin::new(String::from("*")))
        .mount("/theseus", routes![index])
        .launch();
}
