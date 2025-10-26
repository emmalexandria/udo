use nix::unistd::{Uid, User};

pub fn get_user_by_id(uid: Uid) -> Option<User> {
    User::from_uid(uid).ok().flatten()
}

pub fn get_user(name: &str) -> Option<User> {
    if name == "root" {
        Some(get_root_user())
    } else {
        User::from_name(name).ok().flatten()
    }
}

/// This function attempts to return the root user. Note that on BSDs there can be two users with
/// UID 0, 'root' and 'toor'. This function will attempt to return root
///
/// The function will first attempt to construct a user from the `root` username and check if it
/// has UID 0, then it will construct the user from UID 0 and check if that's root. If it's `toor`,
/// you get `toor`. Panics if no user can be constructed from UID 0.
pub fn get_root_user() -> User {
    let from_name = User::from_name("root");

    if let Some(u) = from_name.ok().flatten() {
        return u;
    }

    User::from_uid(Uid::from_raw(0))
        .ok()
        .flatten()
        .expect("Failed to get root user with UID 0")
}
