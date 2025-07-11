# GB Rail Routing

WORK IN PROGRESS

I created this project as a way to learn how to parse the CIF timetables
published by the [National Rail Open Data](https://opendata.nationalrail.co.uk/)
portal.

I parse the feeds into an internal representation that is suitable for use with
the [connection scan algorithm](https://arxiv.org/abs/1703.05997) for computing
optimal journeys on public transport. I'm currently focussed on using this to
build _isochrones_, or areas accessible within a given time frame from a given
start location but I may also add routing from point A to point B later.

## Usage

Currently, the program is just a simple CLI that loads a decompressed timetable
and then executes a query from an origin stop at a given date and time and with
a given maximum trip duration.

```
cargo run -r -- <ORIGIN> <TIMETABLE_PATH> <DATE> <TIME> <MAX_DURATION>
```

It will then print the list of accessible stations and the arrival time to
stdout.
