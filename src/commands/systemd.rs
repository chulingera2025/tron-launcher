use crate::constants::{CONFIG_DIR, DATA_DIR, LOG_DIR};
use crate::error::Result;
use crate::models::TronCtlConfig;
use std::path::Path;
use tracing::info;

/// 生成并安装 systemd 服务文件
pub async fn execute(force: bool) -> Result<()> {
    info!("生成 systemd 服务文件...");

    // 检查是否已存在服务文件
    let service_path = "/etc/systemd/system/java-tron.service";
    if Path::new(service_path).exists() && !force {
        info!("服务文件已存在: {}", service_path);
        info!("如需重新生成，请使用 --force 参数");
        return Ok(());
    }

    // 读取配置
    let config_path = Path::new(CONFIG_DIR).join("tronctl.toml");
    let config: TronCtlConfig = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&content)?
    } else {
        TronCtlConfig::default()
    };

    // 生成服务文件内容
    let service_content = generate_service_file(&config);

    // 写入服务文件
    std::fs::write(service_path, service_content)?;

    info!("systemd 服务文件已生成: {}", service_path);
    info!("运行以下命令启用并启动服务:");
    info!("  sudo systemctl daemon-reload");
    info!("  sudo systemctl enable java-tron");
    info!("  sudo systemctl start java-tron");

    Ok(())
}

/// 生成 systemd 服务文件内容
fn generate_service_file(config: &TronCtlConfig) -> String {
    let fullnode_jar = config.fullnode_jar.to_string_lossy();
    let node_config = config.node_config.to_string_lossy();
    let data_dir = config.data_dir.to_string_lossy();
    let jvm_opts = format!("-Xms{} -Xmx{}", config.jvm_min_heap, config.jvm_max_heap);

    indoc::formatdoc!(
        r#"
        [Unit]
        Description=TRON FullNode Service
        Documentation=https://github.com/tronprotocol/java-tron
        After=network-online.target
        Wants=network-online.target

        [Service]
        Type=simple
        User=root
        WorkingDirectory={DATA_DIR}
        Environment="JAVA_OPTS={jvm_opts}"
        ExecStart=/usr/bin/java $JAVA_OPTS -jar {fullnode_jar} -c {node_config} -d {data_dir}
        ExecStop=/usr/bin/kill -SIGTERM $MAINPID
        Restart=on-failure
        RestartSec=10
        StandardOutput=append:{LOG_DIR}/fullnode.log
        StandardError=append:{LOG_DIR}/fullnode.log

        # 安全设置
        PrivateTmp=true
        NoNewPrivileges=true
        ProtectSystem=full
        ProtectHome=true
        ReadWritePaths={DATA_DIR} {LOG_DIR}

        # 文件句柄限制（TRON 节点需要大量连接）
        LimitNOFILE=1048576

        [Install]
        WantedBy=multi-user.target
        "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_service_file() {
        let config = TronCtlConfig::default();
        let service = generate_service_file(&config);

        assert!(service.contains("[Unit]"));
        assert!(service.contains("[Service]"));
        assert!(service.contains("[Install]"));
        assert!(service.contains("Description=TRON FullNode Service"));
        assert!(service.contains("ExecStart=/usr/bin/java"));
        assert!(service.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn test_service_contains_required_directives() {
        let config = TronCtlConfig::default();
        let service = generate_service_file(&config);

        // 检查必需的指令
        assert!(service.contains("Type=simple"));
        assert!(service.contains("Restart=on-failure"));
        assert!(service.contains("ExecStop=/usr/bin/kill"));
        assert!(service.contains("PrivateTmp=true"));
        assert!(service.contains("ProtectSystem=full"));
        assert!(service.contains("LimitNOFILE=1048576"));
        assert!(service.contains("$JAVA_OPTS"));
    }

    #[test]
    fn test_service_contains_paths() {
        let config = TronCtlConfig::default();
        let service = generate_service_file(&config);

        assert!(service.contains(DATA_DIR));
        assert!(service.contains(LOG_DIR));
    }
}
