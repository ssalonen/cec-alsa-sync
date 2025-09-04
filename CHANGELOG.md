# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 5.0.10

- manually track TV state (on/standby) and trigger commands on change
- default tv power state polling now 5 sec (previously 0.5 sec)
- throttling all cec command transmissions with 500 ms delay (spec)

## 5.0.9

- support autodetecting port name (omit port or leave it empty). Previous default was RPI.
- ci/cd: build for libcec6 shared libraries
- ci/cd: drop support for arm-unknown-linux-gnueabi (armel)

## 5.0.8

- updated libcec-sys which brings linux kernel CEC API support also with arm (in static builds)


## 5.0.7

- CI/CD fixes for non-static

## 5.0.6

- CI/CD fixes for static

## 5.0.5

- ci/cd: reverting static builds for now

## 5.0.4

- CI/CD fixes

## 5.0.3

- CI/CD fixes

## 5.0.2

- CI/CD to build static binaries as well

## 5.0.1

- CI/CD fixes

## 5.0.0

- bump dependency, support for tv on/off commands

## 4.0.4

- cd pipeline for aarch64-unknown-linux-gnu

## 4.0.3

- build also for aarch64-unknown-linux-gnu

## 4.0.2

- CD pipeline fix

## 4.0.1

- CD pipeline fix

## 4.0.0

- Bump to latest cec-rs, version 11.0.2

## 3.1.1

- Filter duplicate key press events (zero duration)

## 3.1.0


- CI: utilize Cross `pre-build` instead of pre-built docker images
- Support running custom commands on vol up, vol down, or mute button press.

## 3.0.3

- Revert 3.0.2 change, now classifying as `AudioSystem` again. There were some issues in practise.
- `Cross.toml` using fully qualified docker names
- `README.md` up-to-date `cargo release` instructions

## 3.0.2

- Classify as `PlaybackDevice`, not `AudioSystem`.

## 3.0.1

Fixes for new Samsung TV update, otherwise TV reverted to TV speaker automatically

- Handle `SystemAudioModeRequest`, responding now with `SetSystemAudioMode`
- Handle `GiveSystemAudioModeStatus`, respond with `SystemAudioModeStatus`
- Handle `GiveAudioStatus`, respond with `ReportAudioStatus` (although with dummy volume)

## 3.0.0

- cec-rs updated to v6. CI/CD compiled now against cec6

## 2.0.0

- libcec-sys update. Now support libcec version 4, 5 and 6.

## 1.2.1

- libcec-sys patch update

## 1.2.0

- Update `cec-rs` to 2.0.0
- log messages from libcec

## 1.1.19

- Fix crate description

## 1.1.18

- CI improvements
- Update `cec-rs` and `Cargo.lock`
