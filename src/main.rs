use std::time::Instant;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use clap::Parser;
use itertools::Itertools;

use crate::{
    csa::ConnectionScan,
    timetable::{stop::StopId, Timetable},
};
mod csa;
mod timetable;

#[derive(Parser)]
struct Args {
    /// TIPLOC of origin station
    origin: String,
    /// Path to timetable file
    timetable_path: String,
    /// Departure date
    date: NaiveDate,
    /// Departure time
    time: NaiveTime,
    /// Max trip duration
    max_duration: i64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let now = Instant::now();
    let timetable = Timetable::read(&args.timetable_path)?;
    println!("Read timetable in {:?}", now.elapsed());

    let origin = StopId::new(&args.origin);
    let connection_scanner =
        ConnectionScan::new(timetable.trips, timetable.stops, timetable.footpaths);

    let date = args.date;
    let start_time = args.time;
    let end_time = start_time + TimeDelta::minutes(args.max_duration);

    println!(
        "Finding stops accessible from {:?} starting at {start_time} arriving by {end_time}",
        &origin
    );

    let now = Instant::now();
    let arrival_times =
        connection_scanner.departure_isochrone(origin, NaiveDateTime::new(date, start_time));
    println!("Found stops in {:?}", now.elapsed());

    arrival_times
        .into_iter()
        .filter(|(_, time)| *time < end_time.num_seconds_from_midnight())
        .sorted_by_key(|(_, time)| *time)
        .for_each(|(tiploc, t)| {
            println!(
                "Can reach {tiploc:?} at {:?}",
                NaiveTime::from_num_seconds_from_midnight_opt(t, 0).unwrap()
            );
        });

    Ok(())
}
