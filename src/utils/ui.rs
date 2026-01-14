use indicatif::{ProgressBar, ProgressStyle};

pub fn create_download_progress_bar(total_size: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] \
                 {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .expect("Invalid progress bar template")
            .progress_chars("#>-"),
    );

    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

#[allow(dead_code)]
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_message(message.to_string());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .expect("Invalid spinner template"),
    );
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_download_progress_bar() {
        let pb = create_download_progress_bar(1000);
        assert_eq!(pb.length().unwrap(), 1000);
    }

    #[test]
    fn test_create_download_progress_bar_zero() {
        let pb = create_download_progress_bar(0);
        assert_eq!(pb.length().unwrap(), 0);
    }

    #[test]
    fn test_create_download_progress_bar_large() {
        let pb = create_download_progress_bar(u64::MAX);
        assert_eq!(pb.length().unwrap(), u64::MAX);
    }

    #[test]
    fn test_create_spinner() {
        let spinner = create_spinner("测试中...");
        assert_eq!(spinner.message(), "测试中...");
    }

    #[test]
    fn test_create_spinner_empty_message() {
        let spinner = create_spinner("");
        assert_eq!(spinner.message(), "");
    }
}
