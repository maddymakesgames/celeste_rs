use std::{
    fmt::Display,
    fs::OpenOptions,
    io::{self, BufReader, Write},
    path::Path,
};

use celeste_rs::saves::def::SaveData;

fn main() {
    let mut args = std::env::args().skip(1);

    let command: String;
    let mut arguments = Vec::new();
    let mut verbose = false;
    let mut raw_arg = args.next();

    loop {
        if let Some(arg) = raw_arg {
            if let Some(opt) = arg.strip_prefix("--") {
                match opt {
                    "verbose" => verbose = true,
                    _ => println!("Unknown option '{opt}'"),
                }
            } else {
                command = arg;
                break;
            }
            raw_arg = args.next();
        } else {
            print_help();
            return;
        }
    }

    for arg in args {
        if let Some(opt) = arg.strip_prefix("--")
            && "verbose" == opt
        {
            verbose = true
        } else {
            arguments.push(arg);
        }
    }

    match command.as_str() {
        "merge" => merge_saves(arguments, verbose),
        "stats" => print_stats(arguments, verbose),
        "clears" => print_clears(arguments, verbose),
        _ => {
            print_help();
            Some(())
        }
    };
}

fn print_help() {
    println!(
        r#"-- Celeste Save Editor CLI --
Version: 0.1
Usage: 
celeste_cli merge [save_file_path] [save_file_2_path] [output_path]
celeste_cli stats [save_file_path] (sid_filter)"#
    );
}

fn merge_saves(args: Vec<String>, verbose: bool) -> Option<()> {
    if args.len() < 3 {
        println!("Too few arguments provided to merge");
        return None;
    }

    let mut save_a = load_save(&args[0], verbose)?;
    let save_b = load_save(&args[1], verbose)?;

    save_a.merge_data(&save_b);

    let out_path = &args[2];
    let mut out_file = match OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(out_path)
    {
        Ok(f) => f,
        Err(ref e) => {
            match e.kind() {
                io::ErrorKind::NotFound => println!("The path '{out_path}' is not found."),
                io::ErrorKind::PermissionDenied => {
                    println!("You do not have permissions to access the path '{out_path}'.")
                }
                _ => println!("Error accessing path '{out_path}': {e}"),
            };
            return None;
        }
    };

    let mut buf = String::new();
    if let Err(e) = save_a.to_writer(&mut buf) {
        println!("Error serializing the merged save file: {e}");
        None
    } else if let Err(e) = out_file.write(buf.as_bytes()) {
        println!("Error writing to file '{out_path}': {e}");
        None
    } else {
        Some(())
    }
}

fn print_stats(args: Vec<String>, verbose: bool) -> Option<()> {
    if args.is_empty() {
        println!("You need to provide a path to the save file to read");
        return None;
    }

    let save_file = load_save(&args[0], verbose)?;

    if args.len() == 2 {
        let search_sid = &args[1];
        if let Some(area) = save_file.find_area_by_sid(search_sid) {
            println!("SID: {}\nID: {}", area.def.sid(), area.def.id);

            for (i, mode) in area.modes.iter().enumerate() {
                let checkpoints = mode.checkpoints.len();
                let side = match i {
                    0 => 'A',
                    1 => 'B',
                    2 => 'C',
                    3 => 'D',
                    _ => '_',
                };
                println!(
                    "{side} Side:\n\tCompleted: {}\n\tCheckpoints Unlocked: \
                     {checkpoints}\n\tBerries Collected: {}\n\tCleared In One Sitting: {}\n\tFull \
                     Cleared: {}\n\tDeaths: {}\n\tPlay Time: {}\n\tFastest Clear: {}\n\tFastest \
                     Full Clear: {}\n\tLow Deaths: {}\n\tLow Dashes: {}\n\tHeart Collected: {}",
                    mode.stats.completed,
                    mode.stats.total_strawberries,
                    mode.stats.single_run_completed,
                    mode.stats.full_clear,
                    mode.stats.deaths,
                    mode.stats.time_played,
                    mode.stats.best_time,
                    mode.stats.best_full_clear_time,
                    mode.stats.best_deaths,
                    mode.stats.best_dashes,
                    mode.stats.heart_gem,
                );
            }
        } else {
            println!("No area with the sid {search_sid} was found");
        }
    } else {
        println!(
            "Save Name: {}\nPlaytime: {}\nDeaths: {}\nNon-Modded Strawberries: {}\nGoldens: \
             {}\nJumps: {}\nDashes: {}\nWall Jumps: {}\nLast Played Vanilla Area: {}\nIs Modded: \
             {}",
            save_file.name,
            save_file.time,
            save_file.total_deaths,
            save_file.total_strawberries,
            save_file.total_golden_strawberries,
            save_file.total_jumps,
            save_file.total_dashes,
            save_file.total_jumps,
            save_file.last_area.sid(),
            save_file.has_modded_save_data,
        );

        if save_file.has_modded_save_data {
            let num_mods = save_file
                .level_sets
                .iter()
                .chain(save_file.level_set_recycle_bin.iter())
                .count();

            println!(
                "Number of modded level sets: {num_mods}\nLast modded area played: {}",
                save_file.last_area_safe.as_ref().unwrap().sid()
            );
        }
    }

    Some(())
}

fn print_clears(args: Vec<String>, verbose: bool) -> Option<()> {
    if args.is_empty() {
        println!("You need to provide a path to the save file to read");
        return None;
    }

    let save_file = load_save(&args[0], verbose)?;


    let mut clears = 0;
    for (area, _) in save_file
        .all_areas()
        .iter()
        .filter(|(a, _)| a.modes.iter().any(|m| m.stats.completed))
    {
        const SIDES: [char; 3] = ['a', 'b', 'c'];
        for (side, mode) in area.modes.iter().enumerate() {
            if mode.stats.completed {
                clears += 1;
                println!("{} {}-side", area.def.sid(), SIDES[side])
            }
        }
    }

    println!("\n{clears} total clears!");

    Some(())
}

/// Loads a celeste save from a path
///
/// Returns an [Option] because it handles printing errors to the user<br>
/// If None is returned, exiting early is likely the best choice
fn load_save(path: impl AsRef<Path> + Display, verbose: bool) -> Option<SaveData> {
    match OpenOptions::new().read(true).open(&path) {
        Ok(file) => match SaveData::from_reader(BufReader::new(file)) {
            Ok(save) => return Some(save),
            Err(e) =>
                if !verbose {
                    println!(
                        "Error reading save file '{path}'. Either the save file is corrupted or \
                         our program has an error.\nIf it is the later please re-run with \
                         --verbose and make a github issue with the error log"
                    )
                } else {
                    println!("Error reading save file '{path}'.\n{e:?}")
                },
        },
        Err(ref e) => match e.kind() {
            io::ErrorKind::NotFound => println!("The path '{path}' is not found."),
            io::ErrorKind::PermissionDenied => {
                println!("You do not have permissions to access the path '{path}'.")
            }
            _ => println!("Error accessing path '{path}': {e}"),
        },
    }
    None
}
