use anyhow::{Error, Result};
use std::path::PathBuf;

use crate::plugin::parser;
use crate::plugin::parser::Sources;
use crate::plugin::reader;
use crate::plugin::reader::get_run_time;

#[derive(Debug, Clone)]
pub struct DataForm {
    path: PathBuf,
    source: Sources,
    unit: String,
    channel: u32,
}

impl DataForm {
    pub fn init(path: &PathBuf) -> Result<DataForm> {
        let filename = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("无法获取文件名");

        let parser = parser::form_name(filename);

        if parser.is_ok() {
            let parser = parser?;
            return Ok(DataForm {
                path: path.clone(),
                source: parser.0,
                unit: parser.1.parse()?,
                channel: parser.2.parse()?,
            });
        }

        println!("文件名分析错误：{} 尝试进入文件获取信息", filename);

        let reader = reader::form_info(path);

        if reader.is_ok() {
            let reader = reader?;
            return Ok(DataForm {
                path: path.clone(),
                source: reader.0,
                unit: reader.1,
                channel: reader.2,
            });
        }

        Err(Error::msg("获取通道与设备号失败,读取失败!"))
    }
    pub fn get_cycle_last(&self) -> Result<(u32, f32)> {
        let (cycle_count, cycle_data) = reader::cycle_last(&self.path, &self.source)?;

        return Ok((cycle_count, cycle_data));
    }
    pub fn get_cycle_first(&self) -> Result<(u32, f32)> {
        let (cycle_count, cycle_data) = reader::cycle_first(&self.path, &self.source)?;

        return Ok((cycle_count, cycle_data));
    }
    pub fn get_form_path(&self) -> PathBuf {
        self.path.clone()
    }
    pub fn channel_eq(&self, other: &str) -> bool {
        let channel = format!("{}-{}", self.unit, self.channel);
        return if channel == other { true } else { false };
    }
    pub fn get_channel(&self) -> String {
        let channel = format!("{}-{}", self.unit, self.channel);
        return channel;
    }
    pub fn get_run_time(&self) -> Result<String> {
        let time = get_run_time(&self.path, &self.source);
        return time;
    }
}

#[cfg(test)]
mod tests {
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
                println!(
                    "\n[{}/{}] 处理: {}",
                    i + 1,
                    entries.len(),
                    path.file_name().unwrap().to_string_lossy()
                );

                match DataForm::init(&path) {
                    Ok(info) => {
                        println!("  ✓ 成功: {:?}", info);
                        success_count += 1;

                        //test for read cycle data
                        let result = info.get_cycle_last();
                        // let result = info.get_run_time();
                        println!("读取结果: {:?}", result);
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
