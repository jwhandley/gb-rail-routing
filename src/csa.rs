use crate::timetable::{
    stop::StopId,
    trip::{Trip, TripId, TripType},
};
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    u32,
};

#[derive(Debug)]
struct Connection {
    trip_id: TripId,
    from_stop: StopId,
    to_stop: StopId,
    departure_time: u32,
    arrival_time: u32,
}

struct Calendar {
    trips: HashMap<TripId, Vec<Trip>>,
}

impl Calendar {
    fn new(trips: Vec<Trip>) -> Self {
        let mut lookup: HashMap<TripId, Vec<Trip>> = HashMap::new();

        for t in trips.into_iter() {
            match lookup.get_mut(&t.id) {
                Some(trips) => trips.push(t),
                None => {
                    lookup.insert(t.id.clone(), vec![t]);
                }
            }
        }

        for (_, v) in lookup.iter_mut() {
            v.sort_by_key(|t| t.trip_type);
        }

        Self { trips: lookup }
    }

    fn runs_on(&self, trip_id: &TripId, date: NaiveDate) -> bool {
        let t = &self.trips[trip_id];
        if t.len() == 1 {
            t[0].runs_on(date)
        } else {
            // assert_eq!(t[0].trip_type, TripType::Permanent);
            match t[1].trip_type {
                TripType::Cancellation => false,
                _ => t[1].runs_on(date),
            }
        }
    }
}

pub struct ConnectionScan {
    connections: Vec<Connection>,
    calendar: Calendar,
}

impl ConnectionScan {
    pub fn new(trips: Vec<Trip>) -> Self {
        let mut connections = vec![];
        for trip in trips.iter() {
            for (from, to) in trip.locations.iter().tuple_windows() {
                let departure_time = from
                    .departure_time()
                    .expect("Should only be an origin or intermediate stop");

                let from_stop = from.id();
                let to_stop = to.id();
                let arrival_time = to
                    .arrival_time()
                    .expect("Should only be an intermediate or destination stop");

                let arrival_secs = if arrival_time < departure_time {
                    arrival_time.num_seconds_from_midnight() + 24 * 3600
                } else {
                    arrival_time.num_seconds_from_midnight()
                };

                connections.push(Connection {
                    trip_id: trip.id.clone(),
                    from_stop,
                    to_stop,
                    departure_time: departure_time.num_seconds_from_midnight(),
                    arrival_time: arrival_secs,
                });
            }
        }

        connections.sort_by_key(|c| c.departure_time);

        let calendar = Calendar::new(trips);

        Self {
            connections,
            calendar,
        }
    }

    pub fn departure_isochrone(
        &self,
        origin: StopId,
        start_time: NaiveDateTime,
    ) -> HashMap<StopId, u32> {
        let time = start_time.time();
        let date = start_time.date();

        let mut trips_set = HashSet::new();
        let mut arrival_times: HashMap<StopId, u32> = HashMap::new();

        arrival_times.insert(origin, time.num_seconds_from_midnight());

        let start_idx = self
            .connections
            .binary_search_by_key(&time.num_seconds_from_midnight(), |c| c.departure_time)
            .unwrap_or_else(|i| i);

        for c in self.connections.iter().skip(start_idx - 1) {
            if !self.calendar.runs_on(&c.trip_id, date) {
                continue;
            }

            let from_stop_arrival = arrival_times.get(&c.from_stop).copied().unwrap_or(u32::MAX);
            let already_boarded = trips_set.contains(&c.trip_id);
            let can_board = from_stop_arrival <= c.departure_time;

            if can_board || already_boarded {
                trips_set.insert(c.trip_id.clone());

                let to_stop_arrival = arrival_times.get(&c.to_stop).copied().unwrap_or(u32::MAX);

                if c.arrival_time < to_stop_arrival {
                    arrival_times.insert(c.to_stop.clone(), c.arrival_time);
                }
            }
        }

        arrival_times
    }
}
