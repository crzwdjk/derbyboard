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

use rocket_contrib::JSON;

use std::collections::HashMap;
use std::time::Duration;
use rocket::request::{FromFormValue,Form};
use rocket::response::Redirect;

mod gamestate;
mod roster;
mod staticpages;
mod guard;

use gamestate::{Penalty, ActiveClock};
use gamestate::jamstate::Team;
use guard::{Game, MutGame};

#[derive(Deserialize)]
struct PenaltyCmd {
    skater: String,
    code: char,
}

#[post("/penalties/<team>", format = "application/json", data = "<cmd>")]
fn add_penalty(mut game: MutGame, team: Team, cmd: JSON<PenaltyCmd>) -> JSON<HashMap<String, Vec<Penalty>>> {
    game.penalty(team, cmd.skater.as_str(), cmd.code);
    JSON(game.team_penalties(team))
}

#[get("/penalties/<team>")]
fn get_penalties(game: Game, team: Team) -> JSON<HashMap<String, Vec<Penalty>>>
{
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
fn scoreupdate(game: Game) -> JSON<ScoreUpdate> {
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
fn post_score(mut game: MutGame, cmd: JSON<UpdateCommand>) -> &'static str
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
            game.cur_jam_mut()[team].pass_star(),
        UpdateCommand::set_time(secs) =>
            game.set_time(Duration::new(secs as u64, 0)),
        UpdateCommand::review_lost(team) => game.review_lost(team),
        // don't need to actually do anything for this case.
        UpdateCommand::review_retained(_) => (),
    }; 
    "success"
}

enum TimeType { TimeToDerby, StartAt }
impl<'a> FromFormValue<'a> for TimeType {
    type Error = &'a str;
    fn from_form_value(v: &'a str) -> Result<Self, Self::Error> {
        match v {
            "1" => Ok(TimeType::StartAt),
            "2" => Ok(TimeType::TimeToDerby),
            _ => Err(v),
        }
    }
}

#[derive(Clone,Copy)]
enum TimeAMPM { AM, PM, None }
impl<'a> FromFormValue<'a> for TimeAMPM {
    type Error = &'a str;
    fn from_form_value(v: &'a str) -> Result<Self, Self::Error> {
        match v {
            "AM" => Ok(TimeAMPM::AM),
            "PM" => Ok(TimeAMPM::PM),
            "" => Ok(TimeAMPM::None),
            _ => Err(v),
        }
    }
}

#[derive(FromForm)]
struct StartGameCommand<'a> {
    hometeam: &'a str,
    awayteam: &'a str,
    timetype: TimeType,
    at_hrs: Option<u8>,
    at_mins: Option<u8>,
    at_ampm: TimeAMPM,
    ttd_hrs: Option<u8>,
    ttd_mins: Option<u8>,
    ttd_secs: Option<u8>,
}

fn start_at_time(at_hrs: u8, at_mins: u8, at_ampm: TimeAMPM) -> Result<Duration, &'static str> {
    if at_hrs >= 24 { return Err("Bad hours") }
    if at_mins >= 60 { return Err("Bad minutes") }
    let real_hrs = match at_ampm {
        TimeAMPM::None => at_hrs,
        TimeAMPM::AM if at_hrs < 12 => at_hrs,
        TimeAMPM::AM if at_hrs == 12 => 0,
        TimeAMPM::PM if at_hrs < 12 => at_hrs + 12,
        TimeAMPM::PM if at_hrs >= 12 => 12,
        _ => return Err("Bad hours"),
    };
    let now = chrono::Local::now().time();
    let when = chrono::naive::time::NaiveTime::from_hms(real_hrs as u32, at_mins as u32, 0);
    let duration = if now < when {
        when.signed_duration_since(now)
    } else {
        when.signed_duration_since(now) + chrono::Duration::hours(24)
    };
    duration.to_std().map_err(|_| "negative duration?!")
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
fn gameroster(game: Game, team: Team) -> JSON<roster::Team> {
    let skaters = game.roster(team);
    JSON(skaters.clone()) // ew. Why can't we serialize a ref?
}

fn main() {
    rocket::ignite().mount(
        "/",
        routes![staticpages::index,  gameroster, startgame,
                staticpages::penalties, staticpages::penaltiesjs, get_penalties,
                staticpages::scoreboard, staticpages::scoreboardjs,
                staticpages::mobilejt, staticpages::mobilejtjs,
                scoreupdate, post_score, add_penalty]
    ).launch();
}
