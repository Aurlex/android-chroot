1. rewrite config validation to use separate structs for Arguments & Config for clearer feedback.
2. refactor some functions that are similar (e.g: mount_fs, mount_loopback, mount_bind)