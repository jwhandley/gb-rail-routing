use chrono::NaiveTime;

use crate::timetable::stop::StopId;

#[derive(Debug)]
pub enum Location {
    Origin {
        tiploc: StopId,
        departure_time: NaiveTime,
    },
    Intermediate {
        tiploc: StopId,
        arrival_time: NaiveTime,
        departure_time: NaiveTime,
    },
    Destination {
        tiploc: StopId,
        arrival_time: NaiveTime,
    },
}

impl Location {
    pub fn id(&self) -> StopId {
        match self {
            Location::Origin { tiploc, .. } => tiploc.clone(),
            Location::Intermediate { tiploc, .. } => tiploc.clone(),
            Location::Destination { tiploc, .. } => tiploc.clone(),
        }
    }

    pub fn departure_time(&self) -> Option<NaiveTime> {
        match self {
            Location::Origin { departure_time, .. } => Some(*departure_time),
            Location::Intermediate { departure_time, .. } => Some(*departure_time),
            Location::Destination { .. } => None,
        }
    }

    pub fn arrival_time(&self) -> Option<NaiveTime> {
        match self {
            Location::Origin { .. } => None,
            Location::Intermediate { arrival_time, .. } => Some(*arrival_time),
            Location::Destination { arrival_time, .. } => Some(*arrival_time),
        }
    }
}
