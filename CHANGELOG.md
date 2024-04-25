# Changelog

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
