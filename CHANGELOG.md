# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] — 2026-10-11

### Added

- Win32 UI: tree of power settings from the registry, Russian descriptions, visibility control.
- Visibility modes: **On** / **Off** / **Auto** (`Attributes` = show / hide / delete for Windows default).
- Self-elevation via UAC (`requireAdministrator` + `runas` fallback).
- Zero crate dependencies (`std` + Win32 FFI only).
- Generated Russian catalog (~154 settings) with sectioned descriptions.
