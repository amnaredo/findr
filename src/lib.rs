use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}
#[derive(Debug)]
pub struct Config {
    dirs: Vec<String>,
    names: Option<Vec<Regex>>,
    entry_types: Option<Vec<EntryType>>,
}


pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("Alejandro Martinez <amnaredo@gmail.com>")
        .about("Rust find")
        .arg(
            Arg::with_name("dirs")
                .value_name("DIR")
                .help("Search directory")
                .default_value(".")
                .min_values(1),
        )
        .arg(
            Arg::with_name("names")
                .value_name("NAME")
                .help("Name")
                .short("n")
                .long("name")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("types")
                .value_name("TYPE")
                .help("Entry type")
                .short("t")
                .long("type")
                .possible_values(&["f", "d", "l"])
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let mut names = vec!();
    if let Some(vals) = matches.values_of_lossy("names") {
        for name in vals {
            match Regex::new(&name) {
                Ok(re) => names.push(re),
                _ => {
                    return Err(From::from(format!("Invalid --name \"{}\"", name)))
                } 
            }
        }
    }

    let entry_types = matches.values_of_lossy("types")
        .map(|vals| {
            vals.iter()
                .filter_map(|val| match val.as_str() {
                    "d" => Some(Dir),
                    "f" => Some(File),
                    "l" => Some(Link),
                    _ => None,
                })
                .collect()
        });
    
    
    Ok(Config {

        dirs: matches.values_of_lossy("dirs").unwrap(),
        names: if names.is_empty() { None } else { Some(names) },
        entry_types
    })
}

pub fn run(config: Config) -> MyResult<()> {

    let mut show_dirs = true;
    let mut show_files = true;
    let mut show_links = true;

    if config.entry_types.is_some() {
        let types_to_show = config.entry_types.unwrap();

        show_dirs = types_to_show.contains(&EntryType::Dir);
        show_files = types_to_show.contains(&EntryType::File);
        show_links = types_to_show.contains(&EntryType::Link);
    }

    for dirname in &config.dirs {
        for entry in WalkDir::new(dirname) {
            // println!("{}", entry?.path().display());
            match entry {
                Err(err) => eprintln!("{}: {}", dirname, err),
                Ok(dir_entry) => {

                    let filetype = dir_entry.path().metadata()?.file_type();
                    let type_match = 
                        filetype.is_dir() && show_dirs || 
                        filetype.is_file() && show_files || 
                        filetype.is_symlink() && show_links;

                    let mut regex_match = true;
                    if let Some(regexps) = &config.names {
                        regex_match = regexps
                            .iter()
                            .any(|regex| {
                                 regex.is_match(&dir_entry.file_name().to_string_lossy())
                            }
                        );
                    }

                    if type_match && regex_match {
                        println!("{}", dir_entry.path().display());
                    }
                }
            }
        }
    }
    Ok(())
}

    