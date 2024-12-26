# Charger Uptime Coding Challenge

This project is for a coding challenge. The objective is to take a formatted text file with charging station availability records, and write uptime percentage for each station to standard-output (`stdout`). 

**Station Uptime** is defined as the percentage of time that any charger at a station was available, out of the entire time period that any charger *at that station* was reporting in.

## TL;DR
This section is a condensed version of the execution instructions.

To run a pre-compiled executable for `x86_64-unknown-linux-gnu` target, run the commands:
```sh
git clone https://github.com/orangatun/charger-uptime-rs.git
cd charger-uptime-rs
./charger-uptime-rs ./input.txt
```
If you don't have Rust installed, install Rust from [here](https://www.rust-lang.org/tools/install). For linux or macOS, run the command:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Assuming you have Rust installed, To build and run, use the commands: 
```sh
git clone https://github.com/orangatun/charger-uptime-rs.git
cd charger-uptime-rs
cargo build --release
mv ./target/release/debug/charger-uptime-rs .
./charger-uptime-rs ./input.txt
```
You can compare it to the expected output file `./input_expected_output.txt`.

For detailed setup, build, and execution instructions, read on. At the end, there is a section with preconditions and other considerations.

## Details of problem statement

### Input File Format

The input file should be a simple ASCII text file. The first section should be a list of station IDs that indicate the Charger IDs present at each station. The second section should be a report of each Charger ID's availability reports. An availability report should contain the Charger ID, the start time, the end time, and if the charger was "up" (i.e. available) or not. 

```
[Stations]
<Station ID 1> <Charger ID 1> <Charger ID 2> ... <Charger ID n>
...
<Station ID n> ...

[Charger Availability Reports]
<Charger ID 1> <start time nanos> <end time nanos> <up (true/false)>
<Charger ID 1> <start time nanos> <end time nanos> <up (true/false)>
...
<Charger ID 2> <start time nanos> <end time nanos> <up (true/false)>
<Charger ID 2> <start time nanos> <end time nanos> <up (true/false)>
...
<Charger ID n> <start time nanos> <end time nanos> <up (true/false)>
```


### Output Format

The output is written to `stdout`. If the input is invalid, the program will print an error message and exit. If there is no error, station availability will be printed to `stdout` as follows:

```
<Station ID 1> <Station ID 1 uptime>
<Station ID 2> <Station ID 2 uptime>
...
<Station ID n> <Station ID n uptime>
```

`Station ID n uptime` should will be an integer in the range [0-100] representing the given station's uptime percentage. The value is rounded down to the nearest percent.

Station IDs are displayed in *ascending order*.


## Project Setup

