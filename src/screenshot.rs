use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver},
    },
    thread::{self, JoinHandle},
};

use listener::screeshot_listener;

pub struct ScreenshotListener {
    path: PathBuf,
    handle: Option<JoinHandle<()>>,
}

impl ScreenshotListener {
    pub fn new(path: &Path) -> Self {
        ScreenshotListener {
            path: path.into(),
            handle: None,
        }
    }

    pub fn listen(&mut self) -> Receiver<PathBuf> {
        let path = self.path.clone();
        let (sender, receiver) = channel();

        self.handle = Some(thread::spawn(move || screeshot_listener(&path, sender)));

        receiver
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
        fs,
        path::{Path, PathBuf},
        sync::{mpsc::Sender},
        thread,
    };

    pub fn screeshot_listener(image_path: &Path, sender: Sender<PathBuf>) {
        let mut old_images = get_images(image_path);

        loop {
            thread::sleep(time::Duration::from_secs(1));

            let new_images = get_new_images(image_path, &old_images);
            for image in &new_images {
                sender
                    .send(image.clone())
                    .expect("Sender should be albe to send image");
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
}
