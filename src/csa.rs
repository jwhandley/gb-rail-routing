use crate::timetable::{
    footpath::Footpath,
    stop::{Stop, StopId},
    trip::{Trip, TripId, TripType},
};
use anyhow::{anyhow, Context};
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use geo_types::Point;
use itertools::Itertools;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

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
            return t[0].runs_on(date);
        }

        match t[1].trip_type {
            TripType::Cancellation => false,
            _ => t[1].runs_on(date),
        }
    }
}

#[derive(Debug)]
struct Transfer {
    from_stop: StopId,
    to_stop: StopId,
    min_transfer_time: u32,
}

#[derive(Serialize)]
struct ArrivalTime {
    id: StopId,
    name: String,
    #[serde(serialize_with = "geojson::ser::serialize_geometry")]
    geometry: Point,
    arrival_time: u32,
}

pub struct ConnectionScan {
    stops: HashMap<StopId, Stop>,
    transfers: HashMap<StopId, Vec<Transfer>>,
    connections: Vec<Connection>,
    calendar: Calendar,
}

impl ConnectionScan {
    pub fn new(trips: Vec<Trip>, stops: Vec<Stop>, pathways: Vec<Footpath>) -> Self {
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

        let stop_map: HashMap<String, &Stop> = stops.iter().map(|s| (s.crs.clone(), s)).collect();

        let transfers = pathways
            .iter()
            .filter_map(|p| {
                let from_stop = stop_map.get(&p.from_crs).map(|s| s.tiploc.clone());

                let to_stop = stop_map.get(&p.to_crs).map(|s| s.tiploc.clone());

                match (from_stop, to_stop) {
                    (Some(from_stop), Some(to_stop)) => Some(Transfer {
                        from_stop,
                        to_stop,
                        min_transfer_time: p.time,
                    }),
                    _ => None,
                }
            })
            .into_group_map_by(|t| t.from_stop.clone());

        Self {
            stops: stops.into_iter().map(|s| (s.tiploc.clone(), s)).collect(),
            transfers,
            connections,
            calendar,
        }
    }

    fn get_transfers(&self, stop: &StopId) -> impl Iterator<Item = &Transfer> {
        match self.transfers.get(stop) {
            Some(transfers) => transfers.iter(),
            None => [].iter(),
        }
    }

    pub fn departure_isochrone(
        &self,
        origin: StopId,
        start_time: NaiveDateTime,
    ) -> anyhow::Result<String> {
        if !self.stops.contains_key(&origin) {
            return Err(anyhow!("Invalid stop id"));
        }

        let time = start_time.time();
        let date = start_time.date();

        let mut trips_set = HashSet::new();
        let mut arrival_times: HashMap<StopId, u32> = HashMap::new();

        arrival_times.insert(origin.clone(), time.num_seconds_from_midnight());

        let start_idx = self
            .connections
            .binary_search_by_key(&time.num_seconds_from_midnight(), |c| c.departure_time)
            .unwrap_or_else(|i| i);

        for c in self.connections.iter().skip(start_idx - 1) {
            if !self.calendar.runs_on(&c.trip_id, date) {
                continue;
            }

            let min_change_time = self
                .stops
                .get(&c.from_stop)
                .map(|s| {
                    if s.tiploc == origin {
                        0
                    } else {
                        s.min_change_time * 60
                    }
                })
                .unwrap_or(0);

            let from_stop_arrival = arrival_times.get(&c.from_stop).copied().unwrap_or(u32::MAX);
            let already_boarded = trips_set.contains(&c.trip_id);
            let can_board = from_stop_arrival <= c.departure_time - min_change_time;

            if can_board || already_boarded {
                trips_set.insert(c.trip_id.clone());

                let to_stop_arrival = arrival_times.get(&c.to_stop).copied().unwrap_or(u32::MAX);

                if c.arrival_time < to_stop_arrival {
                    arrival_times.insert(c.to_stop.clone(), c.arrival_time);

                    for transfer in self.get_transfers(&c.to_stop) {
                        let new_time: u32 = c.arrival_time + transfer.min_transfer_time;
                        let current_time = arrival_times
                            .get(&transfer.to_stop)
                            .copied()
                            .unwrap_or(u32::MAX);

                        if new_time < current_time {
                            arrival_times.insert(transfer.to_stop.clone(), new_time);
                        }
                    }
                }
            }
        }

        let times: Vec<ArrivalTime> = arrival_times
            .into_iter()
            .filter(|(id, _)| self.stops.contains_key(&id))
            .map(|(id, arrival)| {
                let stop = &self.stops[&id];

                ArrivalTime {
                    id: id.clone(),
                    name: stop.name.clone(),
                    geometry: stop.coord.unwrap_or_default(),
                    arrival_time: arrival,
                }
            })
            .collect();

        geojson::ser::to_feature_collection_string(&times).context("Failed to serialize")
    }
}
