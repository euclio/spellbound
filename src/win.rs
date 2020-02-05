use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::iter;
use std::mem;
use std::ops::Deref;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{self, NonNull};

use winapi::{
    Class,
    Interface,
    shared::{
        winerror::{FAILED, SUCCEEDED, S_FALSE},
        wtypesbase::CLSCTX_INPROC_SERVER,
    },
    um::{
        combaseapi::{CoInitializeEx, CoCreateInstance},
        objbase::COINIT_MULTITHREADED,
        spellcheck::{IEnumSpellingError, SpellCheckerFactory, ISpellChecker, ISpellCheckerFactory},
        unknwnbase::IUnknown,
    },
};

struct ComPtr<T>(NonNull<T>);

impl<T> ComPtr<T> {
    fn new(p: *mut T) -> ComPtr<T> where T: Interface {
        ComPtr(NonNull::new(p).unwrap())
    }
}

impl<T> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T> Debug for ComPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("ComPtr")
            .field(&format_args!("{:p}", self.0.as_ptr()))
            .finish()
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let unknown = self.0.as_ptr() as *mut IUnknown;
            (*unknown).Release();
        }
    }
}

fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
}

#[derive(Debug)]
pub struct Checker {
    checker: ComPtr<ISpellChecker>,
}

impl Checker {
    pub fn new() -> Self {
        let hr = unsafe { CoInitializeEx(ptr::null_mut(), COINIT_MULTITHREADED) };
        if !SUCCEEDED(hr) {
            panic!("could not initialize COM: {:?}", hr);
        }

        let mut obj = ptr::null_mut();
        let hr = unsafe {
            CoCreateInstance(
                &SpellCheckerFactory::uuidof(),
                ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &ISpellCheckerFactory::uuidof(),
                &mut obj,
            )
        };

        assert!(SUCCEEDED(hr), "could not create spellchecker factory instance");
        let factory = ComPtr::new(obj as *mut ISpellCheckerFactory);

        let mut checker = ptr::null_mut();
        let lang = wide_string("en-US");
        let hr = unsafe { (*factory).CreateSpellChecker(lang.as_ptr(), &mut checker) };
        assert!(SUCCEEDED(hr), "could not create spellchecker instance");
        let checker = ComPtr::new(checker);

        Checker {
            checker,
        }
    }

    pub fn check(&mut self, text: &str) -> impl Iterator<Item = SpellingError> {
        if text.is_empty() {
            return ErrorIter {
                text: vec![],
                iter: None,
            };
        }

        let text = wide_string(text);
        let mut errors = ptr::null_mut();
        let hr = unsafe { (*self.checker).ComprehensiveCheck(text.as_ptr(), &mut errors) };
        assert!(SUCCEEDED(hr));
        let errors = ComPtr::new(errors);

        ErrorIter {
            text,
            iter: Some(errors),
        }
    }

    pub fn ignore(&mut self, word: &str) {
        if word.is_empty() {
            return;
        }

        let word = wide_string(word);
        let hr = unsafe { (*self.checker).Ignore(word.as_ptr()) };
        assert!(SUCCEEDED(hr));
    }
}

struct ErrorIter {
    text: Vec<u16>,
    iter: Option<ComPtr<IEnumSpellingError>>,
}

impl Iterator for ErrorIter {
    type Item = SpellingError;

    fn next(&mut self) -> Option<SpellingError> {
        let iter = self.iter.as_ref()?;

        let mut err = unsafe { mem::uninitialized() };
        if unsafe { (*iter).Next(&mut err) } != S_FALSE {
            let err = ComPtr::new(err);

            let mut start = 0;
            let mut length = 0;

            unsafe {
                (*err).get_Length(&mut length);
                (*err).get_StartIndex(&mut start);
            }

            let start = start as usize;
            let length = length as usize;

            let err_text = String::from_utf16(&self.text[start..start + length]).unwrap();

            return Some(SpellingError {
                text: err_text,
            });
        } else {
            None
        }
    }
}

pub struct SpellingError {
    text: String,
}

impl SpellingError {
    pub fn text(&self) -> &str {
        &self.text
    }
}
