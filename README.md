# android-chroot
automates the process of creating a mountable chroot on android devices in termux

# disclaimer <3
this could be dangerous. i do not take responsibility for any damage you may cause to your device while using this.

# simple setup
### Requires: Termux, Root

1. run `curl --proto '=https' --tlsv1.2 -sSfL https://github.com/Aurlex/android-chroot/releases/latest/download/quickstart.sh | sh`
2. find a tarball. this can either be a file, or a URL to download
  - for instance: `http://ca.us.mirror.archlinuxarm.org/os/ArchLinuxARM-SYSTEMARCH-latest.tar.gz`
    - replace SYSTEMARCH with your system architecture. find it by running `dpkg --print-architecture`
3. run `tsu` and grant superuser permissions
4. run `android-chroot -r ./root install -p /path/to/suitable/rootfs.tar.gz -s 10G`
  - you can also substitute these values: `-s` for size, `-r` for root path, `-p` for rootfs path, or if you have a url, swap `-p` for `-u`
5. run `android-chroot -r ./root start`

# extended features
- [ ] support X11
- [ ] support resizing the chroot
    - danger: could cause loss of data
    - `android-chroot -r ./root resize NEW_SIZE`
- [X] support mounting and unmounting the chroot
    - `android-chroot -r ./root mount` and `android-chroot -r ./root umount`