#![feature(plugin, proc_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket::response::*;
use rocket_contrib::JSON;

use std::thread;
use std::time::Duration;

mod gamestate;
mod clock;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[post("/start_game")]
fn new_game() -> Redirect {
    Redirect::to("/game")
}

#[get("/game")]
fn show_game() -> &'static str {
                "a game"
            }

#[get("/game/<gameid>/penalties")]
fn penalty_board(gameid: usize) -> &'static str {
    "penalties"
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
}

#[get("/score/update")]
fn scoreupdate() -> JSON<ScoreUpdate> {
    let game = gamestate::get_game();
    let activeclock = game.get_active_clock();
    let mut lineupclock = None;
    let mut jamclock = (game.jamnum(), Duration::new(120, 0));
    match activeclock.kind {
        gamestate::ClockKind::Jam => jamclock.1 = activeclock.clock,
        gamestate::ClockKind::Lineup => lineupclock = Some(activeclock.clock),
        _ => (),
    };

    JSON(ScoreUpdate {
        score: game.total_score(), jamscore: game.jam_score(),
        gameclock: game.get_time(), jamclock: jamclock,
        lineupclock: lineupclock,
    })
}

#[derive(Deserialize)]
struct UpdateCommand {
    score_adj: Option<[i8; 2]>,
    score_set: Option<[i8; 2]>,
    start_jam: Option<bool>,
    start_timeout: Option<String>,
}

#[post("/score/update", format = "application/json", data = "<cmd>")]
fn post_score(cmd: JSON<UpdateCommand>) -> &'static str
{
    let mut game = gamestate::get_game();
    if let Some(adj) = cmd.0.score_adj {
        game.adj_score(adj[0], adj[1]);
    } else if let Some(start) = cmd.0.start_jam {
        if start {
            println!("Jam On!");
            game.start_jam();
        } else {
            println!("Jam Off!");
            game.stop_jam();
        }
    }
    if let Some(to_command) = cmd.0.start_timeout {
        let tokind = gamestate::TimeoutKind::from_str(&to_command);
        if let Some(kind) = tokind {
            game.timeout(kind);
        }
    }
    "success"
}

#[get("/scoreboard.js")]
fn scoreboardjs() -> &'static str {
    include_str!("scoreboard.js")
}

fn main() {
    gamestate::start_game(0, 1);
    thread::spawn(move || {
        loop {
            thread::park_timeout(Duration::new(0, 100_000_000));
            gamestate::get_game().tick();
        }
    });
    rocket::ignite().mount("/", routes![index, show_game, penalty_board,
                                        scoreboard, scoreboardjs, new_game,
                                        scoreupdate, post_score]).launch();
}

