# cec-alsa-sync

[![Crates.io](https://img.shields.io/crates/v/cec-alsa-sync.svg)](https://crates.io/crates/cec-alsa-sync)
[![Docs.rs](https://docs.rs/cec-alsa-sync/badge.svg)](https://docs.rs/cec-alsa-sync)
[![CI](https://github.com/ssalonen/cec-alsa-sync/workflows/Continuous%20Integration/badge.svg)](https://github.com/ssalonen/cec-alsa-sync/actions)
[![Coverage Status](https://coveralls.io/repos/github/ssalonen/cec-alsa-sync/badge.svg?branch=master)](https://coveralls.io/github/ssalonen/cec-alsa-sync?branch=master)

Small command line application to command ALSA volume using CEC.

This can be used to control e.g. Hifiberry DSP volume using TV remote.

## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this](https://www.rust-lang.org/tools/install) guide.
* run `cargo install cec-alsa-sync`

## License

Licensed under GNU General Public License version 2, ([LICENSE](LICENSE) or [https://opensource.org/licenses/GPL-2.0](https://opensource.org/licenses/GPL-2.0))

The CI/CD setup in `.github/` is based on [rust-github/template](https://github.com/rust-github/template), and therefore licensed under  either of

* Apache License, Version 2.0
   ([LICENSE-CI-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license
   ([LICENSE-CI-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Releasing

```cargo release --skip-publish``` and let the github CD pipeline do the rest.