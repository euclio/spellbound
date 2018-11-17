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

Linux is not currently supported, since it does not have a native spell checking
API.

[`ISpellChecker`]: https://docs.microsoft.com/en-us/windows/desktop/api/spellcheck/nn-spellcheck-ispellchecker
[`NSSpellChecker`]: https://developer.apple.com/documentation/appkit/nsspellchecker