### Rust installation
This project is written in Rust, using `cargo`, Rust's package manager. To install Rust compiler, and tooling, please follow the instructions at [Rust's official website](https://www.rust-lang.org/tools/install).

## Building from source

### Clone repository
In order to build from source, clone the repository from github:

```sh
git clone https://github.com/orangatun/charger-uptime-rs.git
```

### Change into project root directory

Change into the directory `charger-uptime-rs` using the `cd charger-uptime-rs` command.

Running the `ls` command should show the `Cargo.toml` file, and the `src` directory among other files.

### Running the project
There are two ways to execute the project:
- Using Rust's Cargo project manager to build from source
- Using an executable, which is target specific.

#### Using Cargo package manager

To build the project, run the build command:
```sh
cargo build
```
The build command will install the dependencies of the project. 

To run tests, run the command
```sh
cargo test
```

To run the project using Cargo, run the following command:
```sh
cargo run relative/path/to/input/file
```
For example, to run it with the `input.txt` file provided in the repository, run the command:
```sh
cargo run ./input.txt
```

Note: By default, cargo builds and runs the project in debug mode. To build or run in release mode, use the `--release` flag:
```sh
cargo run --release ./input.txt
```

##### Building for a different target
The project compiles to the current machine architecture by default. To build for a different target, you'll require `rustup`. `rustup` comes with Rust installation. If you don't have Rust installed, please follow the instructions at [Rust's official website](https://www.rust-lang.org/tools/install).
After you have `rustup` installed, follow these steps:

1. List all targets using the command `rustup target list`. This should list all available targets. Identify a target architecture to compile to. 
2. Run the command `rustup target add <target architecture>` to add a compilation target. For example: `rustup target add x86_64-unknown-linux-gnu` adds a Linux target with amd64 architecture, with `glibc`. For a Linux target with `musl` instead of `glibc`, use `rustup target add x86_64-unknown-linux-musl`.
3. Finally, run cargo build for the custom target using `cargo build --target=<target architecture>` command. For example: `cargo build --target= x86_64-unknown-linux-gnu` builds it for a Linux target with amd64 architecture.

**Note:** I ran into issues with building for `x86_64-unknown-linux-gnu` target, and used [messense's toolchain](https://github.com/messense/homebrew-macos-cross-toolchains) to build for that target.
I was able to build for other targets without any errors, but this one must've had conflicting toolchains on my machine.

If you build without using the `--release` flag, `cargo` will build in debug mode. You'll find the executable at the location:
`./target/x86_64-unknown-linux-gnu/debug/charger-uptime-rs` relative to the project root directory.

If you run it with the `--release` flag, it'll show up in the `./target/x86_64-unknown-linux-gnu/release/charger-uptime-rs` path.

Project structure for reference:
```
.
├── src
│   └── main.rs
├── target
│   └── x86_64-unknown-linux-gnu
│       ├── debug
│       │   └── charger-uptime-rs
│       └── release
│           └── charger-uptime-rs
├── .gitignore
├── Cargo.toml
├── README.md
└── input.txt
```

#### Running an executable

To run the executable, use the following command:
```
./charger-uptime-rs relative/path/to/input/file
```

For example, in case of running the default input.txt with the project structure mentioned, running from `./target/x86_64-unknown-linux-gnu/release/` path
```
./charger-uptime-rs ../../../input.txt
```

## Preconditions and Considerations

The expectation is that the input file will follow the format:
```
[Stations]
<Station ID 1> <Charger ID 1> <Charger ID 2> ... <Charger ID n>
...
<Station ID n> ...

[Charger Availability Reports]
<Charger ID 1> <start time nanos> <end time nanos> <up (true/false)>
<Charger ID 1> <start time nanos> <end time nanos> <up (true/false)>
...
<Charger ID 2> <start time nanos> <end time nanos> <up (true/false)>
<Charger ID 2> <start time nanos> <end time nanos> <up (true/false)>
...
<Charger ID n> <start time nanos> <end time nanos> <up (true/false)>
```

Sample input:
```
[Stations]
0 1001 1002
1 1003

[Charger Availability Reports]
1001 0 50000 true
1002 50000 100000 false
1003 25000 75000 false
```

It can be split into a 'Stations' section, and a 'Charger Availability Reports' section.

### Changing order of sections

Order of 'Station' section and 'Charger Availability Reports' sections can be reversed, and the program will work the same.

For example, this would be processed as valid input.

```
[Charger Availability Reports]
1001 0 50000 true
1002 50000 100000 false
1003 25000 75000 false

[Stations]
0 1001 1002
1 1003
```

### Multiple Stations and Charger Availability sections
[Stations]
0 1001 1002

[Charger Availability Reports]
1001 0 50000 true
1002 50000 100000 false
[Stations]
1 1003
[Charger Availability Reports]
1003 25000 75000 false

### Additional spaces and newline characters
The input file can have any number of spaces, and new line characters separating entries. This is handled by the program. 

**NOTE**: The '[Charger Availability Report]' and '[Stations]' section headings must be of the same format, without any addtional spaces. '[ Stations]' will return an formatting error. 

For example, this is a valid input:
```

[Stations]
     0 1001 1002
1 1003

   [Charger Availability Reports]
1001        0 50000 true
1002 50000       100000 false
1003 25000 75000       false

```

This is invalid input:
```
[ Stations]
0 1001 1002
1 1003

[Charger Availability Reports]
1001 0 50000 true
1002 50000 100000 false
1003 25000 75000 false
```


### Station ID guarantee - unique unsigned 32-bit integer

One of the preconditions is that Station ID is guaranteed to be an unsigned 32-bit integer and unique to any other Station ID.

If the Station ID is not an unsigned 32-bit integer, the program prints an error to stderr and exits.

A non-digit character, a negative number, or a number that causes overflow for an unsigned 32-bit integer will lead to an error. In short, if it can't be parsed into a 32-bit integer, it'll display an error and exit.

The repetition of a Station ID entry does not throw an error. A repeated Station ID entry is taken as a second entry, to update it, adding more chargers to the station. 

### Charger ID guarantee - unique unsigned 32-bit integer

Another precondition is that Station ID is guaranteed to be an unsigned 32-bit integer and unique across all Station IDs.

If the Charger ID is not an unsigned 32-bit integer, the program prints an error to stderr and exits.

The repetition of a Charger ID in a Station ID entry does not throw an error. 
If a Charger ID is mapped to one Station ID and another Stations entry tries to map the same Charger ID to a different Station ID, it'll return an error and exit.

### Start-time and End-time nanos - guarantee 64-bit unsigned integers

One of the preconditions is that `start time nanos` and `end time nanos` are guaranteed to be an unsigned 64-bit integer.

If they're not an unsigned 64-bit integer, the program prints an error to stderr and exits. 


### `up` boolean set to `true` or `false`

The `up` status of a charger is expected to have a `true`/`false` value. 
A Pascal case `True` is also accepted as `true`. A `false` or a 'False' is registered as a false, and any other value returns an error and exits.

### Mulitple availability report entries
Each charger ID can have multiple availability report entries. The time range in an availability entry is taken as [start, end) where the start is inclusive, and the end is exclusive. 

For example, an entry for charger with ID 1001 with an entry with start as 2 and end as 2 has an availability duration of 0.
```
1001 2 2 true
```

### Non-contiguous report entries

The report entries need not be contiguous in time for a charger ID. A gap in reported time is counted as downtime for the charger.

### Mis-ordered charger availability report entries

There's no expectation of charger availability entries being in sorted order. Since there are multiple chargers reporting entries simultaneously, it's possible that the order in the input is not sorted.

The program sorts the entries and processes it. 

For example, this is a valid input:
```
[Stations]
0 1001 1002
1 1003

[Charger Availability Reports]
1001 50000 100000 false
1001 0 50000 true
1002 50000 100000 true
```
