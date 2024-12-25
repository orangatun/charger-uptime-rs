
use std::fs::File;
use std::io::{Lines, BufReader, BufRead,Error};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use std::process;
use std::cmp::Ordering;
use std::env::args;

enum InputKind {
    None, 
    Station,
    ChargerAvailability
}

#[derive(Clone, Debug)]
struct TimeRange {
    from: u64,
    to: u64,
    up: bool
}

fn main() {

    let args: Vec<String> = args().collect();
    if args.len()==0 {
        eprintln!("Missing file path parameter. Please pass a relative file path.");
        process::exit(1);
    }

    // The path to the target binary will be passed as the first argument.
    // Hence the `args[1]` here
    let file_name = &args[1];
    let lines_iterator = read_lines(file_name);

    let mut currently_reading: InputKind = InputKind::None;
    let mut station_charger_map: HashMap<u32, HashSet<u32>> = HashMap::new();
    let mut charger_uptime_map: HashMap<u32, Vec<TimeRange>> = HashMap::new();
    let mut station_order: Vec<u32> = Vec::new();

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
                                            station_order.push(station_id);
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
    
    for i in 0..station_order.len() {
        let station_id = &station_order[i];
        let chargers = station_charger_map.get(station_id).unwrap();
        let mut station_available_time: Vec<TimeRange> = Vec::new();
        for charger in chargers {
            let charger_times: &Vec<TimeRange> = charger_uptime_map.get(charger).unwrap();
            // .iter().filter(|a| a.up).cloned().collect();
            for charger_time in charger_times {
                if charger_time.up {
                    station_available_time.push(charger_time.clone());
                }
            }
        }

        if station_available_time.len()==0 {
            println!("{} {}", station_id, 0);
            continue;
        }

        station_available_time.sort_by(|a,b| {
            if a.from==b.from {
                if a.to<b.to {
                    return Ordering::Less;
                } else if a.to>b.to {
                    return Ordering::Greater;
                } else {
                    return Ordering::Equal;
                }
            } else {
                if a.from<b.from {
                    return Ordering::Less;
                } else {
                    return Ordering::Greater;
                }
            }
            // a.from==b.from?(a.to<b.to?-1:1):(a.from<b.from?-1:1)
        });

        let first_charger_up_time: u64 = station_available_time[0].from;
        let mut unavailable_time : u64 = 0;
        let mut available_till: u64 = station_available_time[0].from;
        for i in 0..station_available_time.len() {
            let charger_time: &TimeRange = &station_available_time[i];
            if charger_time.from>available_till {
                unavailable_time+=charger_time.from - available_till;
            }
            if available_till<charger_time.to {
                available_till = charger_time.to;
            }
        }
        let mut total_time: u64 = available_till-first_charger_up_time;
        let mut available_time: u64 = total_time-unavailable_time;
        if total_time>10000 {
            total_time/=100;
        } else {
            available_time*=100;
        }
        let availability_percent: u64 = available_time/total_time;
        println!("{} {}", station_id, availability_percent);
    }
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