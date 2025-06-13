#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StopId(String);

impl StopId {
    pub fn new(str: &str) -> Self {
        Self(str.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct Stop {
    pub tiploc: StopId,
    pub name: String,
    pub crs: String,
    pub min_change_time: u32,
}

impl Stop {
    pub fn new(tiploc: StopId, name: String, crs: String, min_change_time: u32) -> Self {
        Self {
            tiploc,
            name,
            crs,
            min_change_time,
        }
    }
}
