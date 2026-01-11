use crate::core::ProcessManager;
use crate::error::Result;

pub fn execute(force: bool) -> Result<()> {
    ProcessManager::stop(force)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_when_no_node_running() {
        let result = execute(false);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_force_when_no_node_running() {
        let result = execute(true);
        assert!(result.is_err());
    }
}
