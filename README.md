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

Currently, the program is just a simple web API that loads a decompressed
timetable and then exposes an endpoint that allows you to query which stops are
accesible from an origin stop at a given date and time

To start the server:

```
cargo run -r -- <TIMETABLE_PATH>
```

To execute a query (assuming the server is running locally):

```
curl "http://localhost:8080/isochrone?origin={stop_id}&date={yyyy-mm-dd}&time={hh:mm:ss}"
```
