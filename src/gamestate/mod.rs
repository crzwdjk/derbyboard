mod clock;
pub mod jamstate;
mod penaltycodes;
mod savestate;


use std::collections::HashMap;
use std::iter::FromIterator;

use self::penaltycodes::*;
use self::jamstate::*;
use roster;
use std::time::*;
use std::ops::{Index,IndexMut};


pub struct TeamState {
    timeouts: u8,
    reviews: u8,
    roster: roster::Team,
}

impl TeamState {
    fn new(roster: roster::Team) -> TeamState {
        TeamState { timeouts: 3, reviews: 2, roster: roster }
    }
}

#[derive(Serialize, Clone)]
pub struct Penalty {
    jam: (u8, u8),
    code: PenaltyType,
}

#[allow(non_camel_case_types)]
#[derive(Serialize)]
pub enum ActiveClock {
    timeout(Duration),
    team_timeout(Team, Duration),
    review(Team, Duration),
    jam(u8, Duration),
    lineup(Duration),
    time_to_derby(Duration),
    intermission(Duration),
    none,
}

enum ActiveTimeout {
    None, TeamTO(Team), Official, Review(Team), Halftime, TimeToDerby,
}

pub struct GameState {
    team1: TeamState,
    team2: TeamState,
    clock: clock::Clock,
    tostate: ActiveTimeout,
    jams: Vec<JamState>,
    second_period_start: usize,
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

// TO state machine: clock in jam, lineup, OT, OR... get Team TO
// if team has TOs > 0 subtract one and start TO clock.
//  after 1 min, automatically goes to lineup
// OR state machine: clock in jam, lineup, OT... get Team OR
// if team has ORs > 0, subtract 1 and start OR clock.
//  if lost, get OR lost, and set to 0.

impl GameState {
    pub fn new(roster1: roster::Team, roster2: roster::Team,
           time_to_derby: Duration) -> GameState {
        let firstjam = JamState::default();
        let team1 = TeamState::new(roster1);
        let team2 = TeamState::new(roster2);
        GameState { jams: vec![firstjam], team1: team1, team2: team2,
                    clock: clock::Clock::new(time_to_derby), second_period_start: 0,
                    tostate: ActiveTimeout::TimeToDerby,
        }
    }
    pub fn total_score(&self) -> (u32, u32) {
        let mut sums = (0, 0);
        for jam in &self.jams {
            let score = jam.jam_score();
            sums.0 += score.0;
            sums.1 += score.1;
        }
        sums
    }

