use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use crate::{
    csa::ConnectionScan,
    timetable::io::{read_mca, read_msn},
};
mod csa;
mod timetable;

fn main() -> anyhow::Result<()> {
    let stops = read_msn("../timetable/RJTTF491.MSN")?;
    let trips = read_mca("../timetable/RJTTF491.MCA")?;

    let connection_scanner = ConnectionScan::new(trips);

    let today = NaiveDate::from_ymd_opt(2025, 6, 11).unwrap();
    let time = NaiveTime::from_hms_opt(8, 30, 0).unwrap();
    let start_time = NaiveDateTime::new(today, time);

    let origin = stops[0].tiploc.clone();

    let arrival_times = connection_scanner.departure_isochrone(origin, start_time);

    for (tiploc, time) in arrival_times.into_iter() {
        println!("{tiploc:?}: {time}");
    }

    Ok(())
}
