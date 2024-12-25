
use std::fs::File;
use std::io::{Lines, BufReader, BufRead, Error};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::process;
use std::cmp::Ordering;
use std::env::args;
use std::result::Result;
use std::io::ErrorKind;

enum InputKind {
    None, 
    Station,
    ChargerAvailability
}

#[derive(Clone, Debug, PartialEq, Eq, Ord)]
struct TimeRange {
    from: u64,
    to: u64,
    up: bool
}

impl PartialOrd for TimeRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.from==other.from {
            return Some(self.to.cmp(&other.to));
        } else {
            return Some(self.from.cmp(&other.from));
        }
    }
}

fn main() {

    let file_path_wrapped = get_file_path();
    if let Err(file_path_error) = file_path_wrapped {
        eprintln!("{}",file_path_error);
        process::exit(1);
    }
    let construct_map_result = construct_maps(&file_path_wrapped.unwrap());
    if let Err(construct_map_error) = construct_map_result {
        eprintln!("{}",construct_map_error);
        process::exit(2);
    }
    let (station_charger_map, charger_uptime_map, station_order) = construct_map_result.unwrap();
    compute_availability(station_charger_map, charger_uptime_map, station_order);
}


/// Looks for the file path in CLI params. If not found, prints error
/// message to stderr, and exits. If found, returns file path string.
/// 
/// ### Input: This function does not have any input params
///
/// ### Output: 
/// - `file_path`: A file path String.
fn get_file_path() -> Result<String, Error> {

    let args: Vec<String> = args().collect();
    if args.len()<2 {
        return Err(Error::new(ErrorKind::InvalidInput, format!("Missing file path parameter. Please pass a relative file path.")));
    }

    // The path to the target binary will be passed as the first argument.
    // Hence the `args[1]` here
    Ok(args[1].clone())
}

/// Takes in a string reference to a file path, and returns an iterator of lines
/// ### Input: 
/// - `file_path`: A string reference to file path
/// 
/// ### Output:
/// - `Result<Lines, Error>`: An iterator of lines wrapped in `Ok()` if successful 
/// and `Error` in case of error.
fn read_lines(file_path: &str) -> Result<Lines<BufReader<File>>, Error> {

    let file: File = File::open(file_path)?;
    Ok(BufReader::new(file).lines())
}

/// Takes in a tuple of station-charger map, charger-uptime map, 
/// and station insertion order, to compute availability percentage 
/// for each station
/// 
/// ### Input:
/// - `station_charger_map`: A map of Station ID to IDs of chargers at the station
/// - `charger_uptime_map`: A map of Charger ID to `TimeRange` structs for the charger
/// - `station_order`: A vector with list of stations in the order they appear in input
///
/// ### Output: This function does not return anything.
fn compute_availability( station_charger_map: HashMap<u32, HashSet<u32>>,
                         charger_uptime_map: HashMap<u32, Vec<TimeRange>>,
                         station_order: Vec<u32> ) {

    for i in 0..station_order.len() {
        let station_id = &station_order[i];
        let chargers = station_charger_map.get(station_id).unwrap();

        // Gathering all charger reportings of a station
        let mut station_reported_time: Vec<TimeRange> = Vec::new();
        for charger in chargers {
            let charger_times = charger_uptime_map.get(charger);
            if charger_times.is_none() {
                continue;
            }
            for charger_time in charger_times.unwrap() {
                station_reported_time.push(charger_time.clone());
            }
        }

        if station_reported_time.len()==0 {
            // No charger reported in from this station.
            // Uncomment this next line to display station as 0 percent availability
            // println!("{} {}", station_id, 0);
            continue;
        }

        // Sort in ascending order of 'from time' of availability report
        station_reported_time.sort_by(|a,b| a.cmp(&b));

        // Guaranteed to have at least one reported time at this point.
        // Initializing to the first report.
        let first_report = station_reported_time.first().unwrap();
        let first_reported_time = first_report.from;
        let mut last_reported_time = first_report.to;
        let mut available_time: u64 = 
            if first_report.up {
                first_report.to +1 -first_report.from
            } else { 
                0
            };
        let mut reported_till_time: u64 = last_reported_time;

        // Starting from the second report
        for i in 1..station_reported_time.len() {
            let charger_time: &TimeRange = &station_reported_time[i];
            if !charger_time.up {
                // charger is unavailable
                continue;
            }
            // Unavailability window in between charger reports
            if last_reported_time<charger_time.to {
                last_reported_time = charger_time.to;
            }
            // charger_time window already covered in previous window
            if reported_till_time >= charger_time.to {
                continue;
            }
            // charger_time window partial overlap in previous window
            if reported_till_time >=charger_time.from {
                available_time += charger_time.to-reported_till_time;
            } else {
                available_time += charger_time.to +1 -charger_time.from;
            }
            reported_till_time = charger_time.to;
        }
        let mut total_time: u64 = last_reported_time +1 - first_reported_time;

        if total_time>10000 {
            // Dividing by 100 to avoid overflow by multiplication
            total_time/=100;
        } else {
            // If total time is small, available time will be smaller
            // Multiplying will not cause overflow
            available_time*=100;
        }

        // Total time is guaranteed to not be zero here.
        let availability_percent: u64 = available_time/total_time;
        println!("{} {}", station_id, availability_percent);
    }
}

