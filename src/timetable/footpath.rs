use anyhow::{anyhow, Context};

enum Mode {
    Bus,
    Tube,
    Walk,
    Ferry,
    Metro,
    Tram,
    Transfer,
}

pub struct Footpath {
    from_crs: String,
    to_crs: String,
    mode: Mode,
    time: u32,
}

impl Footpath {
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let entries = s.split(',').collect::<Vec<_>>();

        let mode = match entries[0].split_once('=').expect("Should be a K=V pair").1 {
            "BUS" => Ok(Mode::Bus),
            "TUBE" => Ok(Mode::Tube),
            "WALK" => Ok(Mode::Walk),
            "FERRY" => Ok(Mode::Ferry),
            "METRO" => Ok(Mode::Metro),
            "TRAM" => Ok(Mode::Tram),
            "TRANSFER" => Ok(Mode::Transfer),
            _ => Err(anyhow!("Invalid mode")),
        }?;

        let from_crs = entries[1]
            .split_once('=')
            .context("Should be a K=V pair")?
            .1
            .to_owned();
        let to_crs = entries[2]
            .split_once('=')
            .context("Should be a K=V pair")?
            .1
            .to_owned();
        let time = entries[3]
            .split_once('=')
            .context("Should be a K=V pair")?
            .1
            .parse::<u32>();

        time.map(|t| Footpath {
            from_crs,
            to_crs,
            mode,
            time: t,
        })
        .context("Failed to parse time")
    }
}
