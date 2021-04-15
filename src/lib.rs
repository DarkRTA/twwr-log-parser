use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

#[derive(Default, Debug)]
/// A parsed spoiler log. This closely matches the original spoiler log so if you need to do more
/// complex operations, consider turning the `Vec`s into something easier for your specific tool to
/// work with.
pub struct SpoilerLog {
    // we could prob use a hashmap in the form of location -> item
    pub starting_island: String,
    /// The outer Vec contains the spheres, with the lowest index being sphere 0. The inner Vec
    /// contains the locations in that sphere.
    ///
    /// `playthrough[3][6]` would be the 6th location in sphere 3.
    pub playthrough: Vec<Vec<Location>>,
    pub locations: Vec<Location>,
    pub entrances: Vec<Entrance>,
    pub charts: Vec<Chart>,
}

#[derive(Debug)]
pub struct Location {
    pub location: String,
    pub check: String,
    pub item: String,
}

#[derive(Debug)]
pub struct Entrance {
    pub source: String,
    pub destination: String,
}

#[derive(Debug)]
pub struct Chart {
    pub chart: String,
    pub location: String,
}

#[derive(PartialEq, Copy, Clone)]
enum ParserState {
    Header,
    Playthrough,
    ItemLocs,
    Entrances,
    Charts,
}

// This is not how you write rust code, and this is *definitely* not how you write a parser.
// Please do not reference this shit at all, and instead laugh at how disgusting it is.
/// Parses a spoiler log from the given path.
pub fn parse_log<P>(filename: P) -> SpoilerLog
where
    P: AsRef<Path>,
{
    let mut state = ParserState::Header;

    // mutable state is unavoidable here and needs to be outside of the loop
    let mut loc = String::default();
    let mut log = SpoilerLog::default();
    if let Ok(lines) = read_lines(filename) {
        for line in lines {
            if let Ok(line) = line {
                // determine if we should change our state
                let new_state = match line.as_str() {
                    "Playthrough:" => ParserState::Playthrough,
                    "All item locations:" => ParserState::ItemLocs,
                    "Entrances:" => ParserState::Entrances,
                    "Charts:" => ParserState::Charts,
                    _ => state,
                };
                // this is ugly but required so we skip the line
                if new_state != state {
                    state = new_state;
                    continue;
                }

                //println!("{:?}", state);
                // skip empty lines
                if line.as_str() == "" {
                    continue;
                }

                // parse the starting island
                if line.starts_with("Starting island:") {
                    let idx = line.find(":").unwrap();
                    log.starting_island = line[idx + 1..].trim().into();
                    continue;
                }

                match state {
                    ParserState::Header => (),
                    ParserState::Playthrough => {
                        if line.starts_with("      ") {
                            // check
                            let sphere = log.playthrough.len() - 1;
                            let idx = line.find(":").unwrap();
                            let check = line[..idx].trim();
                            let item = line[idx + 1..].trim();
                            log.playthrough[sphere].push(Location {
                                location: loc.clone(),
                                check: check.into(),
                                item: item.into(),
                            });
                        } else if line.starts_with("  ") {
                            // location
                            loc = line[2..line.len() - 1].to_string();
                        } else {
                            // sphere
                            // rely on the assumption that every time we get a new sphere
                            // line, we are in a new sphere. this assumption is always true
                            // for unedited spoiler logs
                            log.playthrough.push(Vec::new());
                        }
                    }
                    ParserState::ItemLocs => {
                        if line.starts_with("    ") {
                            let idx = line.find(":").unwrap();
                            let check = line[..idx].trim();
                            let item = line[idx + 1..].trim();
                            log.locations.push(Location {
                                location: loc.clone(),
                                check: check.into(),
                                item: item.into(),
                            });
                        } else {
                            loc = line[..line.len() - 1].to_string();
                        }
                    }
                    ParserState::Entrances => {
                        let idx = line.find(":").unwrap();
                        let src = line[..idx].trim();
                        let dst = line[idx + 1..].trim();
                        log.entrances.push(Entrance {
                            source: src.into(),
                            destination: dst.into(),
                        });
                    }
                    ParserState::Charts => {
                        let idx = line.find(":").unwrap();
                        let chart = line[..idx].trim();
                        let loc = line[idx + 1..].trim();
                        log.charts.push(Chart {
                            chart: chart.into(),
                            location: loc.into(),
                        });
                    }
                }
            }
        }
    }

    log
}

// this really should not be public
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
