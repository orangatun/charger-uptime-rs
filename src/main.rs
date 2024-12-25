
use std::fs::File;
use std::io::{Lines, BufReader, BufRead,Error};
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
            if self.to<other.to {
                return Some(Ordering::Less);
            } else if self.to>other.to {
                return Some(Ordering::Greater);
            } else {
                return Some(Ordering::Equal);
            }
        } else {
            if self.from<other.from {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Greater);
            }
        }
    }
}

fn main() {

    let file_path = get_file_path();
    let (station_charger_map, charger_uptime_map, station_order) = construct_maps(&file_path);
    compute_availability(station_charger_map, charger_uptime_map, station_order);
}


/// Looks for the file path in CLI params. If not found, prints error
/// message to stderr, and exits. If found, returns file path string.
/// 
/// ### Input: This function does not have any input params
///
/// ### Output: 
/// - `file_path`: A file path String.
fn get_file_path() -> String {

    let args: Vec<String> = args().collect();
    if args.len()<2 {
        eprintln!("Missing file path parameter. Please pass a relative file path.");
        process::exit(1);
    }

    // The path to the target binary will be passed as the first argument.
    // Hence the `args[1]` here
    args[1].clone()
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
            if charger_times == None {
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
fn construct_maps(file_path: &str) -> ( HashMap<u32, HashSet<u32>>, 
                                        HashMap<u32, Vec<TimeRange>>,
                                        Vec<u32> ) {

    let mut currently_reading: InputKind = InputKind::None;
    let mut station_charger_map: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut charger_uptime_map: HashMap<u32, Vec<TimeRange>> = HashMap::new();
    let mut station_order: Vec<u32> = Vec::new();

    let lines_iterator = read_lines(&file_path);
    if Result::is_err(&lines_iterator) {
        eprintln!("File not found. Please enter a valid path to a file.");
        process::exit(2);
    }
    let lines = lines_iterator.unwrap();
    for wrapped_line in lines {
        if Result::is_err(&wrapped_line) {
            eprintln!("Failed to read line.\n{}.", wrapped_line.unwrap());
            process::exit(2);
        }
        let l = wrapped_line.unwrap();
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
                            station_order.push(station_id);
                        }

                        let charger_set: &mut HashSet<u32> = station_charger_map.get_mut(&station_id).unwrap();
                        charger_set.extend(chargers);
                    },
                    InputKind::ChargerAvailability => {
                        let charger_parse_result = parse_charger_availability(trimmed_l);
                        if let Err(charger_parse_error) = charger_parse_result {
                            eprintln!("{}", charger_parse_error);
                            process::exit(2);
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
    (station_charger_map, charger_uptime_map, station_order)
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