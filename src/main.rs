#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;

use rocket::response::*;
use rocket_contrib::JSON;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::ffi::OsStr;

mod gamestate;
mod clock;
mod roster;
mod penaltycodes;

use gamestate::{Penalty,Team};
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[get("/penalties")]
fn penalties() -> content::HTML<&'static str> {
    content::HTML(include_str!("penalties.html"))
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
    JSON(game.team_penalties(team).unwrap())
}

#[get("/score")]
fn scoreboard() -> content::HTML<&'static str> {
    content::HTML(include_str!("scoreboard.html"))
}

#[derive(Serialize)]
struct ScoreUpdate {
    score: (u32, u32),
    jamscore: (u32, u32),
    gameclock: (u8, Duration),
    jamclock: (u8, Duration),
    lineupclock: Option<Duration>,
    timeout: Option<Duration>,
}

#[get("/score/update")]
fn scoreupdate() -> JSON<ScoreUpdate> {
    let game = gamestate::get_game();
    let activeclock = game.get_active_clock();
    let mut lineupclock = None;
    let mut jamclock = (game.jamnum(), Duration::new(120, 0));
    let mut timeout = None;
    match activeclock.kind {
        gamestate::ClockKind::Jam => jamclock.1 = activeclock.clock,
        gamestate::ClockKind::Lineup => lineupclock = Some(activeclock.clock),
        gamestate::ClockKind::OfficialTimeout => timeout = Some(activeclock.clock),
        _ => (),
    };

    JSON(ScoreUpdate {
        score: game.total_score(), jamscore: game.jam_score(),
        gameclock: game.get_time(), jamclock: jamclock,
        lineupclock: lineupclock, timeout: timeout,
    })
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
enum UpdateCommand {
    score_adj(i8, i8),
    score_set(i8, i8),
    start_jam,
    stop_jam,
    team_timeout(Team),
    official_timeout,
}

#[post("/score/update", format = "application/json", data = "<cmd>")]
fn post_score(cmd: JSON<UpdateCommand>) -> &'static str
{
    let mut game = gamestate::get_game_mut();
    match cmd.0 {
        UpdateCommand::score_adj(a1, a2) => game.adj_score(a1, a2),
        UpdateCommand::start_jam => {
            println!("Jam On!");
            game.start_jam();
        },
        UpdateCommand::stop_jam => {
            println!("Jam Off!");
            game.stop_jam();
        },
        UpdateCommand::official_timeout => {
            game.official_timeout();
        },
        UpdateCommand::team_timeout(team) => {
            game.team_timeout(team);
        }
        _ => { /* XXX */ }
    }
    "success"
}

#[get("/scoreboard.js")]
fn scoreboardjs() -> &'static str {
    include_str!("scoreboard.js")
}

#[get("/penalties.js")]
fn penaltiesjs() -> &'static str {
    include_str!("penalties.js")
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

    rocket::ignite().mount("/", routes![index, penalties, penaltiesjs, gameroster,
                                        scoreboard, scoreboardjs,
                                        scoreupdate, post_score, add_penalty]).launch();
}
