use core::time;
use std::cmp::max;
use std::collections::HashSet;
use std::ffi::{OsString, OsStr};
use std::sync::{Arc, Mutex};
use std::io::{self, Write, Read};
use std::path::PathBuf;
use std::{thread, fs, error};

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

        let spaces = (0..(max_len - 2)).map(|_| " ").collect::<String>();
        println!("#{}#", spaces);
    }
    println!("{}", ceiling);
}

fn choose_working_dir(paths: &HashSet<PathBuf>) -> Result<PathBuf, Box<dyn error::Error>> {

    print!("> ");
    io::stdout().lock().flush()?;

    // Get input position
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let chose_number: usize = input.trim().parse()?;

    
    // Match input
    let res = paths.iter().enumerate()
        .find_map(|p| match p.0 == chose_number {
            true => Some(p.1),
            false => None
        })
        .ok_or_else(|| "Path does not exists!");
        
    Ok(res?.to_path_buf())
}

fn ask_confirmation() -> Result<bool, io::Error> {
    let mut string = String::new();

    print!("Are you sure? [y/n]\n> ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut string)?;
    
    match string.to_lowercase().as_str().trim() {
        "y" => Ok(true),
        "ye" => Ok(true),
        "yes" => Ok(true),
        _ => Ok(false),
    }
}

fn choose_new_name(old_name: &str) -> Result<OsString, io::Error> {
    let mut new_name = String::new();
    print!("Choose new name for the file: \"{}\".\n> ", old_name);
    io::stdout().flush()?;
    io::stdin().read_line(&mut new_name)?;

    Ok(OsString::from(new_name.trim()))
}


fn move_image(image: &PathBuf, destination_path: &PathBuf) -> Result<(), Box<dyn error::Error>> {

    let new_name = choose_new_name(image.file_name().unwrap().to_str().unwrap())?;

    if !ask_confirmation()? { return Ok(()); }

    fs::rename(image, destination_path.join("img").join(new_name))?;

    Ok(())
}

fn screeshot_listener(image_path: &PathBuf, state: Arc<Mutex<State>>) {
    let mut old_images = get_images(image_path);

    loop {
        thread::sleep(time::Duration::from_secs(3));

        let state = state.lock().unwrap();

        let destination_path = match *state { 
            State::Idle => continue,
            State::Stopped => break,
            State::Listening(ref path) => path.clone(),
        };

        let new_images = get_new_images(image_path, &old_images);
        for image in &new_images {

            if let Err(err) = move_image(image, &destination_path) {
                eprintln!("An error accurred: {}", err.to_string());
            }
        }

        // refresh images
        old_images.extend(new_images);
    }
}

fn main() {
    let args = ScreenshotArgs::parse();
    let state = Arc::new(Mutex::new(State::Idle));


    
    let notes = get_note_dirs(&args.note);



    let state_screen = Arc::clone(&state);
    let screenshot_handle = thread::spawn(move || screeshot_listener(&args.screenshot, state_screen));

    loop {
        thread::sleep(time::Duration::from_secs(1));

        let mut state = state.lock().unwrap();
        println!("{:?}", *state);

        if let State::Stopped = *state { break; }

        print_menu(&notes);
        let result = match choose_working_dir(&notes) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("An error accurred: {}", err.to_string());
                continue;
            }
        };
                
        *state = State::Listening(result);
    }


    screenshot_handle.join().unwrap();
}

