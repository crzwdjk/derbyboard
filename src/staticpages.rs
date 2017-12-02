use rocket::response::content;
use handlebars::Handlebars;
use handlebars;

use guard::get_game;
use roster;
use gamestate::jamstate::Team;

#[get("/score")]
pub fn scoreboard() -> content::Html<&'static str> {
    content::Html(include_str!("scoreboard.html"))
}

#[get("/scoreboard.js")]
pub fn scoreboardjs() -> &'static str { include_str!("scoreboard.js") }

#[get("/penalties")]
fn penalties() -> content::Html<&'static str> {
    content::Html(include_str!("penalties.html"))
}

#[get("/penalties.js")]
fn penaltiesjs() -> &'static str { include_str!("penalties.js") }

#[get("/mobilejt.js")]
fn mobilejtjs() -> &'static str { include_str!("mobilejt.js") }

#[get("/scoresheet")]
fn scoresheet() -> content::Html<&'static str> {
    content::Html(include_str!("scoresheet.html"))
}

#[get("/scoresheet.js")]
fn scoresheetjs() -> &'static str { include_str!("scoresheet.js") }


#[get("/mobilejt")]
fn mobilejt() -> content::Html<&'static str> {
    content::Html(include_str!("mobilejt.html"))
}

#[derive(Serialize)]
struct GameInfo<'a> {
    home: &'a str,
    away: &'a str,
}

#[derive(Serialize)]
struct HomepageState<'a> {
    game_in_progress: bool,
    rosters: Vec<(String, String)>,
    game: Option<GameInfo<'a>>,
}

#[get("/")]
fn index() -> Result<content::Html<String>, handlebars::RenderError> {
    let rosters = roster::list_rosters().into_iter().map(|(f, r)| (f, r.name));
    let guard = get_game();
    let game = guard.as_ref();
    let gameinfo = game.map(|g| GameInfo { home: g.roster(Team::Home).name.as_str(),
                                           away: g.roster(Team::Away).name.as_str() });

    HBS.render("startgame", &HomepageState {
        game_in_progress: game.is_some(),
        rosters: rosters.collect(),
        game: gameinfo,
    } ).map(|s| content::Html(s))

}

fn init_templates() -> Handlebars {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("startgame",
                                        include_str!("startgame.hbs")).unwrap();
    // ...
    handlebars
}

lazy_static! {
    static ref HBS: Handlebars = init_templates();
}
