# Changelog

## atomic-write-file 0.3.0

### Unix changes

* Updated dependency on `nix` to version 0.30.

### Breaking changes

* Changed Rust edition from 2021 to 2024.

* Explicitly set the MSRV to 0.85 (the first Rust version to support the 2024
  edition).

## atomic-write-file 0.2.3

* Fixed documentation to state that `discard()` (not `commit()`) is called on
  drop (contributed by
  [Lucy](https://github.com/andreacorbellini/rust-atomic-write-file/pull/11)).

## atomic-write-file 0.2.2

### Unix changes

* Fixed a build error with `androideabi` (contributed by
  [xuxiaocheng0201](https://github.com/andreacorbellini/rust-atomic-write-file/pull/10)).

## atomic-write-file 0.2.1

* Fixed some broken documentation links.

## atomic-write-file 0.2.0

### New features

* Added the `AtomicWriteFile::directory` method to allow accessing the
  file descriptor of the parent directory, without making any system call (this
  method is available on all platforms, but it has a meaningful implementation
  only on Unix).

### Unix changes

* Updated dependency on `nix` to version 0.29.

* Sync the parent directory after committing or discarding the temporary file
  (via `fsync(2)`). This ensures that changes are persisted and not rolled-back
  if a crash occurs.

  In the previous releases, syncing the directory was intended to be a step
  that had to be explicitly performed by the caller *if* they wanted this
  behavior, and it was intentionally left out as a performance improvement for
  those callers that did not care about changes being persisted. However we do
  realize that callers may not be aware of the importance of this step, hence
  now it's done automatically.

### Breaking changes

* The `unnamed-tmpfile` is now disabled by default and needs to be explicitly
  enabled with `features = ["unnamed-tmpfile"]` in `Cargo.toml`. This decision
  was made to better support processes that run early at boot, and filesystems
  that have poor support for anonymous temporary files.

## atomic-write-file 0.1.4

### Linux changes

* Fix a potential data loss problem with anonymous temporary files on btrfs
  (see [GitHub issue
  #6](https://github.com/andreacorbellini/rust-atomic-write-file/issues/6) for
  details).

## atomic-write-file 0.1.3

### Unix changes

* Update dependency on `nix` to version 0.28. This improves compatibility with
  Illumos, Solaris, and Solaris-like operating systems. (contributed by
  [Rain](https://github.com/andreacorbellini/rust-atomic-write-file/pull/5)).

## atomic-write-file 0.1.2

### Linux changes

* Detect whether anonymous temporary files are supported or not, and
  automatically fall back to named temporary files in case they're not.

## atomic-write-file 0.1.1

### Unix changes

* Update dependency on `nix` to version 0.27 (contributed by
  [messense](https://github.com/andreacorbellini/rust-atomic-write-file/pull/2)).

## atomic-write-file 0.1.0

* Initial release.
