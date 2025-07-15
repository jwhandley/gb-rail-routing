use geo_types::Point;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct StopId(String);

impl StopId {
    pub fn new(str: &str) -> Self {
        Self(str.to_owned())
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Stop {
    pub tiploc: StopId,
    pub name: String,
    pub crs: String,
    pub min_change_time: u32,
    pub coord: Option<Point>,
}

impl Stop {
    pub fn new(
        tiploc: StopId,
        name: String,
        crs: String,
        coord: Option<Point>,
        min_change_time: u32,
    ) -> Self {
        Self {
            tiploc,
            name,
            crs,
            coord,
            min_change_time,
        }
    }
}
