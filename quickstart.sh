yes | (pkg update && pkg upgrade && pkg install rust tsu git)
cargo install --git https://www.github.com/Aurlex/android-chroot.git
export PATH=$PATH:$HOME/.cargo/bin