use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use hunspell_sys::{Hunhandle, Hunspell_add, Hunspell_create, Hunspell_destroy, Hunspell_spell};
use lazy_static::lazy_static;

lazy_static! {
    static ref SEARCH_PATH: Vec<&'static Path> = vec![Path::new("/usr/share/hunspell"),];
}

#[derive(Debug)]
pub struct Checker {
    hunspell: *mut Hunhandle,
}

impl Checker {
    pub fn new() -> Self {
        let hunspell = SEARCH_PATH
            .iter()
            .find_map(|path| {
                let aff = path.join("en_US.aff");
                let dic = path.join("en_US.dic");

                if aff.exists() && dic.exists() {
                    let hunspell = unsafe {
                        Hunspell_create(
                            aff.as_os_str().as_bytes().as_ptr() as *const i8,
                            dic.as_os_str().as_bytes().as_ptr() as *const i8,
                        )
                    };
                    Some(hunspell)
                } else {
                    None
                }
            })
            .unwrap();

        Checker { hunspell }
    }

    pub fn check<'a, 'b: 'a>(
        &'b mut self,
        text: &'a str,
    ) -> impl Iterator<Item = SpellingError> + 'a {
        let hunspell = self.hunspell;

        text.split_whitespace().flat_map(move |word| {
            let cstr = CString::new(word).unwrap();
            let bytes = cstr.as_bytes_with_nul();
            let is_recognized =
                unsafe { Hunspell_spell(hunspell, bytes.as_ptr() as *const i8) } != 0;

            if !is_recognized {
                Some(SpellingError { word: cstr })
            } else {
                None
            }
        })
    }

    pub fn ignore(&mut self, word: &str) {
        let cstr = CString::new(word).unwrap();

        unsafe { Hunspell_add(self.hunspell, cstr.as_bytes_with_nul().as_ptr() as *const i8) };
    }
}

impl Drop for Checker {
    fn drop(&mut self) {
        unsafe {
            Hunspell_destroy(self.hunspell);
        }
    }
}

pub struct SpellingError {
    word: CString,
}

impl SpellingError {
    pub fn text(&self) -> &str {
        self.word.to_str().expect("original String is UTF-8")
    }
}
