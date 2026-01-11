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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ensure_dir_exists_creates_new() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_subdir");

        assert!(!test_path.exists());

        ensure_dir_exists(&test_path).await.unwrap();

        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[tokio::test]
    async fn test_ensure_dir_exists_already_exists() {
        let temp_dir = TempDir::new().unwrap();

        let result = ensure_dir_exists(temp_dir.path()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ensure_dir_exists_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a").join("b").join("c");

        ensure_dir_exists(&nested_path).await.unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.is_dir());
    }

    #[tokio::test]
    async fn test_ensure_parent_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("file.txt");

        ensure_parent_exists(&file_path).await.unwrap();

        assert!(file_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_ensure_parent_exists_no_parent() {
        let path = std::path::PathBuf::from("/");
        let result = ensure_parent_exists(&path).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_disk_free_space() {
        let path = Path::new("/tmp");
        let space = get_disk_free_space(path).unwrap();
        assert!(space >= 0);
    }

    #[test]
    fn test_get_disk_free_space_root() {
        let path = Path::new("/");
        let space = get_disk_free_space(path).unwrap();
        assert!(space >= 0);
    }
}
