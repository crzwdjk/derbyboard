#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate lazy_static;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate derbyjson;

use rocket::response::*;
use rocket_contrib::JSON;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;

mod gamestate;
mod jamstate;
mod clock;
mod roster;
mod penaltycodes;

use gamestate::{Penalty, ActiveClock};
use jamstate::Team;
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[derive(Deserialize)]
struct PenaltyCmd {
    skater: String,
    code: char,
}

#[post("/penalties/<team>", format = "application/json", data = "<cmd>")]
fn add_penalty(team: Team, cmd: JSON<PenaltyCmd>) -> JSON<HashMap<String, Vec<Penalty>>> {
    let mut game = gamestate::get_game_mut();
    game.penalty(team, cmd.skater.as_str(), cmd.code);
    JSON(game.team_penalties(team))
}

#[get("/penalties/<team>")]
fn get_penalties(team: Team) -> JSON<HashMap<String, Vec<Penalty>>>
{
    let game = gamestate::get_game();
    JSON(game.team_penalties(team))
}

#[derive(Serialize)]
struct ScoreUpdate {
    score: (u32, u32),
    jamscore: (u32, u32),
    gameclock: (u8, Duration),
    activeclock: ActiveClock,
    timeouts: (u8, u8),
    reviews: (u8, u8),
}

#[get("/score/update")]
fn scoreupdate() -> JSON<ScoreUpdate> {
    let game = gamestate::get_game();
    let cur_jam = game.cur_jam();
    let jamscore = if cur_jam.starttime.is_some() {
        cur_jam.jam_score()
    } else {
        match game.prev_jam() {
            Some(ref prev_jam) => prev_jam.jam_score(),
            None => (0, 0)
        }
    };

    JSON(ScoreUpdate {
        score: game.total_score(), jamscore: jamscore,
        gameclock: game.get_time(), activeclock: game.get_active_clock(),
        reviews: game.reviews(), timeouts: game.timeouts(),
    })
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
enum UpdateCommand {
    score_adj(i8, i8),
    //score_set(i8, i8),
    set_time(u16),
    start_jam,
    stop_jam,
    team_timeout(Team),
    star_pass(Team),
    official_timeout,
    official_review(Team),
    review_lost(Team),
    review_retained(Team),
}

#[post("/score/update", format = "application/json", data = "<cmd>")]
fn post_score(cmd: JSON<UpdateCommand>) -> &'static str
{
    let mut game = gamestate::get_game_mut();
    match cmd.0 {
        UpdateCommand::score_adj(a1, a2) =>
            game.cur_jam_mut().adj_score(a1, a2),
        UpdateCommand::start_jam => game.start_jam(),
        UpdateCommand::stop_jam => game.stop_jam(),
        UpdateCommand::official_timeout => game.official_timeout(),
        UpdateCommand::team_timeout(team) => { game.team_timeout(team); },
        UpdateCommand::star_pass(team) =>
            game.cur_jam_mut()[team].pass_star(),
        UpdateCommand::set_time(secs) =>
            game.set_time(Duration::new(secs as u64, 0)),
        _ => {}
    }; 
    "success"
}

#[get("/score")]
fn scoreboard() -> content::HTML<&'static str> {
    content::HTML(include_str!("scoreboard.html"))
}

#[get("/scoreboard.js")]
fn scoreboardjs() -> &'static str { include_str!("scoreboard.js") }

#[get("/penalties")]
fn penalties() -> content::HTML<&'static str> {
    content::HTML(include_str!("penalties.html"))
}

#[get("/penalties.js")]
fn penaltiesjs() -> &'static str { include_str!("penalties.js") }

#[get("/mobilejt.js")]
fn mobilejtjs() -> &'static str { include_str!("mobilejt.js") }

#[get("/mobilejt")]
fn mobilejt() -> content::HTML<&'static str> {
    content::HTML(include_str!("mobilejt.html"))
}

#[get("/gameroster/<team>")]
fn gameroster(team: Team) -> JSON<roster::Team> {
    let game = gamestate::get_game();
    let skaters = game.roster(team);
    JSON(skaters.clone()) // ew. Why can't we serialize a ref?
}

fn main() {
    let rosters = roster::load_rosters(OsStr::new("rosters")).unwrap_or(Vec::new());
    println!("Loaded {} rosters", rosters.len());
    gamestate::start_game(&rosters[0], &rosters[1]);
    thread::spawn(move || {
        loop {
            thread::park_timeout(Duration::new(0, 100_000_000));
            gamestate::get_game_mut().tick();
        }
    });

    rocket::ignite().mount("/", routes![index,  gameroster,
                                        penalties, penaltiesjs, get_penalties,
                                        scoreboard, scoreboardjs,
                                        mobilejt, mobilejtjs,
                                        scoreupdate, post_score, add_penalty]).launch();
}
