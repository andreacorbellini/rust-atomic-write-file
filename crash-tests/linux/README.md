# Crash tests for Linux

These are tests to ensure that
[atomic-write-file](https://crates.io/crates/atomic-write-file) holds up to its
promises when a kernel panic occurs on Linux.

These tests verify that for a file written with atomic-write-file either has
its old contents, or the new ones, and never any intermediate contents. They
work by doing the following inside a [Qemu](https://www.qemu.org/) virtual
machine:

- initializing an empty filesystem
- writing a file with some initial contents
- using atomic-write-file to write and commit new contents to the file
- triggering a kernel panic
- checking the contents of the file

The tests support the following filesystem types:
- ext3
- ext4
- btrfs
- xfs

Other filesystems may work as well, but they are currently untested.

Note that unjournaled filesystems (like ext2) will not work because a kernel
panic will leave them in a broken state.

## Usage

Tests are run by the `run-tests.sh` script. To use it with default settings
(current kernel, ext4 filesystem):

```sh
./run-tests.sh
```

Custom settings can be controlled through command line flags, for example:

```sh
./run-tests.sh --kernel path/to/my-custom-vmlinuz --filesystem-type btrfs --cargo-features unnamed-tmpfile
```
