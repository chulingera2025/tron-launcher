use crate::error::{Result, TronCtlError};
use nix::unistd::Uid;

pub fn check_root() -> Result<()> {
    if !Uid::effective().is_root() {
        return Err(TronCtlError::InsufficientPermissions);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn is_root() -> bool {
    Uid::effective().is_root()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_root_consistency() {
        let root1 = is_root();
        let root2 = is_root();
        assert_eq!(root1, root2);
    }

    #[test]
    fn test_check_root_consistency() {
        let result1 = check_root();
        let result2 = check_root();
        assert_eq!(result1.is_ok(), result2.is_ok());
    }

    #[test]
    fn test_check_root_matches_is_root() {
        let is_root_result = is_root();
        let check_root_result = check_root();

        if is_root_result {
            assert!(check_root_result.is_ok());
        } else {
            assert!(check_root_result.is_err());
            if let Err(e) = check_root_result {
                assert!(matches!(e, TronCtlError::InsufficientPermissions));
            }
        }
    }
}
