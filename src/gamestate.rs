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

pub struct GameState {
    team1: u32,
    team2: u32,
    timeouts_left: [u8; 2],
    ors: [u8; 2],
    gameclock: (u8, Duration),
    clock_running: bool,
    jams: Vec<JamState>,
    last_update: Instant,
    timeouts: Vec<(TimeoutKind, Duration)>,
}

enum TimeoutKind {
    Official,
    Team(u8),
    Review(u8),
}

use std::iter::Sum;

impl GameState {
    fn new(team1: u32, team2: u32) -> GameState {
        let firstjam = JamState::default();
        return GameState { team1: team1, team2: team2, jams: vec![firstjam],
                           ors: [2, 2], timeouts: [3, 3],
                           gameclock: (1, Duration::new(30*60, 0)),
                           last_update: Instant::now(), clock_running: false,
        };
    }
    pub fn new_jam(&mut self) {
        self.jams.push(JamState::default())
    }
    pub fn timeout(&mut self, kind: TimeoutKind) {
        // stop jam, if one is going
        // start timeout clock if not running
        // if running and type is Official, reset clock
        // if type is Team or Review, decrement team's count
        
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
        self.clock_running = true;
        let jam = self.jams.last_mut().unwrap();
        if let None = jam.starttime {
            jam.starttime = Some(Instant::now());
        }
    }
    pub fn stop_jam(&mut self) {
        {
            let jam = self.jams.last_mut().unwrap();
            if let None = jam.endtime {
                jam.endtime = Some(Instant::now());
            }
        }
        self.jams.push(JamState::default());
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
    pub fn get_time(&self) -> (u8, Duration) {
        self.gameclock
    }
    pub fn get_jam_time(&self) -> (u8, Duration) {
        let jam = self.jams.last().unwrap();
        let time = match jam.starttime {
            None => Duration::new(120, 0),
            Some(starttime) => match jam.endtime {
                None => Instant::now() - starttime,
                Some(endtime) => Duration::new(0,0),
            }
        };
        (self.jams.len() as u8, time)
    }
    pub fn tick(&mut self) -> () {
        let now = Instant::now();
        if self.clock_running {
            let decrement = now - self.last_update;
            if self.gameclock.1 >= decrement {
                self.gameclock.1 -= decrement;
            } else {
                self.gameclock.1 = Duration::new(0,0);
                // TODO: end period
            }
        } else {
            // TODO: handle timeout
        }
            
        self.last_update = now;
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

