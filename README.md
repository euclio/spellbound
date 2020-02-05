# spellbound
[![travis-ci Build Status](https://travis-ci.com/euclio/spellbound.svg?branch=master)](https://travis-ci.com/euclio/spellbound)
[![AppVeyor Build Status](https://ci.appveyor.com/api/projects/status/github/euclio/spellbound?svg=true)](https://ci.appveyor.com/project/euclio/spellbound)

`spellbound` is a small crate that binds to the native platform's spell checking
APIs and wraps them in a friendlier, rustic interface.

Supported platforms and corresponding APIs:

| Platform | API                |
| -------- | ------------------ |
| MacOS    | [`NSSpellChecker`] |
| Windows  | [`ISpellChecker`]  |
| *nix     | [`hunspell`]

[`ISpellChecker`]: https://docs.microsoft.com/en-us/windows/desktop/api/spellcheck/nn-spellcheck-ispellchecker
[`NSSpellChecker`]: https://developer.apple.com/documentation/appkit/nsspellchecker
[`hunspell`]: https://hunspell.github.io/
