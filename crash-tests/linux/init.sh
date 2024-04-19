#!/bin/sh -e

echo 'Mounting /proc'
mount -t proc none /proc
echo 'Mounting /sys'
mount -t sysfs none /sys
echo 'Mounting /test'
fstype=$(grep -Eo 'test.fs=\w+' /proc/cmdline | cut -d= -f2)
modprobe "$fstype" || true
mount -t "$fstype" /dev/sdb /test

if grep -q test.verify /proc/cmdline; then
  echo 'Verifying test file contents'
  echo '-----'
  xxd /test/file
  echo '-----'
  poweroff -f
fi

echo 'Running test binary'
atomic-write-file-test

echo 'Done'
