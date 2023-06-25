use core::panic;
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time,
};

#[derive(Debug, thiserror::Error)]
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
    stop_signal: Option<Sender<()>>,
}

impl ScreenshotListener {
    pub fn new(path: &Path) -> Self {
        ScreenshotListener {
            path: path.into(),
            handle: None,
            stop_signal: None,
        }
    }

    /// Starts a new thread and listen for new changes in the
    /// `path` directory
    pub fn listen(&mut self) -> Receiver<PathBuf> {
        if self.handle.is_some() {
            panic!("Called listen when already listening!");
        }

        let path = self.path.clone();
        let (sender, receiver) = channel();
        let (stop_signal_sx, stop_signal_rx) = channel();

        self.stop_signal = Some(stop_signal_sx);
        self.handle = Some(thread::spawn(move || {
            Self::screeshot_listener(&path, sender, stop_signal_rx)
        }));

        receiver
    }

    pub fn stop(mut self) -> Result<(), ListenerError<PathBuf>> {
        if let Some(ref s) = self.stop_signal {
            s.send(()).unwrap();
        }

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
        stop_signal: Receiver<()>,
    ) -> Result<(), ListenerError<PathBuf>> {
        let mut images = Self::get_images(image_path)?;

        loop {
            // Stop thread
            match stop_signal.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => break,
                Err(_) => (),
            }

            thread::sleep(time::Duration::from_secs(1));

            let new_images = Self::get_images(image_path)?;
            for image in new_images.difference(&images) {
                sender.send(image.clone())?;
            }

            // refresh images
            images = new_images;
        }

        Ok(())
    }

    fn get_images(path: &Path) -> io::Result<HashSet<PathBuf>> {
        let iter = fs::read_dir(path)?
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
