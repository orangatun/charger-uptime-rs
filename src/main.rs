
use std::fs::File;
use std::io::{Lines, BufReader, BufRead,Error};
use std::collections::{HashMap, HashSet};
use regex::Regex;

enum InputKind {
    None, 
    Station,
    ChargerAvailability
}

struct TimeRange {
    from: u64,
    to: u64,
    up: bool
}

fn main() {

    let file_name = "./input.txt";
    let lines_iterator = read_lines(file_name);

    let mut currently_reading: InputKind = InputKind::None;
    let mut station_charger_map: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut charger_uptime_map: HashMap<u32, Vec<TimeRange>> = HashMap::new();
    
    match lines_iterator {
        Ok(lines) => {
            for line in lines {
                match line {
                    Ok(l) => {
                        match l.trim() {
                            "" => {},
                            "[Stations]" => currently_reading = InputKind::Station,
                            "[Charger Availability Reports]" => currently_reading = InputKind::ChargerAvailability,
                            trimmed_l => {
                                match currently_reading {
                                    InputKind::None => panic!("Unable to parse section"),
                                    InputKind::Station => {
                                        let (station_id, chargers) = parse_station(trimmed_l).unwrap();
                                        
                                        if !station_charger_map.contains_key(&station_id) {
                                            station_charger_map.insert(station_id, HashSet::new());
                                        }

                                        let charger_set: &mut HashSet<u32> = station_charger_map.get_mut(&station_id).unwrap();
                                        charger_set.extend(chargers);
                                    },
                                    InputKind::ChargerAvailability => {
                                        let (charger_id, time_range) = parse_charger_availability(trimmed_l).unwrap();
                                        if !charger_uptime_map.contains_key(&charger_id) {
                                            let mut uptime_ranges: Vec<TimeRange> = Vec::new();
                                            uptime_ranges.push(time_range);
                                            charger_uptime_map.insert(charger_id, uptime_ranges);
                                        } else {
                                            let uptime_ranges: &mut Vec<TimeRange> = charger_uptime_map.get_mut(&charger_id).unwrap();
                                            uptime_ranges.push(time_range);
                                        }
                                    },
                                }
                            }
                        }
                    }
                    Err(e) => panic!("{e:?}"),
                }
            }
        },
        Err(error) => panic!("Problem opening file: {error:?}"),
    };
}

/// Takes in a string reference to a file path, and returns an iterator of lines
/// ### Input: 
/// - `file_name`: A string reference to file path
/// 
/// ### Output:
/// - `Result<Lines, Error>`: An iterator of lines wrapped in `Ok()` if successful 
/// and `Error` in case of error.
fn read_lines(file_name: &str) -> Result<Lines<BufReader<File>>, Error> {

    let file: File = File::open(file_name)?;
    Ok(BufReader::new(file).lines())
}

/// Parses a line of station info and returns it wrapped in a `Result()`.
/// ### Input :
/// - `line`: A string reference containing station id and ids of chargers at a station.
/// Expected format of `line`:
/// <Station ID 1> <Charger ID 1> <Charger ID 2> ... <Charger ID n>
///
/// ### Output:
/// - `Result<(Station ID, Vec<Charger IDs>), Error>`: A tuple of station id and a vector 
/// of charger ids wrapped in `Ok()` if successful and `Error` in case of error. 
fn parse_station(line: &str) -> Result<(u32, Vec<u32>), Error> {

    let re = Regex::new(r"\s+").unwrap();
    let mut splits: Vec<&str> = re.split(line).collect();
    let station_id = splits.swap_remove(0).parse::<u32>().unwrap();
    let mut chargers: Vec<u32> = Vec::new();

    while splits.len()>0 {
        chargers.push(splits.pop().unwrap().parse::<u32>().unwrap());
    }
    Ok((station_id, chargers))

}

/// Parses a line of charger availability info and returns it wrapped in a `Result()`.
/// ### Input :
/// - `line`: A string reference containing charger id, start time, end time,
/// and up/down status of charger.
/// Expected format of `line`:
/// <Charger ID 1> <start time nanos> <end time nanos> <up (true/false)>
///
/// ### Output:
/// - `Result<(Charger ID, TimeRange struct), Error>`: A tuple of station id and a struct 
/// `TimeRange` wrapped in `Ok()` if successful and `Error` in case of error. 
/// The `TimeRange` struct contains parsed start time, end time, and up/down status of charger. 
fn parse_charger_availability(line: &str) -> Result<(u32, TimeRange), Error> {
    let re = Regex::new(r"(?<charger_id>\d+)\s+(?<start_time>\d+)\s+(?<end_time>\d+)\s*(?<up_status>\w+)").unwrap();
    let captures = re.captures(line).unwrap();
    let charger_id: u32 = captures["charger_id"].parse::<u32>().unwrap();
    let time_range = TimeRange {
        from: captures["start_time"].parse::<u64>().unwrap(),
        to: captures["end_time"].parse::<u64>().unwrap(),
        up: match &captures["up_status"] {
            "true" | "True" => true,
            _ => false,
        },
    };
    Ok((charger_id, time_range))
}