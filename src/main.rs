#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate lazy_static;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate derbyjson;
extern crate handlebars;
extern crate chrono;

use rocket_contrib::Json;

use std::collections::HashMap;
use std::time::Duration;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::http::RawStr;

mod gamestate;
mod roster;
mod staticpages;
mod guard;
mod timetoderby;

use gamestate::{Penalty, ActiveClock};
use gamestate::jamstate::{Team,TeamJamState};
use guard::{Game, MutGame};
use timetoderby::*;

#[derive(Deserialize)]
struct PenaltyCmd {
    skater: String,
    code: char,
}

#[post("/penalties/<team>", format = "application/json", data = "<cmd>")]
fn add_penalty(mut game: MutGame, team: Team, cmd: Json<PenaltyCmd>)
               -> Json<HashMap<String, Vec<Penalty>>>
{
    game.penalty(team, cmd.skater.as_str(), cmd.code);
    Json(game.team_penalties(team))
}

#[get("/penalties/<team>")]
fn get_penalties(game: Game, team: Team) -> Json<HashMap<String, Vec<Penalty>>>
{
    Json(game.team_penalties(team))
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
fn scoreupdate(game: Game) -> Json<ScoreUpdate> {
    let cur_jam = game.cur_jam();
    let jamscore = if cur_jam.starttime.is_some() {
        cur_jam.jam_score()
    } else {
        match game.prev_jam() {
            Some(ref prev_jam) => prev_jam.jam_score(),
            None => (0, 0)
        }
    };

    Json(ScoreUpdate {
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
fn post_score(mut game: MutGame, cmd: Json<UpdateCommand>) -> &'static str
{
    match cmd.0 {
        UpdateCommand::score_adj(a1, a2) =>
            game.cur_jam_mut().adj_score(a1, a2),
        UpdateCommand::start_jam => game.start_jam(),
        UpdateCommand::stop_jam => game.stop_jam(),
        UpdateCommand::official_timeout => game.official_timeout(),
        UpdateCommand::team_timeout(team) => { game.team_timeout(team); },
        UpdateCommand::official_review(team) => { game.official_review(team); }
        UpdateCommand::star_pass(team) =>
            game.cur_jam_mut()[team].set_starpass(true),
        UpdateCommand::set_time(secs) =>
            game.set_time(Duration::new(secs as u64, 0)),
        UpdateCommand::review_lost(team) => game.review_lost(team),
        // don't need to actually do anything for this case.
        UpdateCommand::review_retained(_) => (),
    }; 
    "success"
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum JamCommand {
    Lead(bool),
    Lost(bool),
    Call(bool),
    Starpass(bool),
    ScoringTrip { trip: u8, points: u8 },
}

#[post("/jam/<jam>/<team>/command", format = "application/json", data = "<cmd>")]
fn jam_command(mut game: MutGame, jam: usize, team: Team, cmd: Json<JamCommand>) -> &'static str
{
    let ref mut teamjam = game.get_jam_mut(jam)[team];
    match cmd.0 {
        JamCommand::Lead(yesno) => teamjam.set_lead(yesno),
        JamCommand::Call(yesno) => teamjam.set_call(yesno),
        JamCommand::Lost(yesno) => teamjam.set_lost(yesno),
        JamCommand::Starpass(yesno) => teamjam.set_starpass(yesno),
        JamCommand::ScoringTrip { trip, points } => teamjam.set_score(trip, points),
    };
    "success"
}

#[get("/scoresheet/update")]
fn get_scoresheet(game: Game) -> Json<Vec<(TeamJamState, TeamJamState)>> {
    let stuff = game.jams().iter().map(|jamstate| {
        (jamstate[Team::Home].clone(), jamstate[Team::Away].clone())
    }).collect::<Vec<_>>();

    Json(stuff)
}

#[derive(FromForm)]
struct StartGameCommand<'a> {
    hometeam: &'a RawStr,
    awayteam: &'a RawStr,
    timetype: TimeType,
    at_hrs: Option<u8>,
    at_mins: Option<u8>,
    at_ampm: TimeAMPM,
    ttd_hrs: Option<u8>,
    ttd_mins: Option<u8>,
    ttd_secs: Option<u8>,
}

#[post("/startgame", data = "<form>")]
fn startgame<'a>(form: Form<'a, StartGameCommand<'a>>) -> Redirect
{
    let cmd = form.get();
    let team1 = roster::get_team(cmd.hometeam, String::from("Home")).unwrap(); // XXX
    let team2 = roster::get_team(cmd.awayteam, String::from("Away")).unwrap(); // XXX
    let time = match cmd.timetype {
        TimeType::TimeToDerby => Duration::new((cmd.ttd_hrs.unwrap_or_default() as u64) * 3600
                                               + (cmd.ttd_mins.unwrap_or_default() as u64) * 60
                                               + (cmd.ttd_secs.unwrap_or_default() as u64), 0),
        TimeType::StartAt => start_at_time(cmd.at_hrs.unwrap_or_default(),
                                           cmd.at_mins.unwrap_or_default(),
                                           cmd.at_ampm).unwrap(),// XXX
    };
    guard::start_game(team1, team2, time);
    Redirect::to("/")
}

#[get("/gameroster/<team>")]
fn gameroster(game: Game, team: Team) -> Json<roster::Team> {
    let skaters = game.roster(team);
    Json(skaters.clone()) // ew. Why can't we serialize a ref?
}

fn main() {
    rocket::ignite().mount(
        "/",
        routes![staticpages::index,  gameroster, startgame,
                staticpages::penalties, staticpages::penaltiesjs, get_penalties,
                staticpages::scoreboard, staticpages::scoreboardjs,
                staticpages::mobilejt, staticpages::mobilejtjs,
                staticpages::scoresheet, staticpages::scoresheetjs,
                get_scoresheet, jam_command,
                scoreupdate, post_score, add_penalty]
    ).launch();
}
