use chrono;
use std::time::Duration;
use rocket::request::FromFormValue;
use rocket::http::RawStr;


pub enum TimeType { TimeToDerby, StartAt }

impl<'a> FromFormValue<'a> for TimeType {
    type Error = &'a str;
    fn from_form_value(v: &'a RawStr) -> Result<Self, Self::Error> {
        match v.as_bytes() {
            b"1" => Ok(TimeType::StartAt),
            b"2" => Ok(TimeType::TimeToDerby),
            _ => Err(v),
        }
    }
}

#[derive(Clone,Copy)]
pub enum TimeAMPM { AM, PM, None }

impl<'a> FromFormValue<'a> for TimeAMPM {
    type Error = &'a str;
    fn from_form_value(v: &'a RawStr) -> Result<Self, Self::Error> {
        match v.as_bytes() {
            b"AM" => Ok(TimeAMPM::AM),
            b"PM" => Ok(TimeAMPM::PM),
            b"" => Ok(TimeAMPM::None),
            _ => Err(v),
        }
    }
}

pub fn start_at_time(at_hrs: u8, at_mins: u8, at_ampm: TimeAMPM) -> Result<Duration, &'static str> {
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
    let when = chrono::naive::NaiveTime::from_hms(real_hrs as u32, at_mins as u32, 0);
    let duration = if now < when {
        when.signed_duration_since(now)
    } else {
        when.signed_duration_since(now) + chrono::Duration::hours(24)
    };
    duration.to_std().map_err(|_| "negative duration?!")
}

