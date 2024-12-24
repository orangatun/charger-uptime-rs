use std::fs::File;
use std::io::{Lines, BufReader, BufRead,Error};

fn main() {
    let file_name = "./input.txt";
    let lines_iterator = read_lines(file_name);
    match lines_iterator {
        Ok(lines) => {
            for line in lines {
                match line {
                    Ok(l) => println!("{l}"),
                    Err(e) => panic!("{e:?}"),
                }
            }
        },
        Err(error) => panic!("Problem opening file: {error:?}"),
    };
}

/// Takes in a string reference to a file path, and returns an iterator of lines
/// ### Inputs: 
/// - `file_name`: A string reference to file path
/// 
/// ### Output:
/// - `Result<Lines, Error>`: An iterator of lines wrapped in `Ok()` if successful and and `Error` in case of error.
fn read_lines(file_name: &str) -> Result<Lines<BufReader<File>>, Error> {
    let file: File = File::open(file_name)?;
    Ok(BufReader::new(file).lines())
}