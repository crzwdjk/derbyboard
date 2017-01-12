pub enum PenaltyCode {
    BackBlock,
    LowBlock,
    HighBlock,
    Forearms,
    Elbows,
    BlockWithHead,
    MultiPlayer,
    Directional,
    CutTrack,
    IllegalProcedure,
    OutOfPlay,
    OutOfBounds,
    SkatingOutOfBounds,
    Insubordination,
    Misconduct,
    DelayOfGame,
    Unknown,
}

#[derive(Default)]
struct TeamJamState {
    lineup: [u32; 6],
    points_j: Vec<u8>,
    points_p: Vec<u8>,
    penalties: Vec<(u32, PenaltyCode)>,
    boxtrips: Vec<(u32, u32, u32)>,
    starpass: bool,
}

use std::cmp::max;
use std::time::*;
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
    fn new(team1: u32, team2: u32) -> GameState {
        let firstjam = JamState::default();
        let team1 = TeamState { timeouts: 3, reviews: 2 };
        let team2 = TeamState { timeouts: 3, reviews: 2 };
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
}

pub fn start_game(team1: u32, team2: u32) -> () {
    match CUR_GAME {
        None => unsafe {
            let gp = &CUR_GAME as *const Option<Mutex<GameState>> as *mut Option<Mutex<GameState>>;
            *gp = Some(Mutex::new(GameState::new(team1, team2)));
        },
        Some(ref m) => {
            let mut mg = m.lock().unwrap();
            *mg = GameState::new(team1, team2);
        }
    }
}

pub fn get_game<'a>() -> MutexGuard<'a, GameState> {
    if let Some(ref m) = CUR_GAME {
        m.lock().unwrap()
    } else {
        panic!();
    }
}

use std::sync::{Mutex,MutexGuard};

static CUR_GAME: Option<Mutex<GameState>> = None;
