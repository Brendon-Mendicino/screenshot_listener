use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use std::sync::mpsc::{Receiver, TryRecvError};

use std::{fs, thread};

use anyhow::Context;
use clap::Parser;

mod screenshot;

use screenshot::ScreenshotListener;

use crate::terminal::Terminal;

mod terminal;

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
    // /// If enabled chooses the last screenshot in the `SCREENSHOT` dir
    // #[arg(short, long)]
    // last: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum MenuState {
    Selection,
    Listening(PathBuf),
    Stopped,
}

fn contains_img_dir(path: &PathBuf) -> anyhow::Result<bool> {
    let res = fs::read_dir(path)
        .with_context(|| format!("Failed to read: {}", path.to_string_lossy()))?
        .filter(|p| {
            let p = match p {
                Ok(val) => val,
                Err(_) => return false,
            };

            let img = OsString::from_str("img").unwrap();

            p.file_name() == img && fs::metadata(p.path()).map_or(false, |p| p.is_dir())
        })
        .count()
        == 1;

    Ok(res)
}

/// A directory to be included must contain a `img` subfolder
fn get_note_dirs(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let iter = fs::read_dir(path)?
        .filter_map(|p| match p {
            Ok(val) => Some(val.path()),
            Err(_) => None,
        })
        .flat_map(|p| match fs::metadata(&p) {
            Ok(ref val) if val.is_dir() && contains_img_dir(&p).map_or(false, |v| v) => Some(p),
            Ok(_) => None,
            Err(_) => None,
        });

    Ok(Vec::from_iter(iter))
}

fn move_image(image: &Path, destination_path: &Path, term: &Terminal) -> anyhow::Result<bool> {
    let new_name = term.input(format!(
        "Choose new new for the file \"{}\"",
        image.file_name().unwrap().to_string_lossy()
    ))?;

    let to = destination_path.join("img").join(new_name);
    if !term.confirm(format!(
        "Do you want to move this image to \"{}\"?",
        to.to_string_lossy()
    ))? {
        return Ok(false);
    }

    fs::rename(image, to.clone()).with_context(|| {
        format!(
            "Renaming file from: \"{}\" to \"{}\" failed!",
            image.to_string_lossy(),
            to.to_string_lossy()
        )
    })?;

    Ok(true)
}

fn menu(note_path: &Path, screenshot_listener: Receiver<PathBuf>) -> anyhow::Result<()> {
    let notes = get_note_dirs(note_path).expect("Path sould exists!");

    let term = Terminal::new();
    let mut state = MenuState::Selection;

    loop {
        match state {
            MenuState::Selection => {
                let items = Vec::from_iter(
                    notes
                        .iter()
                        .map(|p| p.file_name().unwrap().to_string_lossy()),
                );

                let index = match term.select_opt(items.as_slice())? {
                    Some(index) => index,
                    None => break,
                };

                state = MenuState::Listening(notes[index].clone());
            }
            MenuState::Listening(ref path) => {
                match screenshot_listener.try_recv() {
                    Ok(image) => {
                        move_image(&image, path, &term)?;
                    }
                    Err(TryRecvError::Empty) => (),
                    Err(err) => {
                        return Err(err).context("Listener died unexpectedly!");
                    }
                }

                match term.confirm_opt()? {
                    Some(value) if value => (),
                    Some(_) => state = MenuState::Selection,
                    None => state = MenuState::Stopped,
                }
            }
            MenuState::Stopped => {
                break;
            }
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = ScreenshotArgs::parse();

    let mut listener = ScreenshotListener::new(&args.screenshot);
    let receiver: Receiver<PathBuf> = listener.listen();

    let res = thread::scope(|s| s.spawn(move || menu(&args.note, receiver)).join().unwrap());

    listener.stop()?;

    res?;

    Ok(())
}
