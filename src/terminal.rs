use std::io;

use dialoguer::{
    console::Term,
    theme::{ColorfulTheme, Theme},
    Confirm, FuzzySelect,
};

pub struct Terminal {
    theme: ColorfulTheme,
    term: Term,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
            term: Term::buffered_stderr(),
        }
    }

    pub fn is_ok(&self) -> io::Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt("Do you want to continue?")
            .interact_on(&self.term)
    }

    pub fn select<T>(&self, items: &[T]) -> io::Result<usize>
    where
        T: ToString,
    {
        FuzzySelect::with_theme(&self.theme)
            .with_prompt("Choose working directory")
            .items(items)
            .default(0)
            .interact_on(&self.term)
    }
}
