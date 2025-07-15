use std::{sync::Arc, time::Instant};

use actix_web::{error, get, web, App, HttpServer};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use serde::Deserialize;

use crate::{
    csa::ConnectionScan,
    timetable::{stop::StopId, Timetable},
};
mod csa;
mod timetable;

#[derive(Deserialize)]
struct Params {
    /// TIPLOC of origin station
    origin: String,
    /// Departure date
    date: NaiveDate,
    /// Departure time
    time: NaiveTime,
}

#[get("/isochrone")]
async fn isochrone(
    params: web::Query<Params>,
    csa: web::Data<Arc<ConnectionScan>>,
) -> actix_web::Result<String> {
    let origin = StopId::new(&params.origin);
    let date = params.date;
    let start_time = params.time;

    csa.departure_isochrone(origin, NaiveDateTime::new(date, start_time))
        .map_err(|err| error::ErrorBadRequest(err))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let timetable_path = std::env::args().nth(1).unwrap();

    let now = Instant::now();
    let timetable = Timetable::read(&timetable_path).unwrap();
    println!("Read timetable in {:?}", now.elapsed());

    let connection_scanner = Arc::new(ConnectionScan::new(
        timetable.trips,
        timetable.stops,
        timetable.footpaths,
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(connection_scanner.clone()))
            .service(isochrone)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
