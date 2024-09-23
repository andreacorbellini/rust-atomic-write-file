#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

status=$'\e[1;32m'
error=$'\e[1;31m'
reset=$'\e[0m'

qemu=qemu-system-$(uname -m)
kernel=/boot/vmlinuz-$(uname -r)
modules=/usr/lib/modules/$(uname -r)

busybox=$(which busybox)
init_script=init.sh

filesystem_type=ext4
cargo_features=

build_dir=build
initramfs_build_dir=$build_dir/rootfs
initramfs=$build_dir/rootfs.img
testfs=$build_dir/testfs.img
output=$build_dir/output.txt
target=$(uname -m)-unknown-linux-gnu
test_bin=$build_dir/cargo/$target/release/atomic-write-file-test

options=$(getopt --name "$0" --options k:m:f:F: --long kernel:,modules:,filesystem-type:,cargo-features: -- "$@")

eval set -- "$options"

while true; do
  case "$1" in
    -k|--kernel)
      kernel=$(readlink -f "$2")
      shift 2
      ;;
    -m|--modules)
      modules=$(readlink -f "$2")
      shift 2
      ;;
    -f|--filesystem-type)
      filesystem_type=$2
      shift 2
      ;;
    -F|--cargo-features)
      cargo_features=$2
      shift 2
      ;;
    --)
      shift
      break
      ;;
  esac
done

if [[ $# -ne 0 ]]; then
  echo "$0: extra arguments: $*" >&2
  exit 1
fi

cd "$(dirname "$(readlink -f "$0")")"

#
# Compile the test binary running atomic-write-file
#
# This needs to be a statically-linked binary because the root filesystem won't
# ship with libc
#

echo "${status}Compiling${reset} static binary at $test_bin"
RUSTFLAGS='-C target-feature=+crt-static' CARGO_TARGET_DIR=$build_dir/cargo cargo build --release --features "$cargo_features" --target "$target"

#
# Create the root filesystem and put it into an initramfs
#
# The root filesystem contains:
# - busybox (to provide /bin/sh and other basic tools)
# - the test binary
# - an init script
# - kernel modules (to support uncommon filesystems like btrfs)
#

echo "${status}Building${reset} initramfs at $initramfs"
echo "  Using busybox at $busybox"
echo "  Using modules at $modules"
echo "  Using $init_script as /sbin/init"

rm -rf "$initramfs" "$initramfs_build_dir"

echo "  ${status}Creating${reset} filesystem"

mkdir -p \
  "$initramfs_build_dir/bin" \
  "$initramfs_build_dir/dev" \
  "$initramfs_build_dir/lib" \
  "$initramfs_build_dir/proc" \
  "$initramfs_build_dir/sbin" \
  "$initramfs_build_dir/sys" \
  "$initramfs_build_dir/test" \
  "$initramfs_build_dir/usr"

ln -s ../bin "$initramfs_build_dir/usr/bin"
ln -s ../lib "$initramfs_build_dir/usr/lib"

cp "$init_script" -T "$initramfs_build_dir/sbin/init"
cp "$test_bin" -t "$initramfs_build_dir/bin"
cp "$busybox" -t "$initramfs_build_dir/bin"
"$busybox" --install -s "$initramfs_build_dir/bin"

echo "    ${status}Adding${reset} uncompressed kernel modules"

mkdir -p "$initramfs_build_dir/lib/modules"
cp -a "$modules" -t "$initramfs_build_dir/lib/modules"

include_mods=( "$filesystem_type" )

for mod in "${include_mods[@]}"; do
  while read -r mod_path; do
    if [[ "$mod_path" = *.ko.gz ]]; then
      gunzip "$mod_path" -o "$mod_path.uncompressed"
    elif [[ "$mod_path" = *.ko.xz ]]; then
      unxz "$mod_path" -o "$mod_path.uncompressed"
    elif [[ "$mod_path" = *.ko.zst ]]; then
      unzstd "$mod_path" -o "$mod_path.uncompressed"
    else
      continue
    fi
    mv "$mod_path.uncompressed" "$mod_path"
  done < <(modprobe --dirname "$initramfs_build_dir" --show-depends "$mod" | grep ^insmod | cut -d' ' -f2)
done

echo "  ${status}Creating${reset} squashfs file"

mksquashfs "$initramfs_build_dir" "$initramfs"

#
# Create the filesystem used for testing
#

echo "${status}Creating${reset} test $filesystem_type filesystem at $testfs"

dd if=/dev/zero of="$testfs" bs=1M count=200
"mkfs.$filesystem_type" "$testfs"

#
# Run the virtual machine
#
# This will create a file with atomic-write-file and then trigger a kernel
# panic
#

echo "${status}Running${reset} virtual machine to write test file"
echo "       Using qemu at $qemu"
echo "     Using kernel at $kernel"
echo "  Using initramfs at $initramfs"

"$qemu" \
  -kernel "$kernel" \
  -append "root=/dev/sda ro panic=-1 console=ttyS0 quiet test.fs=$filesystem_type" \
  -drive "index=0,media=disk,format=raw,file=$initramfs" \
  -drive "index=1,media=disk,format=raw,file=$testfs" \
  -no-reboot \
  -nographic

#
# Run the virtual machine, again, to verify the file contents
#
# We could avoid spawning a virtual machine by using tools like debugfs, but
# these tools are not available for all filesystem types (or at least I'm not
# aware of them).
#

echo "${status}Re-running${reset} virtual machine to verify test file contents"
echo "       Using qemu at $qemu"
echo "     Using kernel at $kernel"
echo "  Using initramfs at $initramfs"

"$qemu" \
  -kernel "$kernel" \
  -append "root=/dev/sda ro panic=-1 console=ttyS0 quiet test.fs=$filesystem_type test.verify" \
  -drive "index=0,media=disk,format=raw,file=$initramfs" \
  -drive "index=1,media=disk,format=raw,file=$testfs" \
  -no-reboot \
  -nographic \
  | tee "$output"

if [[ $(sed -n '/-----/, /-----/p' "$output" | xxd -r) = hello ]]; then
  echo "${status}Success${reset}"
else
  echo "${error}Failure${reset}"
  exit 1
fi
