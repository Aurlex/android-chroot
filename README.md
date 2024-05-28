# android-chroot
automates the process of creating a mountable chroot on android devices in termux

# disclaimer <3
this could be dangerous. i do not take responsibility for any damage you may cause to your device while using this.

# simple setup
1. be rooted
2. have termux installed
3. run `yes | (pkg update && pkg upgrade && pkg install rust tsu)`
4. install android-chroot by cloning the repository and building it with cargo.
5. find a tarball. this can either be a file, or a URL to download. (or both, to download to a specified path)
6. cd to the target directory.
7. run `tsu` and grant superuser permissions
8. run `android-chroot -r ./root install -p /path/to/suitable/rootfs.tar.gz -s 10G`
9. hope no errors occur.
10. run `android-chroot -r ./root start`

# extended features
- support resizing the chroot
    - danger: could cause loss of data
    - `android-chroot resize NEW_SIZE`
- support mounting and unmounting the chroot
    - `android-chroot -r ./root mount` and `android-chroot -r ./root umount`
- creation of local file `android-chroot.toml`
    - so arguments do not need to be specified at each startup
    - automated on install
