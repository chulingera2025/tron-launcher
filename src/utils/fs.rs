use crate::error::Result;
use std::path::Path;
use tokio::fs;

pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path).await?;
    }
    Ok(())
}

pub async fn ensure_parent_exists(file_path: &Path) -> Result<()> {
    if let Some(parent) = file_path.parent() {
        ensure_dir_exists(parent).await?;
    }
    Ok(())
}

pub fn get_disk_free_space(path: &Path) -> Result<u64> {
    use sysinfo::Disks;

    let disks = Disks::new_with_refreshed_list();

    for disk in &disks {
        let mount_point = disk.mount_point();
        if path.starts_with(mount_point) {
            return Ok(disk.available_space() / (1024 * 1024 * 1024));
        }
    }

    Ok(0)
}
