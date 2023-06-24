use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, SendError, Sender},
    thread::{self, JoinHandle},
    time,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListenerError<T> {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Sender error: {0}")]
    Sender(#[from] SendError<T>),

    #[error("Thread join error")]
    ThreadError,
}

pub struct ScreenshotListener {
    path: PathBuf,
    handle: Option<JoinHandle<Result<(), ListenerError<PathBuf>>>>,
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

        self.handle = Some(thread::spawn(move || {
            Self::screeshot_listener(&path, sender)
        }));

        receiver
    }

    pub fn stop(mut self) -> Result<(), ListenerError<PathBuf>> {
        if let Some(h) = self.handle.take() {
            match h.join() {
                Ok(res) => res,
                Err(_) => Err(ListenerError::ThreadError),
            }
        } else {
            Ok(())
        }
    }

    fn screeshot_listener(
        image_path: &Path,
        sender: Sender<PathBuf>,
    ) -> Result<(), ListenerError<PathBuf>> {
        let mut images = Self::get_images(image_path)?;

        loop {
            thread::sleep(time::Duration::from_secs(1));

            let new_images = Self::get_images(image_path)?;
            for image in new_images.difference(&images) {
                sender.send(image.clone())?;
            }

            // refresh images
            images = new_images;
        }
    }

    fn get_images(path: &Path) -> io::Result<HashSet<PathBuf>> {
        let iter = fs::read_dir(path)?
            .into_iter()
            .filter_map(|p| match p {
                Ok(val) => Some(val.path()),
                Err(_) => None,
            })
            .filter_map(|p| match fs::metadata(&p) {
                Ok(ref val) if val.is_file() => Some(p),
                Ok(_) => None,
                Err(_) => None,
            });

        Ok(iter.collect())
    }
}

impl Drop for ScreenshotListener {
    fn drop(&mut self) {
        if self.handle.is_some() {
            panic!("Listener must be stopped! Call the ScreeshotListener::stop method.");
        }
    }
}
