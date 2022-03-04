/*
 * Copyright (c) 2022, Patrick Wilmes <patrick.wilmes@bit-lake.com>
 *
 * SPDX-License-Identifier: BSD-2-Clause
 */
extern crate core;

use std::{env};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
/*
./adr init docs/architecture/
./adr new "some title"
./adr list
 */
fn main() {
    let args: Vec<String> = env::args().collect();
    match validate_command_line_args(&args) {
        Ok(command) => handle_command(command),
        Err(_) => panic!(),
    }
}

#[derive(Debug)]
enum ArgumentError {
    InvalidCommandError,
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgumentError::InvalidCommandError => write!(f, "The first argument must be a command like [list|new|init]"),
        }
    }
}

impl std::error::Error for ArgumentError {}

#[derive(Debug)]
enum Command {
    Init { path: String },
    New { title: String },
    List,
}

fn validate_command_line_args(args: &Vec<String>) -> Result<Command, ArgumentError> {
    /*
    The first argument 0 is the program name, so if we want to access the command
    we have to access the index 1.
     */
    let possible_command = args.get(1);
    match possible_command {
        None => Err(ArgumentError::InvalidCommandError),
        Some(command) => {
            let possible_arg = args.get(2);
            match possible_arg {
                None => {
                    if command != "list" {
                        Err(ArgumentError::InvalidCommandError)
                    } else {
                        Ok(Command::List)
                    }
                }
                Some(arg) => {
                    if command == "new" {
                        Ok(Command::New { title: arg.to_owned() })
                    } else {
                        Ok(Command::Init { path: arg.to_owned() })
                    }
                }
            }
        }
    }
}

fn handle_command(command: Command) {
    match command {
        Command::Init { path } => {
            if adr_file_handling::adr_dir_file_exists_in_wd() {
                panic!("Wow adr_file found! You're already tracking adrs!")
            }
            adr_file_handling::create_adr_dir_file(&path).expect("Unable to create adr_file");
            file_system_ops::create_directory_structure(&path);
            file_system_ops::move_init_md_to_target_dir(&path);
        }
        Command::New { title } => {
            let location = adr_file_handling::get_location_from_adr_file();
            let adr_count = file_system_ops::count_adrs_except_init_md(&location);
            let new_adr_number = adr_count + 1;
            let middle_filename_part = title.to_lowercase().replace(" ", "_");
            let mut filename = String::from(location.clone());
            filename.push_str(String::from("/").as_str());
            filename.push_str(new_adr_number.to_string().as_str());
            filename.push_str("_");
            filename.push_str(middle_filename_part.as_str());
            filename.push_str(".md");
            file_system_ops::create_adr_from_template_at_location(&filename);
        }
        Command::List => {
            let location = adr_file_handling::get_location_from_adr_file();
            file_system_ops::list_all_adr_files_at_location(&location);
        }
    }
}

mod adr_file_handling {
    use super::*;

    const ADR_FILE_NAME: &str = ".adr_file";

    pub fn adr_dir_file_exists_in_wd() -> bool {
        Path::new(ADR_FILE_NAME).exists()
    }

    /*
    We need to keep track where the adrs are stored, so we are going
    to write the adr location into a file called .adr_dir.
     */
    pub fn create_adr_dir_file(location: &String) -> std::io::Result<()> {
        let mut file = File::create(ADR_FILE_NAME)?;
        file.write_all(location.as_bytes()).expect("Unable to write location to .adr_file");
        Ok(())
    }

    pub fn get_location_from_adr_file() -> String {
        let mut file = File::open(ADR_FILE_NAME).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read .adr_file");
        contents
    }
}

mod file_system_ops {
    use std::fs;
    use std::fs::{copy, create_dir_all};
    use std::path::Path;

    pub fn create_directory_structure(location: &String) {
        match create_dir_all(Path::new(location)) {
            Ok(_) => {}
            Err(e) => eprintln!("{:?}", e),
        }
    }

    pub fn move_init_md_to_target_dir(location: &String) {
        let mut path_to_init_md = location.clone();
        path_to_init_md.push_str("/init.md");
        match copy(Path::new("resources/init.md"), Path::new(path_to_init_md.as_str())) {
            Ok(_) => {}
            Err(e) => eprintln!("{:?}", e),
        }
    }

    pub fn count_adrs_except_init_md(location: &String) -> usize {
        get_all_adr_files_at_location(location).len()
    }

    pub fn list_all_adr_files_at_location(location: &String) {
        for file in get_all_adr_files_at_location(location) {
            println!("{}", file);
        }
    }

    pub fn create_adr_from_template_at_location(filename: &String) {
        match copy(Path::new("resources/template.md"), Path::new(filename)) {
            Ok(_) => {}
            Err(e) => eprintln!("{:?}", e),
        }
    }

    pub fn get_all_adr_files_at_location(location: &String) -> Vec<String> {
        let mut adr_files :Vec<String> = Vec::new();
        for entry in fs::read_dir(location).unwrap() {
            let file_or_dir = entry.unwrap();
            if fs::metadata(file_or_dir.path()).unwrap().is_file() {
                if file_or_dir.file_name().to_str().unwrap() != "init.md" {
                    adr_files.push(String::from(file_or_dir.file_name().to_str().unwrap()));
                }
            }
        }
        adr_files
    }
}
