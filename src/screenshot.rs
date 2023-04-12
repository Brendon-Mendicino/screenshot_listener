use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use listener::screeshot_listener;

pub struct ScreenshotListener {
    path: PathBuf,
    handle: Option<JoinHandle<()>>,
}

/// State of the program
#[derive(Debug)]
pub enum ListeningState {
    Idle,
    Listening(PathBuf),
    Stopped,
}

impl ScreenshotListener {
    pub fn new(path: &Path) -> Self {
        ScreenshotListener {
            path: path.into(),
            handle: None,
        }
    }

    pub fn listen(&mut self) -> Arc<Mutex<ListeningState>> {
        let state = Arc::new(Mutex::new(ListeningState::Idle));
        let path = self.path.clone();
        let return_state = state.clone();

        self.handle = Some(thread::spawn(move || screeshot_listener(&path, state)));

        return_state
    }

    pub fn stop(mut self) -> thread::Result<()> {
        if let Some(h) = self.handle.take() {
            h.join()?;
        }

        Ok(())
    }
}

impl Drop for ScreenshotListener {
    fn drop(&mut self) {
        if self.handle.is_some() {
            panic!("Listener must be stopped!");
        }
    }
}

mod listener {
    use core::time;
    use std::{
        collections::BTreeSet,
        error::Error,
        ffi::OsString,
        fs,
        io::{self, Write},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
        thread,
    };

    use super::ListeningState;

    pub fn screeshot_listener(image_path: &Path, state: Arc<Mutex<ListeningState>>) {
        let mut old_images = get_images(image_path);

        loop {
            thread::sleep(time::Duration::from_secs(1));

            let state = state.lock().unwrap();

            let destination_path = match *state {
                ListeningState::Idle => continue,
                ListeningState::Stopped => break,
                ListeningState::Listening(ref path) => path.clone(),
            };

            let new_images = get_new_images(image_path, &old_images);
            for image in &new_images {
                if let Err(err) = move_image(image, &destination_path) {
                    eprintln!("An error accurred: {}", err);
                }
            }

            // refresh images
            old_images.extend(new_images);
        }
    }

    fn get_images(path: &Path) -> BTreeSet<PathBuf> {
        let iter = fs::read_dir(path)
            .unwrap()
            .into_iter()
            .filter_map(|p| match p {
                Ok(val) => Some(val.path()),
                Err(_) => None,
            })
            .filter_map(|p| match fs::metadata(&p) {
                Ok(ref val) => {
                    if val.is_file() {
                        Some(p)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            });

        BTreeSet::from_iter(iter)
    }

    fn get_new_images(path: &Path, old_images: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
        let mut new_images = BTreeSet::<PathBuf>::new();

        fs::read_dir(path).unwrap().for_each(|p| {
            if let Ok(val) = p {
                if !old_images.contains(&val.path()) {
                    new_images.insert(val.path());
                }
            }
        });

        new_images
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
}
