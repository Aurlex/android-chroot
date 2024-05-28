#!/bin/fish

if test $USER != "root"
    echo "This script must be run as root."
    return 1
end

if test (not string match -q -- "*com.termux*" $PREFIX)
    echo "Are you silly?"
    return 1
end

if test -e "disk.img"
    echo "disk.img already exists."
    return 1
end

if test -e "root"
    echo "root/ already exists."
    return 1
end

mkdir root

read -l -P "Enter filesystem size (in gigabytes) (default 30): " size

set size (string trim $size)

if test $size = ""
    set size "30"
end

echo "Clearing other loopback devices..."
losetup -D

if test $status -ne 0
    echo "Could not clear other loopback devices."
    return 1
end

echo "Allocating disk space..."
truncate -s {$size}G disk.img

if test $status -ne 0
    echo "Could not allocate disk space."
    return 1
end

read -l -P "Enter filesystem type (default: ext4): " fs

set fs (string trim $fs)

if test $fs = ""
    set fs "ext4"
end

if test (command -v mkfs.$fs)
    echo "Formatting disk space..."
    mkfs.$fs disk.img
else if test (command -v make_$fs)
    echo "Formatting disk space..."
    make_$fs disk.img
else
    echo "That filesystem is not supported by your device. Try another"
    return 1
end

if test $status -ne 0
    echo "Failed to format disk space."
    return 1
end

set delete_it "y"

if test -e "rootfs"
    read -l -P "rootfs tarball already exists. Delete it? [Y/n]: " delete_it
    set delete_it (string lower delete_it)
    set delte_it (string trim delete_it)
end
switch $delete_it
case "y"
    read -l -P "Enter a link to a tarball, or use the default (ArchLinuxARM): " tarball

    set tarball (string trim $tarball)
    if test $tarball = ""
        set tarball "http://os.archlinuxarm.org/os/ArchLinuxARM-$(arch)-latest.tar.gz"
    end

    echo "Downloading tarball..."
    wget -O rootfs $tarball

    if test $status -ne 0
        echo "Could not download tarball."
        return 1
    end
case "n"
    echo "Keeping rootfs."
case "*"
    echo "Not a valid option."
    return 1
end

echo "Assigning loopback device to disk.img..."
losetup -f disk.img

if test $status -ne 0
    echo "Could not assign loopback device."
    return 1
end

set mount_point (losetup -a | grep $PWD/disk.img | awk '{print $1}' | tr -d :)

if test $status -ne 0
    echo "Failed to find valid mount point."
    return 1
end

# set mount_point
switch $mount_point
case "* *"
    echo "More than one mount point found. This should not happen."
    return 1
case ""
    echo "No paths found. This should not happen."
    return 1
end

echo "Mounting disk.img..."
mount -t $fs $mount_point root

if test $status -ne 0
    echo "Failed to mount disk.img."
    return
end

cd root
tar -xf ../rootfs

if test $status -ne 0
    echo "Failed to extract tarball."
    return 1
end

rm ../rootfs
cd ../

rm root/etc/resolv.conf
cat $PREFIX/etc/resolv.conf > root/etc/resolv.conf
cat $hostname > root/etc/hostname > root/etc/hostname
cat $PREFIX/etc/hosts >  root/etc/hosts

mkdir root/sdcard

echo \
"#!/bin/fish
losetup -f disk.img
set mount_point (losetup -a | grep \$PWD/disk.img | awk '{print \$1}' | tr -d :)
mount -t $fs \$mount_point root
mount -o bind /dev root/dev
mount -t proc proc root/proc
mount -t sysfs sysfs root/sys
mount -t tmpfs tmpfs root/tmp
mount -t devpts devpts root/dev/pts
mount -o bind /sdcard root/sdcard" > mount_arch.fish

echo \
"#!/bin/fish
umount -l root/sdcard
umount -l root/dev/pts
umount -l root/tmp
umount -l root/sys
umount -l root/proc
umount -l root/dev
umount -l root
losetup -D" > unmount_arch.fish

echo \
"#!/bin/fish
fish mount_arch.fish
unshare --uts env -i TERM=xterm-256color chroot root (awk -F: -v user='root' '\$1 == user {print \$NF}' root/etc/passwd)
fish unmount_arch.fish" > start_arch.fish

chmod +x start_arch.fish
chmod +x unmount_arch.fish
chmod +x mount_arch.fish

fish unmount_arch.fish
