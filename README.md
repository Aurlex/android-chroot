# android-chroot
automates the process of creating a mountable chroot, designed for android

# disclaimer <3
this could be dangerous. i do not take responsibility for any damage you may cause to your device while using this.

# simple setup
1. install android-chroot by cloning the repository and installing it with cargo.
2. find a tarball. this can either be a file, or a URL to download. (or both, to download to a specified path)
3. cd to the target directory.
4. run `android-chroot -r ./root install -p /path/to/suitable/rootfs.tar.gz -s 10G`
5. hope no errors occur.
6. run `android-chroot start` in the target directory

# extended features
- support resizing the chroot
    - danger: could cause loss of data
    - `android-chroot resize NEW_SIZE`
- support mounting and unmounting the chroot
    - `android-chroot -r ./root mount` and `android-chroot -r ./root umount`
- creation of local file `android-chroot.toml`
    - so arguments do not need to be specified at each startup
    - automated on install