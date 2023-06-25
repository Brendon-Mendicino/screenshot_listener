use anyhow::{self, Context};

use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, FuzzySelect, Input};

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

    pub fn confirm<S: Into<String>>(&self, prompt: S) -> anyhow::Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .interact_on(&self.term)
            .context("Could not read terminal input!")
    }

    pub fn confirm_opt(&self) -> anyhow::Result<Option<bool>> {
        Confirm::with_theme(&self.theme)
            .with_prompt("Do you want to continue?")
            .interact_on_opt(&self.term)
            .context("Could not read terminal input!")
    }

    pub fn select_opt<T>(&self, items: &[T]) -> anyhow::Result<Option<usize>>
    where
        T: ToString,
    {
        FuzzySelect::with_theme(&self.theme)
            .with_prompt("Choose working directory")
            .items(items)
            .default(0)
            .interact_on_opt(&self.term)
            .context("Could not read terminal input!")
    }

    pub fn input<S: Into<String>>(&self, prompt: S) -> anyhow::Result<String> {
        Input::with_theme(&self.theme)
            .with_prompt(prompt)
            .interact_on(&self.term)
            .context("Could not read terminal input!")
    }
}
