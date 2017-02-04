use std::collections::HashMap;
use std::iter::FromIterator;

use penaltycodes::*;

#[derive(Default)]
struct TeamJamState {
    lineup: [u32; 6],
    points_j: Vec<u8>,
    points_p: Vec<u8>,
    penalties: Vec<(usize, PenaltyType)>,
    starpass: bool,
    lead: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Team {
    Home = 1,
    Away = 2,
}

impl Team {
    fn from_int(v: usize) -> Option<Team> {
        match v {
            1 => Some(Team::Home),
            2 => Some(Team::Away),
            _ => None,
        }
    }
}

use rocket;
impl<'a> rocket::request::FromParam<'a> for Team {
    type Error = &'static str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match param {
            "1" => Ok(Team::Home),
            "2" => Ok(Team::Away),
            _ => Err("Team must be 1 or 2"),
        }
    }
}
use std::cmp::max;
use std::time::*;
use roster;
use clock;

impl TeamJamState {
    fn update_points(&mut self, adj: i8) {
        let mut pointvec = if self.starpass { &mut self.points_p }
                           else { &mut self.points_j };
        if let None = pointvec.last() {
            pointvec.push(0)
        }
        let mut p = pointvec.last_mut().unwrap();
        *p = max(*p as i8 + adj, 0) as u8;
    }
    fn pass_star(&mut self) { self.starpass = true }
}


#[derive(Default)]
struct JamState {
    team1: TeamJamState,
    team2: TeamJamState,
    starttime: Option<Instant>,
    endtime: Option<Instant>,
}

use std::ops::{Index,IndexMut};
impl Index<Team> for JamState {
    type Output = TeamJamState;

    fn index(&self, team: Team) -> &TeamJamState {
        match team {
            Team::Home => &self.team1,
            Team::Away => &self.team2,
        }
    }
}
impl IndexMut<Team> for JamState {
    fn index_mut(&mut self, team: Team) -> &mut TeamJamState {
        match team {
            Team::Home => &mut self.team1,
            Team::Away => &mut self.team2,
        }
    }
}

pub struct TeamState {
    timeouts: u8,
    reviews: u8,
    roster: roster::Team,
}

impl TeamState {
    fn new(roster: &roster::Team) -> TeamState {
        TeamState { timeouts: 3, reviews: 2, roster: roster.clone() }
    }
}

#[derive(Serialize, Clone)]
pub struct Penalty {
    jam: u8,
    code: PenaltyType,
}

#[derive(Serialize)]
pub struct TeamPenalties {
    team: u8,
    penalties: HashMap<String, Vec<Penalty>>,
}

pub struct GameState {
    team1: TeamState,
    team2: TeamState,
    clock: clock::Clock,
    jams: Vec<JamState>,
}

impl Index<Team> for GameState {
    type Output = TeamState;

    fn index(&self, team: Team) -> &TeamState {
        match team {
            Team::Home => &self.team1,
            Team::Away => &self.team2,
        }
    }
}
impl IndexMut<Team> for GameState {
    fn index_mut(&mut self, team: Team) -> &mut TeamState {
        match team {
            Team::Home => &mut self.team1,
            Team::Away => &mut self.team2,
        }
    }
}

/*
#[derive(Clone,Copy)]
pub enum TimeoutKind {
    Official,
    Team(u8),
    Review(u8),
}*/

// TO state machine: clock in jam, lineup, OT, OR... get Team TO
// if team has TOs > 0 subtract one and start TO clock.
//  after 1 min, automatically goes to lineup
// OR state machine: clock in jam, lineup, OT... get Team OR
// if team has ORs > 0, subtract 1 and start OR clock.
//  if lost, get OR lost, and set to 0.
pub enum ClockKind {
    TeamTimeout { team: u8, which: u8 },
    OfficialReview {team: u8 },
    OfficialTimeout, Jam, Lineup, Intermission
}

pub struct JamTime {
    pub clock: Duration,
    pub kind: ClockKind,
}

use std::iter::Sum;

impl GameState {
    fn new(roster1: &roster::Team, roster2: &roster::Team) -> GameState {
        let firstjam = JamState::default();
        let team1 = TeamState::new(roster1);
        let team2 = TeamState::new(roster2);
        GameState { jams: vec![firstjam], team1: team1, team2: team2,
                    clock: clock::Clock::new(),
        }
    }
    pub fn total_score(&self) -> (u32, u32) {
        let mut sums = (0, 0);
        for jam in &self.jams {
            let p1j = jam.team1.points_j.iter().sum::<u8>();
            let p1p = jam.team1.points_p.iter().sum::<u8>();
            let p2j = jam.team2.points_j.iter().sum::<u8>();
            let p2p = jam.team2.points_p.iter().sum::<u8>();
            sums.0 += (p1j as u32) + (p1p as u32);
            sums.1 += (p2j as u32) + (p2p as u32);
        }
        sums
    }

