use crate::error::{Result, TronCtlError};
use nix::unistd::Uid;

pub fn check_root() -> Result<()> {
    if !Uid::effective().is_root() {
        return Err(TronCtlError::InsufficientPermissions);
    }
    Ok(())
}

pub fn is_root() -> bool {
    Uid::effective().is_root()
}
