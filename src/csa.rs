use crate::timetable::{
    stop::StopId,
    trip::{Trip, TripId, TripType},
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;

struct Connection {
    trip_id: TripId,
    from_stop: StopId,
    to_stop: StopId,
    departure_time: NaiveTime,
    arrival_time: NaiveTime,
}

pub struct Calendar {
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

    fn runs_on(&self, trip_id: TripId, date: NaiveDate) -> bool {
        let t = &self.trips[&trip_id];
        if t.len() == 1 {
            t[0].runs_on(date)
        } else {
            assert_ne!(t[0].trip_type, TripType::Permanent);
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
            for pair in trip.locations.chunks(2) {
                let departure_time = pair[0]
                    .departure_time()
                    .expect("Should only be an origin or intermediate stop");

                let from_stop = pair[0].id();
                let to_stop = pair[1].id();
                let arrival_time = pair[1]
                    .arrival_time()
                    .expect("Should only be an intermediate or destination stop");

                connections.push(Connection {
                    trip_id: trip.id.clone(),
                    from_stop,
                    to_stop,
                    departure_time,
                    arrival_time,
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
    ) -> HashMap<StopId, NaiveTime> {
        let mut arrival_times = HashMap::new();
        arrival_times.insert(origin, start_time.time());

        let start_idx = self
            .connections
            .binary_search_by_key(&start_time.time(), |c| c.departure_time)
            .unwrap_or_else(|i| i);

        for connection in self.connections.iter().skip(start_idx) {
            todo!()
        }

        arrival_times
    }
}
