use std::cmp::max;
use std::time::*;

use super::penaltycodes::*;

#[derive(Default,Serialize,Clone)]
pub struct TeamJamState {
    lineup: [u32; 6],
    jammerpoints: Vec<u8>,
    pivotpoints: Vec<u8>,
    pub penalties: Vec<(usize, PenaltyType)>,
    starpass: bool,
    lead: bool,
    lost: bool,
    call: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Team {
    Home = 1,
    Away = 2,
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

impl TeamJamState {
    pub fn update_points(&mut self, adj: i8) {
        let mut pointvec = if self.starpass { &mut self.pivotpoints }
                           else { &mut self.jammerpoints };
        if let None = pointvec.last() {
            pointvec.push(0)
        }
        let mut p = pointvec.last_mut().unwrap();
        *p = max(*p as i8 + adj, 0) as u8;
    }
    pub fn set_starpass(&mut self, yes: bool) {
        self.starpass = yes;
        if self.starpass && self.lead {
            self.lead = false;
            self.lost = true;
        }
    }
    pub fn set_score(&mut self, trip: u8, points: u8) {
        let idx = trip as usize;
        let points = if !self.starpass {
            &self.jammerpoints
        } else {
            &self.pivotpoints
        };
    }
    pub fn set_lead(&mut self, yes: bool) { self.lead = yes }
    pub fn set_call(&mut self, yes: bool) { self.call = yes }
    pub fn set_lost(&mut self, yes: bool) { self.lost = yes; if self.lost { self.lead = false } }
}


#[derive(Default)]
pub struct JamState {
    pub team1: TeamJamState,
    pub team2: TeamJamState,
    pub starttime: Option<Instant>,
    pub endtime: Option<Instant>,
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

impl JamState {
    pub fn jam_score(&self) -> (u32, u32) {
        let p1j = self.team1.jammerpoints.iter().sum::<u8>();
        let p1p = self.team1.pivotpoints.iter().sum::<u8>();
        let p2j = self.team2.jammerpoints.iter().sum::<u8>();
        let p2p = self.team2.pivotpoints.iter().sum::<u8>();
        (p1j as u32 + p1p as u32, p2j as u32 + p2p as u32)
    }
    pub fn adj_score(&mut self, t1adj: i8, t2adj: i8) -> () {
        self.team1.update_points(t1adj);
        self.team2.update_points(t2adj);
    }
}
