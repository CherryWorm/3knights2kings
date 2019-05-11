use rocket::fairing::AdHoc;
use rocket_contrib::templates::Template;
use crate::tablebase::{Tablebase, Evaluation};
use rocket_contrib::json::{JsonValue, Json};
use std::collections::HashMap;
use crate::state::Position;
use rocket::State;
use crate::tablebase::Value::MateIn;
use crate::state;
use serde::{Deserialize, Serialize};
use rocket::response::NamedFile;
use std::path::{PathBuf, Path};


#[derive(Serialize, Deserialize, Clone)]
struct EvalParam {
    fen: String,
    target: String
}

#[derive(Serialize, Deserialize, Clone)]
struct EvalResponse {
    mate_in: isize,
    best_moves: Vec<[String; 2]>
}

#[get("/")]
fn index() -> Template {
    let context: HashMap<String, String> = HashMap::new();
    Template::render("index", context)
}

#[get("/<asset..>")]
fn assets(asset: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("assets/").join(asset)).ok()
}


#[post("/eval", format="application/json", data="<arg>")]
fn eval(arg: Json<EvalParam>, tb: State<Tablebase>) -> Result<Json<EvalResponse>, u32> {
    let param = arg.into_inner().clone();
    let target = Position::from_string(&param.target).unwrap();
    let r = chess::Board::from_fen(param.fen)
        .and_then(|state| if state.is_sane() { Some(state) } else { None })
        .map(|state| tb.eval(state, target))
        .map(|evaluation| {
            let mate_in = match evaluation.value {
                MateIn(i) => i as isize,
                Draw => -1
            };
            let best_moves = evaluation.best_moves.iter().map(|m| [m.get_source().to_string(), m.get_dest().to_string()]).collect();
            Json(EvalResponse {mate_in, best_moves})
        });
    if r.is_none() {
        Err(400)
    }
    else {
        Ok(r.unwrap())
    }
}

pub fn start_server(tb: Tablebase) {
    rocket::ignite()
        .manage(tb)
        .mount("/", routes![index, eval, assets])
        .attach(Template::fairing())
        .launch();
}