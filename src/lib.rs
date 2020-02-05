//! `spellbound` is a small crate that binds to the native platform's spell checking APIs and
//! provides a friendlier API.
//!
//! This corresponds to [`ISpellChecker`] on Windows, [`NSSpellChecker`] on MacOS, and [`hunspell`]
//! on other *nix platforms.
//!
//! # Example
//!
//! ```
//! use spellbound::Checker;
//!
//! let mut checker = Checker::new();
//!
//! let errors: Vec<_> = checker.check("I beleeve I can fly").collect();
//!
//! assert_eq!(errors.len(), 1);
//! assert_eq!(errors[0].text(), "beleeve");
//! ```
//!
//! [`ISpellChecker`]: https://docs.microsoft.com/en-us/windows/desktop/api/spellcheck/nn-spellcheck-ispellchecker
//! [`NSSpellChecker`]: https://developer.apple.com/documentation/appkit/nsspellchecker
//! [`hunspell`]: https://hunspell.github.io/

extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "macos")] {
        mod mac;
        use crate::mac as imp;
    } else if #[cfg(windows)] {
        mod win;
        use crate::win as imp;
    } else if #[cfg(unix)] {
        mod unix;
        use crate::unix as imp;
    } else {
        compile_error!("target platform is not supported");
    }
}

/// Instance of the system spell checker.
#[derive(Debug)]
pub struct Checker(imp::Checker);

impl Checker {
    /// Create an instance of the system spell checker.
    pub fn new() -> Self {
        Checker(imp::Checker::new())
    }

    /// Check a text for spelling errors. Returns an iterator over the errors present in the text.
    pub fn check<'a, 'b: 'a>(
        &'b mut self,
        text: &'a str,
    ) -> impl Iterator<Item = SpellingError> + 'a {
        self.0.check(text).map(SpellingError)
    }

    /// Instructs the spell checker to ignore a word in future checks. The word is temporarily
    /// added to the spell checker's ignore list, and other instances of the spell checker will not
    /// ignore the word.
    pub fn ignore(&mut self, word: &str) {
        self.0.ignore(word)
    }
}

/// A spelling error.
pub struct SpellingError(imp::SpellingError);

impl SpellingError {
    /// Returns the text of the misspelled word.
    pub fn text(&self) -> &str {
        self.0.text()
    }
}

#[cfg(test)]
mod tests {
    use super::Checker;

    #[test]
    fn no_errors() {
        let text = "I'm happy that this sentence has no errors.";
        let mut checker = Checker::new();
        assert_eq!(checker.check(&text).count(), 0);
    }

    #[test]
    fn single_error() {
        let text = "asdf";
        let mut checker = Checker::new();
        let errors = checker.check(&text).collect::<Vec<_>>();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text(), "asdf");
    }

    #[test]
    fn multiple_errors() {
        let text = "asdf hjkl qwer uiop";
        let mut checker = Checker::new();
        let errors = checker.check(&text).collect::<Vec<_>>();
        assert_eq!(errors.len(), 4);
        assert_eq!(errors[0].text(), "asdf");
        assert_eq!(errors[1].text(), "hjkl");
        assert_eq!(errors[2].text(), "qwer");
        assert_eq!(errors[3].text(), "uiop");
    }

    #[test]
    fn empty() {
        let mut checker = Checker::new();
        assert_eq!(checker.check("").count(), 0);
    }

    #[test]
    fn ignore() {
        let mut checker = Checker::new();

        assert_eq!(checker.check("foobarbaz").count(), 1);

        checker.ignore("foobarbaz");

        assert_eq!(checker.check("foobarbaz").count(), 0);
    }

    #[test]
    fn ignore_not_permanent() {
        let mut checker = Checker::new();

        checker.ignore("foobarbaz");

        drop(checker);

        let mut checker = Checker::new();

        assert_eq!(checker.check("foobarbaz").count(), 1);
    }
}
