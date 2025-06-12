use chrono::{Datelike, NaiveDate};

use crate::timetable::location::Location;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TripType {
    Permanent,
    New,
    Overlay,
    Cancellation,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TripId(String);

impl TripId {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug)]
pub struct Trip {
    pub id: TripId,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub trip_type: TripType,
    pub days_run: [bool; 7],
    pub locations: Vec<Location>,
}

impl Trip {
    pub fn new(
        id: TripId,
        start_date: NaiveDate,
        end_date: NaiveDate,
        trip_type: TripType,
        days_run: [bool; 7],
    ) -> Self {
        Self {
            id,
            start_date,
            end_date,
            trip_type,
            days_run,
            locations: vec![],
        }
    }

    pub fn runs_on(&self, date: NaiveDate) -> bool {
        self.start_date <= date
            && self.end_date >= date
            && self.days_run[date.weekday().num_days_from_monday() as usize]
    }

    pub fn add_location(&mut self, loc: Location) {
        self.locations.push(loc);
    }
}