    pub fn start_jam(&mut self) {
        self.clock.start_jam();
        self.jams.last_mut().unwrap().starttime = Some(Instant::now());
    }
    pub fn stop_jam(&mut self) {
        self.clock.stop_jam();
        self.jams.last_mut().unwrap().endtime = Some(Instant::now());
        self.jams.push(JamState::default());
    }
    pub fn get_time(&self) -> (u8, Duration) {
        self.clock.get_time()
    }
    pub fn get_active_clock(&self) -> JamTime {
        let (ty, duration) = self.clock.get_active_clock();
        let kind = match ty {
            clock::Clocktype::Jam => ClockKind::Jam,
            clock::Clocktype::Lineup => ClockKind::Lineup,
            clock::Clocktype::Intermission => ClockKind::Intermission,
            clock::Clocktype::OtherTimeout => ClockKind::OfficialTimeout, // XXX: get rid of ClockKind
            _ => unimplemented!() // TODO: handle TO/OR
        };
        JamTime { clock: duration, kind: kind }
    }

    pub fn jam_score(&self) -> (u32, u32) {
        if let Some(jam) = self.jams.last() {
            let p1j = jam.team1.points_j.iter().sum::<u8>();
            let p1p = jam.team1.points_p.iter().sum::<u8>();
            let p2j = jam.team2.points_j.iter().sum::<u8>();
            let p2p = jam.team2.points_p.iter().sum::<u8>();
            (p1j as u32 + p1p as u32, p2j as u32 + p2p as u32)
        } else {
            (0, 0)
        }
    }
    pub fn adj_score(&mut self, t1adj: i8, t2adj: i8) -> () {
        if let Some(jam) = self.jams.last_mut() {
            jam.team1.update_points(t1adj);
            jam.team2.update_points(t2adj);
        }
    }

    pub fn tick(&mut self) -> () {
        let clocktype = self.clock.get_active_clock().0;
        let clock_expired = self.clock.tick();
        if clock_expired {
            match clocktype {
                clock::Clocktype::Jam => self.stop_jam(),
                _ => (),
                // Nothing needed for lineup, our jam exists
                // Timeout: expire 
            }
        }
    }
    pub fn jamnum(&self) -> u8 {
        // TODO: period handling
        return self.jams.len() as u8;
    }

    pub fn team_penalties(&self, team: Team) -> Option<HashMap<String, Vec<Penalty>>> {
        let roster = &self[team].roster.skaters;
        let nskaters = roster.len();
        let mut penalties_by_skater = Vec::new();
        penalties_by_skater.resize(nskaters, Vec::new());
        for (jamnum, jam) in self.jams.iter().enumerate() {
            let jampenalties = &jam[team].penalties;
            for &(idx, code) in jampenalties {
                penalties_by_skater[idx].push(Penalty {
                    code: code, jam: (jamnum + 1) as u8
                });
            }
        }

        let z = roster.iter().map(|s| s.number.clone()).zip(penalties_by_skater.into_iter());
        Some(HashMap::from_iter(z))
    }

    pub fn penalty(&mut self, team: Team, skater: &str, code: char) {
        let skater_idx = {
            let team = &self[team].roster;
            match team.skaters.binary_search_by_key(&skater, |s| &*s.number) {
                Ok(idx) => idx,
                Err(_) => { println!("skater {} not found", skater); return }
            }
        };
        let jam = self.jams.last_mut().unwrap();
        let mut penalties = &mut jam[team].penalties;
        penalties.push((skater_idx, PenaltyType::from_char(code)));
        println!("got penalty {} for skater: {} at {} ", code, skater, skater_idx);
    }
    pub fn official_timeout(&mut self) -> () {
        self.clock.other_timeout();
    }
    pub fn team_timeout(&mut self, team: Team) -> bool {
        let timeout_happened = {
            let mut t = &mut self[team];
            if t.timeouts > 0 {
                t.timeouts -= 1; true
            } else { false }
        };
        if timeout_happened {
            self.clock.team_timeout();
            true
        } else {
            self.clock.other_timeout();
            false
        }
    }
    pub fn roster(&self, team: Team) -> &roster::Team {
        &self[team].roster
    }
 }

pub fn start_game(team1: &roster::Team, team2: &roster::Team) -> () {
    match CUR_GAME {
        None => unsafe {
            let gp = &CUR_GAME as *const Option<RwLock<GameState>> as *mut Option<RwLock<GameState>>;
            *gp = Some(RwLock::new(GameState::new(team1, team2)));
        },
        Some(ref m) => {
            let mut mg = m.write().unwrap();
            *mg = GameState::new(team1, team2);
        }
    }
}

pub fn get_game<'a>() -> RwLockReadGuard<'a, GameState> {
    if let Some(ref m) = CUR_GAME {
        m.read().unwrap()
    } else {
        panic!();
    }
}

pub fn get_game_mut<'a>() -> RwLockWriteGuard<'a, GameState> {
    if let Some(ref m) = CUR_GAME {
        m.write().unwrap()
    } else {
        panic!();
    }
}

//use std::sync::{Mutex,MutexGuard};
use std::sync::{RwLock,RwLockReadGuard,RwLockWriteGuard};

static CUR_GAME: Option<RwLock<GameState>> = None;
