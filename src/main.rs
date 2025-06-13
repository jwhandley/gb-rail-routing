use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use itertools::Itertools;

use crate::{
    csa::ConnectionScan,
    timetable::{
        io::{read_alf, read_mca, read_msn},
        stop::StopId,
    },
};
mod csa;
mod timetable;

fn main() -> anyhow::Result<()> {
    let _stops = read_msn("../timetable/RJTTF491.MSN")?;
    let trips = read_mca("../timetable/RJTTF491.MCA")?;
    let footpaths = read_alf("../timetable/RJTTF491.ALF")?;

    let origin = StopId::new("GUILDFD");

    let connection_scanner = ConnectionScan::new(trips);

    let today = NaiveDate::from_ymd_opt(2025, 6, 11).unwrap();
    let time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let start_time = NaiveDateTime::new(today, time);
    let end_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

    println!("Scanning from: {:?}", &origin);

    let arrival_times = connection_scanner.departure_isochrone(origin, start_time);

    arrival_times
        .into_iter()
        .filter(|(_, time)| *time < end_time.num_seconds_from_midnight())
        .sorted_by_key(|(_, time)| *time)
        .for_each(|(tiploc, time)| {
            println!(
                "{tiploc:?}: {}",
                NaiveTime::from_num_seconds_from_midnight_opt(time, 0).unwrap()
            );
        });

    Ok(())
}
