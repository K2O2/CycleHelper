use anyhow::{Error, Result};
use regex::Regex;

// 预编译正则表达式
lazy_static::lazy_static! {
    static ref SHENGHONG_PATTERN1: Regex = Regex::new(r"^UN(\d+)_CN(\d+)_").unwrap();
    static ref SHENGHONG_PATTERN2: Regex = Regex::new(r"^CN(\d+)_UN(\d+)_").unwrap();

    static ref WEICHUANG_PATTERN1: Regex = Regex::new(r"^SN(\d+)_Ch(\d+)_").unwrap();
    static ref WEICHUANG_PATTERN2: Regex = Regex::new(r"^Ch(\d+)_SN(\d+)_").unwrap();

    static ref NEBULA_PATTERN: Regex = Regex::new(r"^([^_]+)_(\d+)_").unwrap();
}

#[derive(Debug, PartialEq)]
#[derive(Clone)]
pub(crate) enum Sources {
    Shenghong,
    Weichuang,
    Nebula,
}

pub(crate) fn form_name(filename: &str) -> Result<(Sources, String, String)> {
    // 尝试匹配Shenghong格式（两种顺序）
    if let Some(caps) = SHENGHONG_PATTERN1.captures(filename) {
        return Ok((Sources::Shenghong, caps[1].to_string(), caps[2].to_string()));
    }

    if let Some(caps) = SHENGHONG_PATTERN2.captures(filename) {
        return Ok((Sources::Shenghong, caps[2].to_string(), caps[1].to_string()));
    }

    // 尝试匹配Weichuang格式（两种顺序）
    if let Some(caps) = WEICHUANG_PATTERN1.captures(filename) {
        return Ok((Sources::Weichuang, caps[1].to_string(), caps[2].to_string()));
    }

    if let Some(caps) = WEICHUANG_PATTERN2.captures(filename) {
        return Ok((Sources::Weichuang, caps[2].to_string(), caps[1].to_string()));
    }

    // 尝试匹配Nebula格式
    if let Some(caps) = NEBULA_PATTERN.captures(filename) {
        return Ok((Sources::Nebula, caps[1].to_string(), caps[2].to_string()));
    }

    // 如果都不匹配，返回错误
    Err(Error::msg(format!(
        "Unable to parse filename: {}",
        filename
    )))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    #[test]
    fn test_form_init() {
        use std::fs;
        use std::time::Instant;

        let dir_path = PathBuf::from(r"E:\share\cycles\20260202\循环数据采集20260202\D2-25");

        println!("开始遍历目录: {:?}", dir_path);

        let entries: Vec<_> = fs::read_dir(&dir_path)
            .expect("无法读取目录")
            .filter_map(Result::ok)
            .collect();

        let mut success_count = 0;
        let mut fail_count = 0;
        let start_time = Instant::now();

        for (i, entry) in entries.iter().enumerate() {
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "xlsx") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                println!(
                    "\n[{}/{}] 处理: {}",
                    i + 1,
                    entries.len(),
                    file_name
                );

                match form_name(&file_name) {
                    Ok(info) => {
                        println!("  ✓ 成功: {:?}", info);
                        success_count += 1;

                        //test for read cycle data
                        // let result = info.get_cycle_last();
                        // let result = info.get_run_time();
                        // println!("读取结果: {:?}", result);
                    }
                    Err(e) => {
                        println!("  ✗ 错误: {}", e);
                        fail_count += 1;
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        println!("\n=== 处理完成 ===");
        println!("总文件数: {}", entries.len());
        println!("成功: {} 个", success_count);
        println!("失败: {} 个", fail_count);
        println!("耗时: {:.2?}", duration);
    }
}