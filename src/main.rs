use std::cmp::max;
use std::collections::HashSet;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::{fs, io};
use std::path::PathBuf;

use clap::Parser;




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


fn get_images(path: &PathBuf) -> HashSet<PathBuf> {
    HashSet::from_iter(
        fs::read_dir(path).unwrap()
            .into_iter()
            .filter_map(|p| {
                let p = match p {
                    Ok(val) => val.path(),
                    Err(_) => return None,
                };

                match fs::metadata(&p) {
                    Ok(ref val) => if val.is_file() { Some(p) } else { None },
                    Err(_) => None
                }
            })
    )
}


fn contains_img_dir(path: &PathBuf) -> bool {
    fs::read_dir(path).unwrap()
        .into_iter()
        .filter(|p| {
            let p = match p {
                Ok(val) => val,
                Err(_) => return false,
            };

            p.file_name() == OsString::from("img") && fs::metadata(p.path()).unwrap().is_dir()
        })
        .count() == 1
}

/// A directory to be included must contain a `img` subfolder
fn get_note_dirs(path: &PathBuf) -> HashSet<PathBuf> {
    HashSet::from_iter(
        fs::read_dir(path).unwrap()
            .into_iter()
            .filter_map(|p| {
                let p = match p {
                    Ok(val) => val.path(),
                    Err(_) => return None,
                };

                match fs::metadata(&p) {
                    Ok(ref val) => if val.is_dir() && contains_img_dir(&p) { Some(p) } else { return None },
                    Err(_) => return None
                }
            })
    )
}


fn get_new_images(path: &PathBuf, old_images: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    let mut new_images = HashSet::<PathBuf>::new();

    fs::read_dir(path).unwrap()
        .for_each(|p| {
            match p {
                Ok(val) => {
                    if old_images.contains(&val.path()) { return }
                    new_images.insert(val.path())
                },
                Err(_) => false,
            };
        });

    new_images
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

