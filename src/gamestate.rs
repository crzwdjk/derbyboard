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

struct TeamState {
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

#[derive(Clone,Copy)]
pub enum TimeoutKind {
    Official,
    Team(u8),
    Review(u8),
}

impl TimeoutKind {
    pub fn from_str(s: &str) -> Option<TimeoutKind> {
        match s {
            "official" => Some(TimeoutKind::Official),
            "team 1" => Some(TimeoutKind::Team(0)),
            "team 2" => Some(TimeoutKind::Team(1)),
            "or 1" => Some(TimeoutKind::Review(0)),
            "or 2" => Some(TimeoutKind::Review(1)),
            _ => None
        }
    }
}

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
    pub fn new_jam(&mut self) {
        self.jams.push(JamState::default())
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
            _ => unimplemented!() // TODO: handle TO/OR
        };
        JamTime { clock: duration, kind: kind }
    }

    pub fn timeout(&mut self, kind: TimeoutKind) { unimplemented!() }

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
        self.clock.tick();
    }
    pub fn jamnum(&self) -> u8 {
        // TODO: period handling
        return self.jams.len() as u8;
    }
    pub fn get_team(&self, teamnum: u8) -> Option<&roster::Team> {
        match teamnum {
            1 => Some(&self.team1.roster),
            2 => Some(&self.team2.roster),
            _ => None,
        }
    }

    pub fn team_penalties(&self, teamnum: u8) -> Option<HashMap<String, Vec<Penalty>>> {
        let ref roster = match self.get_team(teamnum) {
            None => return None, Some(t) => t }.skaters;
        let nskaters = roster.len();
        let mut penalties_by_skater = Vec::new();
        penalties_by_skater.resize(nskaters, Vec::new());
        for (jamnum, jam) in self.jams.iter().enumerate() {
            let jampenalties = if teamnum == 1 {
                &jam.team1.penalties
            } else {
                &jam.team2.penalties
            };
            for &(idx, code) in jampenalties {
                penalties_by_skater[idx].push(Penalty {
                    code: code, jam: (jamnum + 1) as u8
                });
            }
        }

        let z = roster.iter().map(|s| s.number.clone()).zip(penalties_by_skater.into_iter());
        Some(HashMap::from_iter(z))
    }

    pub fn penalty(&mut self, teamnum: u8, skater: &str, code: char) {
        let skater_idx = {
            let team = match self.get_team(teamnum) { Some(t) => t, None => return };
            match team.skaters.binary_search_by_key(&skater, |s| &*s.number) {
                Ok(idx) => idx,
                Err(_) => { println!("skater {} not found", skater); return }
            }
        };
        let jam = self.jams.last_mut().unwrap();
        let mut penalties = if teamnum == 1 { &mut jam.team1.penalties } else { &mut jam.team2.penalties };
        penalties.push((skater_idx, PenaltyType::from_char(code)));
        println!("got penalty {} for skater: {} at {} ", code, skater, skater_idx);
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
