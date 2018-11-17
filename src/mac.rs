use std::ops::Deref;
use std::slice;
use std::str;
use std::sync::Mutex;

use cocoa::{
    appkit::NSSpellChecker,
    base::{id, nil, NO},
    foundation::{NSInteger, NSNotFound, NSString, NSUInteger},
};
use lazy_static::lazy_static;

/// `NSSpellChecker` is not thread safe. It should only be used from one thread, or it will cause
/// spurious `EXC_BAD_ACCESS` errors. If access to it is synchronized, however, it should be safe
/// to send across threads.
struct NSSpellCheckerWrapper(id);

unsafe impl Send for NSSpellCheckerWrapper {}

impl Deref for NSSpellCheckerWrapper {
    type Target = id;

    fn deref(&self) -> &id {
        &self.0
    }
}

lazy_static! {
    static ref CHECKER: Mutex<NSSpellCheckerWrapper> =
        Mutex::new(unsafe { NSSpellCheckerWrapper(NSSpellChecker::sharedSpellChecker(nil)) });
}

#[derive(Debug)]
pub struct Checker {
    document_tag: NSInteger,
}

impl Drop for Checker {
    fn drop(&mut self) {
        unsafe {
            CHECKER
                .lock()
                .unwrap()
                .closeSpellDocumentWithTag(self.document_tag)
        };
    }
}

impl Checker {
    pub fn new() -> Self {
        Self {
            document_tag: unsafe { NSSpellChecker::uniqueSpellDocumentTag(nil) },
        }
    }

    pub fn ignore(&mut self, word: &str) {
        let word = unsafe { NSString::alloc(nil).init_str(word) };
        unsafe {
            CHECKER
                .lock()
                .unwrap()
                .ignoreWord_inSpellDocumentWithTag(word, self.document_tag)
        };
    }

    pub fn check(&mut self, text: &str) -> impl Iterator<Item = SpellingError> {
        let text = unsafe { NSString::alloc(nil).init_str(text) };

        SpellcheckIter {
            document_tag: self.document_tag,
            text: text,
            offset: 0,
        }
    }
}

#[derive(Debug)]
pub struct SpellingError {
    text: String,
}

impl SpellingError {
    pub fn text(&self) -> &str {
        &self.text
    }
}

struct SpellcheckIter {
    text: id, /* NSString */
    document_tag: NSInteger,
    offset: NSUInteger,
}

impl Iterator for SpellcheckIter {
    type Item = SpellingError;

    fn next(&mut self) -> Option<Self::Item> {
        let (range, _) =
            unsafe {
                CHECKER.lock().unwrap()
                .checkSpellingOfString_startingAt_language_wrap_inSpellDocumentWithTag_wordCount(
                    self.text,
                    self.offset as NSInteger,
                    nil,
                    NO,
                    self.document_tag,
                )
            };

        if range.location == NSNotFound as NSUInteger {
            return None;
        };

        self.offset = range.location + range.length;

        let misspelling = unsafe {
            let misspelling = self.text.substringWithRange(range);
            let misspelling_bytes = misspelling.UTF8String() as *const u8;
            str::from_utf8(slice::from_raw_parts(misspelling_bytes, misspelling.len())).unwrap()
        };

        Some(SpellingError {
            text: misspelling.to_owned(),
        })
    }
}
