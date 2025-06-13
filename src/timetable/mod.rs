pub mod footpath;
pub mod location;
pub mod stop;
pub mod trip;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{NaiveDate, NaiveTime};

use crate::timetable::{
    footpath::Footpath,
    location::Location,
    stop::{Stop, StopId},
    trip::{Trip, TripId, TripType},
};

fn find_first_file_with_extension<P: AsRef<Path>>(dir: P, extension: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext| ext.to_ascii_lowercase() == extension)
        })
}

pub struct Timetable {
    pub stops: Vec<Stop>,
    pub trips: Vec<Trip>,
    pub footpaths: Vec<Footpath>,
}

impl Timetable {
    pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let msn_path = find_first_file_with_extension(&path, "msn")
            .context("Timetable must have .MSN file")?;
        let mca_path = find_first_file_with_extension(&path, "mca")
            .context("Timetable must have .MCA file")?;
        let alf_path = find_first_file_with_extension(&path, "alf")
            .context("Timetable must have .ALF file")?;

        let stops = read_msn(msn_path)?;
        let trips = read_mca(mca_path)?;
        let footpaths = read_alf(alf_path)?;

        Ok(Self {
            stops,
            trips,
            footpaths,
        })
    }
}

fn read_msn<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Stop>> {
    let f = File::open(path)?;
    let rdr = BufReader::new(f);

    let mut stops = vec![];
    for l in rdr.lines() {
        let line = l?;

        // Skip comments and header
        if line.starts_with('/') || line.contains("FILE-SPEC=05") {
            continue;
        }

        // Ignore aliases and below
        if line.starts_with('L') {
            break;
        }

        let name = line[5..31].trim().to_owned();
        let tiploc = StopId::new(line[36..43].trim());
        let crs = line[49..52].to_owned();

        let min_change_time = line[64..65].parse::<u32>()?;

        stops.push(Stop::new(tiploc, name, crs, min_change_time));
    }

    Ok(stops)
}

fn read_alf<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Footpath>> {
    let f = File::open(path)?;
    let rdr = BufReader::new(f);

    let mut footpaths = vec![];
    for l in rdr.lines() {
        let line = l?;

        let path = Footpath::parse(&line)?;
        footpaths.push(path);
    }

    Ok(footpaths)
}

fn read_mca<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<Trip>> {
    let f = File::open(path)?;
    let rdr = BufReader::new(f);

    let mut trips = vec![];
    let mut current_trip = None;

    for l in rdr.lines() {
        let line = l?;

        if line.starts_with("HD") || line.starts_with("TI") || line.starts_with("AA") {
            continue;
        }

        // Start of a trip
        if line.starts_with("BS") {
            let trip_id = TripId::new(line[3..9].to_owned());
            let start_date = NaiveDate::parse_from_str(&line[9..15], "%y%m%d")?;
            let end_date = NaiveDate::parse_from_str(&line[15..21], "%y%m%d")?;
            let trip_type = match line.chars().last() {
                Some('P') => TripType::Permanent,
                Some('O') => TripType::Overlay,
                Some('N') => TripType::New,
                Some('C') => TripType::Cancellation,
                _ => panic!("Unexpected character at end of line: {}", line),
            };

            let mut days_run = [false; 7];
            line[21..28].char_indices().for_each(|(i, d)| {
                if d == '1' {
                    days_run[i] = true
                }
            });

            current_trip = Some(Trip::new(
                trip_id, start_date, end_date, trip_type, days_run,
            ));
        } else if line.starts_with("LO") {
            let tiploc = StopId::new(line[2..10].trim());
            let departure_time = NaiveTime::parse_from_str(&line[10..14], "%H%M")?;

            let loc = Location::Origin {
                tiploc,
                departure_time,
            };

            if let Some(current_trip) = current_trip.as_mut() {
                current_trip.add_location(loc);
            }
        } else if line.starts_with("LI") {
            let tiploc = StopId::new(line[2..10].trim());
            let arrival_time = NaiveTime::parse_from_str(&line[10..14], "%H%M");
            let departure_time = NaiveTime::parse_from_str(&line[15..19], "%H%M");

            if let (Ok(departure_time), Ok(arrival_time)) = (departure_time, arrival_time) {
                let loc = Location::Intermediate {
                    tiploc,
                    arrival_time,
                    departure_time,
                };

                if let Some(current_trip) = current_trip.as_mut() {
                    current_trip.add_location(loc);
                }
            }
        } else if line.starts_with("LT") {
            let tiploc = StopId::new(line[2..10].trim());
            let arrival_time = NaiveTime::parse_from_str(&line[10..14], "%H%M")?;

            let loc = Location::Destination {
                tiploc,
                arrival_time,
            };

            if let Some(current_trip) = current_trip.as_mut() {
                current_trip.add_location(loc);
            }

            if let Some(trip) = current_trip.take() {
                trips.push(trip);
            }

            current_trip = None;
        }
    }

    Ok(trips)
}
