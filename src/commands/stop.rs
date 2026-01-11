use crate::core::ProcessManager;
use crate::error::Result;

pub fn execute(force: bool) -> Result<()> {
    ProcessManager::stop(force)
}
