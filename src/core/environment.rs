use crate::constants::{RECOMMENDED_DISK_GB, RECOMMENDED_MEMORY_GB, REQUIRED_JAVA_VERSION};
use crate::error::{Result, TronCtlError};
use crate::utils::{fs, permissions};
use std::path::Path;
use std::process::Command;
use sysinfo::System;
use tracing::{info, warn};

pub struct EnvironmentChecker;

impl EnvironmentChecker {
    pub fn check_all() -> Result<()> {
        Self::check_permissions()?;
        Self::check_java_version()?;
        Self::check_memory()?;
        Self::check_disk_space()?;
        Ok(())
    }

    fn check_permissions() -> Result<()> {
        permissions::check_root()?;
        info!("权限检查通过 (root)");
        Ok(())
    }

    fn check_java_version() -> Result<()> {
        let output = Command::new("java")
            .arg("-version")
            .output()
            .map_err(|_| TronCtlError::ConfigError("Java 未安装或不在 PATH 中".to_string()))?;

        let version_str = String::from_utf8_lossy(&output.stderr);

        if version_str.contains("1.8") || version_str.contains("\"8\"") {
            info!("Java 版本检查通过 (1.8)");
            Ok(())
        } else {
            let current_version = version_str
                .lines()
                .next()
                .unwrap_or("unknown")
                .to_string();

            Err(TronCtlError::IncompatibleJavaVersion {
                required: REQUIRED_JAVA_VERSION.to_string(),
                current: current_version,
            })
        }
    }

    fn check_memory() -> Result<()> {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total_gb = sys.total_memory() / (1024 * 1024 * 1024);

        if total_gb < RECOMMENDED_MEMORY_GB {
            warn!(
                "内存不足: 推荐 {}GB, 当前 {}GB",
                RECOMMENDED_MEMORY_GB, total_gb
            );
        } else {
            info!("内存检查通过 ({}GB)", total_gb);
        }

        Ok(())
    }

    fn check_disk_space() -> Result<()> {
        let available_gb = fs::get_disk_free_space(Path::new("/"))?;

        if available_gb < RECOMMENDED_DISK_GB {
            warn!(
                "磁盘空间不足: 推荐 {}GB, 当前 {}GB",
                RECOMMENDED_DISK_GB, available_gb
            );
        } else {
            info!("磁盘空间检查通过 ({}GB)", available_gb);
        }

        Ok(())
    }
}