/// Takes in a string reference to a file path, and returns a tuple of 
/// station-charger map, charger-uptime map, and station insertion order
/// ### Input: 
/// - `file_path`: A string reference to file path
/// 
/// ### Output: A tuple consisting of
/// - `station_charger_map`: A map of Station ID to IDs of chargers at the station
/// - `charger_uptime_map`: A map of Charger ID to `TimeRange` structs for the charger
/// - `station_order`: A vector with list of stations in the order they appear in input
fn construct_maps(file_path: &str) -> Result<( HashMap<u32, HashSet<u32>>, 
                                        HashMap<u32, Vec<TimeRange>>,
                                        Vec<u32> ), Error> {

    let mut currently_reading: InputKind = InputKind::None;
    let mut station_charger_map: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut charger_uptime_map: HashMap<u32, Vec<TimeRange>> = HashMap::new();
    let mut station_order: Vec<u32> = Vec::new();

    let lines_iterator = read_lines(&file_path);
    if let Err(lines_iterator_error) = lines_iterator {
        return Err(lines_iterator_error);
    }
    let lines = lines_iterator.unwrap();
    for wrapped_line in lines {
        if let Err(line_error) = wrapped_line {
            return Err(line_error);
        }
        let l = wrapped_line.unwrap();
        match l.trim() {
            "" => {},
            "[Stations]" => currently_reading = InputKind::Station,
            "[Charger Availability Reports]" => currently_reading = InputKind::ChargerAvailability,
            trimmed_l => {
                match currently_reading {
                    InputKind::None => {
                        return Err(Error::new(ErrorKind::InvalidData, "Invalid file format. Unable to read file."));
                    },
                    InputKind::Station => {
                        let station_parse_result = parse_station(trimmed_l);
                        if let Err(station_parse_error) = station_parse_result {
                            return Err(station_parse_error);
                        }
                        let (station_id, chargers) = station_parse_result.unwrap();
                        if !station_charger_map.contains_key(&station_id) {
                            station_charger_map.insert(station_id, HashSet::new());
                            station_order.push(station_id);
                        }

                        let charger_set: &mut HashSet<u32> = station_charger_map.get_mut(&station_id).unwrap();
                        charger_set.extend(chargers);
                    },
                    InputKind::ChargerAvailability => {
                        let charger_parse_result = parse_charger_availability(trimmed_l);
                        if let Err(charger_parse_error) = charger_parse_result {
                            return Err(charger_parse_error);
                        }
                        let (charger_id, time_range) = charger_parse_result.unwrap();
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
    return Ok((station_charger_map, charger_uptime_map, station_order))
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
    if splits.len()==0 {
        return Err(Error::new(ErrorKind::InvalidData, format!("Could not parse station: \n'{}'", line)));
    }
    let station_id_str = splits.swap_remove(0);
    let station_id_wrapped = station_id_str.parse::<u32>();
    if station_id_wrapped.is_err() {
        return Err(Error::new(ErrorKind::InvalidData, format!("Invalid station ID: '{}'", station_id_str)));
    }
    let station_id = station_id_wrapped.unwrap();
    let mut chargers: Vec<u32> = Vec::new();

    while splits.len()>0 {
        let charger_id_wrapped = splits.pop().unwrap().parse::<u32>();
        if charger_id_wrapped.is_err() {
            return Err(Error::new(ErrorKind::InvalidData, format!("Invalid station entry for Station ID: {}.\nCould not parse charger ID.", station_id)));
        }
        chargers.push(charger_id_wrapped.unwrap());
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
    let re = Regex::new(r"(?<charger_id>\d+)\s+(?<start_time>\d+)\s+(?<end_time>\d+)\s*(?<up_status>\w*)").unwrap();
    let captures_wrapped = re.captures(line);
    if captures_wrapped.is_none() {
        return Err(Error::new(ErrorKind::InvalidData, "Could not parse charger availability entry. Please check the input file."));
    }
    let captures = captures_wrapped.unwrap();
    let charger_id_wrapped = captures["charger_id"].parse::<u32>();
    if charger_id_wrapped.is_err() {
        return Err(Error::new(ErrorKind::InvalidData, "Invalid charger availability entry.\nCould not parse charger ID."));
    }
    let charger_id = charger_id_wrapped.unwrap();
    let start_time_wrapped = captures["start_time"].parse::<u64>();
    if start_time_wrapped.is_err() {
        return Err(Error::new(ErrorKind::InvalidData, format!("Invalid charger availability entry.\nCould not parse start time for charger ID: {}.", charger_id)));
    }
    let end_time_wrapped = captures["end_time"].parse::<u64>();
    if end_time_wrapped.is_err() {
        return Err(Error::new(ErrorKind::InvalidData, format!("Invalid charger availability entry.\nCould not parse end time for charger ID: {}.", charger_id)));
    }
    // Note: Any input for up status that's not 'true' or 'True' will be considered as false.
    let time_range = TimeRange {
        from: start_time_wrapped.unwrap(),
        to: end_time_wrapped.unwrap(),
        up: match &captures["up_status"] {
            "true" | "True" => true,
            _ => false,
        },
    };
    if time_range.from>time_range.to {
        return Err(Error::new(ErrorKind::InvalidData, format!("Invalid charger availability entry for charger ID {}!\nAvailability from is after availability to.", charger_id)));
    }
    Ok((charger_id, time_range))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_station_test_valid() {
        let station_string = "1 1001 1002";
        let chargers_vec: Vec<u32> = vec![1001, 1002];
        let station_id: u32 = 1;
        let parse_output = parse_station(&station_string);
        assert!(!parse_output.is_err());
        let (station_id_parsed, chargers_parsed) = parse_output.unwrap();
        assert_eq!(station_id, station_id_parsed); 
        assert_eq!(chargers_vec, chargers_parsed);
    }
}