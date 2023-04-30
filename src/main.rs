use core::time;
use std::cmp::max;
use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{error, fs, thread};

use clap::Parser;

mod screenshot;
use screenshot::ScreenshotListener;

/// Program to move screeshots
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ScreenshotArgs {
    /// Screenshot path
    #[arg(short, long, default_value = "/home/brendon/Pictures/Screenshots")]
    screenshot: PathBuf,

    /// Notes path
    #[arg(short, long, default_value = "/home/brendon/uni/appunti")]
    note: PathBuf,
}

#[derive(Debug)]
enum MenuState {
    Idle,
    Listening(PathBuf),
    Stopped,
}

enum Input {
    Continue,
    Back,
    Exit,
}

fn print_menu(paths: &BTreeSet<PathBuf>) {
    let max_len = max(
        80,
        paths
            .iter()
            .map(|p| p.display().to_string().chars().count())
            .max()
            .unwrap_or(0)
            + 6,
    );
    let ceiling = (0..max_len).map(|_| "#").collect::<String>();

    println!("{}", ceiling);
    for (i, p) in paths.iter().enumerate() {
        let item = format!("# {:3}. {}", i, p.display());
        let end = (0..(max_len - item.chars().count() - 1))
            .map(|_| " ")
            .collect::<String>()
            + "#";
        println!("{}{}", item, end);

        let spaces = (0..(max_len - 2)).map(|_| " ").collect::<String>();
        println!("#{}#", spaces);
    }
    println!("{}", ceiling);
}

fn choose_working_dir(paths: &BTreeSet<PathBuf>) -> Result<PathBuf, Box<dyn error::Error>> {
    print!("> ");
    io::stdout().lock().flush()?;

    // Get input position
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let chose_number: usize = input.trim().parse()?;

    // Match input
    let res = paths
        .iter()
        .enumerate()
        .find_map(|p| match p.0 == chose_number {
            true => Some(p.1),
            false => None,
        })
        .ok_or("Path does not exists!");

    Ok(res?.to_path_buf())
}

fn ask_for_continuation() -> Result<Input, Box<dyn error::Error>> {
    let mut string = String::new();

    print!("Press [b/back] to go back, [e] to exit, press anything to see new updates.\n> ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut string)?;

    match string.to_lowercase().as_str().trim() {
        "b" | "back" => Ok(Input::Back),
        "e" | "exit" => Ok(Input::Exit),
        _ => Ok(Input::Continue),
    }
}

fn contains_img_dir(path: &PathBuf) -> bool {
    fs::read_dir(path)
        .unwrap()
        .into_iter()
        .filter(|p| {
            let p = match p {
                Ok(val) => val,
                Err(_) => return false,
            };

            let img = OsString::from_str("img").unwrap();

            p.file_name() == img && fs::metadata(p.path()).unwrap().is_dir()
        })
        .count()
        == 1
}

/// A directory to be included must contain a `img` subfolder
fn get_note_dirs(path: &Path) -> Result<BTreeSet<PathBuf>, Box<dyn Error>> {
    let iter = fs::read_dir(path)?
        .into_iter()
        .filter_map(|p| match p {
            Ok(val) => Some(val.path()),
            Err(_) => return None,
        })
        .flat_map(|p| match fs::metadata(&p) {
            Ok(ref val) => {
                if val.is_dir() && contains_img_dir(&p) {
                    Some(p)
                } else {
                    None
                }
            }
            Err(_) => None,
        });

    Ok(BTreeSet::from_iter(iter))
}

fn choose_new_name(old_name: &str) -> Result<OsString, io::Error> {
    let mut new_name = String::new();
    print!("Choose new name for the file: \"{}\".\n> ", old_name);
    io::stdout().flush()?;
    io::stdin().read_line(&mut new_name)?;

    Ok(OsString::from(new_name.trim()))
}

fn ask_confirmation() -> Result<bool, io::Error> {
    let mut input_line = String::new();

    print!("Are you sure? [y/n]\n> ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input_line)?;

    match input_line.to_lowercase().as_str().trim() {
        "y" => Ok(true),
        "ye" => Ok(true),
        "yes" => Ok(true),
        _ => Ok(false),
    }
}

fn move_image(image: &Path, destination_path: &Path) -> Result<(), Box<dyn Error>> {
    println!("Currently in \"{}\"", destination_path.display());

    let new_name = choose_new_name(image.file_name().unwrap().to_str().unwrap())?;

    if !ask_confirmation()? {
        return Ok(());
    }

    fs::rename(image, destination_path.join("img").join(new_name))?;

    Ok(())
}

fn menu(note_path: &Path, receiver: Receiver<PathBuf>) {
    let notes = get_note_dirs(note_path).expect("Path sould exists!");

    let mut state = MenuState::Idle;

    loop {
        thread::sleep(time::Duration::from_secs(1));

        if let MenuState::Idle = state {
            print_menu(&notes);
            let result = match choose_working_dir(&notes) {
                Ok(val) => val,
                Err(err) => {
                    eprintln!("An error accurred: {}", err);
                    continue;
                }
            };

            state = MenuState::Listening(result);
        } else if let MenuState::Listening(_) = state {
            match ask_for_continuation().unwrap() {
                Input::Back => state = MenuState::Idle,
                Input::Exit => state = MenuState::Stopped,
                Input::Continue => (),
            }
        } else {
            break;
        }
    }
}

fn main() {
    let args = ScreenshotArgs::parse();

    let mut listener = ScreenshotListener::new(&args.screenshot);
    let receiver = listener.listen();

    let choice_handle = thread::spawn(move || menu(&args.note, receiver));

    choice_handle.join().unwrap();
    listener.stop().unwrap();
}