    pub fn start_jam(&mut self) {
        self.clock.start_jam();
        self.tostate = ActiveTimeout::None;
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
    pub fn get_active_clock(&self) -> ActiveClock {
        let (ty, duration) = self.clock.get_active_clock();
        match ty {
            clock::Clocktype::Jam => ActiveClock::jam(self.jamnum(), duration),
            clock::Clocktype::Lineup => ActiveClock::lineup(duration),
            clock::Clocktype::Intermission => {
                match self.tostate {
                    ActiveTimeout::Halftime =>
                        ActiveClock::intermission(duration),
                    ActiveTimeout::TimeToDerby =>
                        ActiveClock::time_to_derby(duration),
                    ActiveTimeout::None => ActiveClock::none,
                    _ => unreachable!(),
                }
            }
            clock::Clocktype::OtherTimeout => {
                match self.tostate {
                    ActiveTimeout::Official =>
                        ActiveClock::timeout(duration),
                    ActiveTimeout::Review(team) =>
                        ActiveClock::review(team, duration),
                    _ => unreachable!(),
                }
            }
            clock::Clocktype::TeamTimeout =>  {
                if let ActiveTimeout::TeamTO(team) = self.tostate {
                    ActiveClock::team_timeout(team, duration)
                } else { unreachable!() }
            }
            clock::Clocktype::None => {
                ActiveClock::none
            }
        }
    }

    pub fn tick(&mut self) -> () {
        let oldclocktype = self.clock.get_active_clock().0;
        let clock_expired = self.clock.tick();
        if clock_expired {
            let newclocktype = self.clock.get_active_clock().0;
            match oldclocktype {
                clock::Clocktype::Jam => self.stop_jam(),
                clock::Clocktype::Intermission => {
                    if let ActiveTimeout::Halftime = self.tostate {
                        self.team1.reviews = 2;
                        self.team2.reviews = 2;
                    }
                    self.tostate = ActiveTimeout::None;
                    if self.clock.get_time().0 == 2 {
                        self.second_period_start = self.jams.len() - 1;
                    }
                },
                _ => (),
                // Nothing needed for lineup, our jam exists
                // Timeout: expire??
            };
            match newclocktype {
                clock::Clocktype::Intermission => {
                    if self.clock.get_time().0 == 1 {
                        // period 1 expired, in intermission
                        self.second_period_start = self.jams.len() - 1;
                        self.tostate = ActiveTimeout::Halftime;
                    } else {
                        // period 2 expired, game over.
                        // TODO: overtime
                        self.tostate = ActiveTimeout::None;
                    }
                },
                clock::Clocktype::Jam | clock::Clocktype::Lineup => {},
                clock::Clocktype::None => {
                    self.tostate = ActiveTimeout::None;
                }
                _ => unreachable!()
            };
        }
    }
    pub fn jamnum(&self) -> u8 {
        return (self.jams.len() - self.second_period_start) as u8;
    }

    fn jamidx_to_periodjam(&self, jamidx: usize) -> (u8, u8) {
        if self.second_period_start == 0 {
            (1u8, jamidx as u8 + 1)
        } else if jamidx < self.second_period_start {
            (1, jamidx as u8 + 1)
        } else {
            (2, (jamidx - self.second_period_start + 1) as u8)
        }
    }

    pub fn team_penalties(&self, team: Team) -> HashMap<String, Vec<Penalty>> {
        let roster = &self[team].roster.skaters;
        let nskaters = roster.len();
        let mut penalties_by_skater = Vec::new();
        penalties_by_skater.resize(nskaters, Vec::new());
        for (jamidx, jam) in self.jams.iter().enumerate() {
            let jampenalties = &jam[team].penalties;
            let (period, jamnum) = self.jamidx_to_periodjam(jamidx);
            for &(idx, code) in jampenalties {
                penalties_by_skater[idx].push(Penalty {
                    code: code, jam: (period, jamnum)
                });
            }
        }

        let z = roster.iter().map(|s| s.number.clone()).zip(penalties_by_skater.into_iter());
        HashMap::from_iter(z)
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
    }
    pub fn official_timeout(&mut self) -> () {
        self.stop_jam();
        self.tostate = ActiveTimeout::Official;
        self.clock.other_timeout();
    }
    pub fn team_timeout(&mut self, team: Team) -> bool {
        self.stop_jam();
        let timeout_allowed = self[team].timeouts > 0;
        if timeout_allowed {
            self[team].timeouts -=1;
            self.tostate = ActiveTimeout::TeamTO(team);
            self.clock.team_timeout();
        } else {
            self.tostate = ActiveTimeout::Official;
            self.clock.other_timeout();
        }
        timeout_allowed
    }

    pub fn official_review(&mut self, team: Team) -> bool {
        self.stop_jam();
        let review_allowed = self[team].reviews > 0;
        if review_allowed {
            self[team].reviews -= 1;
            self.tostate = ActiveTimeout::Review(team);
        } else {
            self.tostate = ActiveTimeout::Official;
        }
        self.clock.other_timeout();
        review_allowed
    }
    pub fn set_time(&mut self, time: Duration) {
        self.clock.set_time(time);
    }
    pub fn review_lost(&mut self, team: Team) {
        self[team].reviews = 0;
    }
    pub fn roster(&self, team: Team) -> &roster::Team {
        &self[team].roster
    }
    pub fn timeouts(&self) -> (u8, u8) {
        (self[Team::Home].timeouts, self[Team::Away].timeouts)
    }
    pub fn reviews(&self) -> (u8, u8) {
        (self[Team::Home].reviews, self[Team::Away].reviews)
    }
    pub fn cur_jam(&self) -> &JamState { self.jams.last().unwrap() }
    pub fn prev_jam(&self) -> Option<&JamState> {
        let len = self.jams.len();
        if len >= 2 {
            self.jams.get(len - 2)
        } else { None }
    }
    pub fn cur_jam_mut(&mut self) -> &mut JamState {
        self.jams.last_mut().unwrap()
    }
    pub fn get_jam(&self, i: usize) -> &JamState { &self.jams[i - 1] }
    pub fn get_jam_mut(&mut self, i: usize) -> &mut JamState { &mut self.jams[i - 1] }
    pub fn jams(&self) -> &[JamState] { self.jams.as_ref() }
}

