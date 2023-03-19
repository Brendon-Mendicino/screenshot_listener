use std::cmp::max;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::io;
use std::path::PathBuf;

use clap::Parser;

mod file_operations;
use file_operations::*;



/// Program to move screeshots
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ScreenshotArgs {

    /// Screenshot path
    #[arg(short, long, default_value = "/home/brendon/Pictures")]
    screenshot: PathBuf,
    
    /// Notes path
    #[arg(short, long, default_value = "/home/brendon/uni/appunti")]
    note: PathBuf,
}


/// State of the program
#[derive(Debug)]
enum State {
    Idle,
    Listening(PathBuf),
    Stopped,
}




fn print_menu(paths: &HashSet<PathBuf>) {
    let max_len = max(
        80, 
        paths.into_iter()
            .map(|p| p.display().to_string().chars().count())
            .max()
            .unwrap_or(0) + 6
    );
    let ceiling = (0..max_len).map(|_| "#").collect::<String>();

    println!("{}", ceiling);
    for (i, p) in paths.iter().enumerate() {
        let item = format!("# {:3}. {}", i, p.display());
        let end = (0..(max_len - item.chars().count() - 1)).map(|_| " ").collect::<String>() + "#";
        println!("{}{}", item, end);
        println!("#{}#", (0..(max_len - 2)).map(|_| " ").collect::<String>())
    }
    println!("{}", ceiling);
}

fn choose_working_dir(paths: &HashSet<PathBuf>) -> Result<PathBuf, io::Error> {

    // Get input position
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let chose_number: usize = match input.trim().parse() {
        Ok(val) => val,
        Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
    };

    
    // Match input
    let res = paths.iter().enumerate()
        .find_map(|p| match p.0 == chose_number {
            true => Some(p.1),
            false => None
        });

    match res {
        Some(val) => Ok(val.to_path_buf()),
        None => Err(io::Error::new(io::ErrorKind::Other, String::from("Path does not exist!"))),
    }
}


fn main() {
    let args = ScreenshotArgs::parse();
    let state = Arc::new(Mutex::new(State::Idle));


    
    let notes = get_note_dirs(&args.note);

    loop {
        let mut state = state.lock().unwrap();
        println!("{:?}", *state);


        print_menu(&notes);
        let result = choose_working_dir(&notes);
        if let Err(err) = result {
            eprintln!("An error accurred: {}", err.to_string());
            continue;
        }
        
        *state = State::Listening(result.unwrap());
    }
}

