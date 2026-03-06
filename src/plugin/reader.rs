use anyhow::{Error, Result};
use calamine::{Reader, Xlsx, open_workbook};
use regex::Regex;
use std::path::PathBuf;

use crate::plugin::parser::Sources;

const SHENGHONG_DATA: &str = "循环数据表";
const NEBULA_DATA: &str = "循环层";
const WEICHUANG_DATA:&str = "循环数据表";

pub(crate) fn cycle_first(file_path: &PathBuf, data_type: &Sources) -> Result<(u32, f32)> {
    let mut reader: Xlsx<_> = open_workbook(file_path).expect("Cannot open file");

    return match data_type {
        Sources::Shenghong => {
            let sheet = reader
                .worksheet_range(SHENGHONG_DATA)
                .expect("Cannot find sheet range");

            let max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((3, 0))
                .expect("读取失败：错误的数据格式,请检查是否为盛洪BTS生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((3, 2))
                .expect("读取失败：错误的数据格式,请检查是否为盛洪BTS生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity))
        }
        Sources::Nebula => {
            let sheet = reader
                .worksheet_range(NEBULA_DATA)
                .expect("Cannot find sheet range");

            let mut max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((3, 0))
                .expect("读取失败：错误的数据格式,请检查是否为星云生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((3, 5))
                .expect("读取失败：错误的数据格式,请检查是否为星云生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity.abs()))//nebula is -1
        },
        Sources::Weichuang => {
            let sheet = reader
                .worksheet_range(WEICHUANG_DATA)
                .expect("Cannot find sheet range");

            let mut max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((3, 0))
                .expect("读取失败：错误的数据格式,请检查是否为伟创生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((3, 5))
                .expect("读取失败：错误的数据格式,请检查是否为伟创生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity))
        }
    };
}

pub(crate) fn cycle_last(file_path: &PathBuf, data_type: &Sources) -> Result<(u32, f32)> {
    let mut reader: Xlsx<_> = open_workbook(file_path).expect("Cannot open file");

    return match data_type {
        Sources::Shenghong => {
            let sheet = reader
                .worksheet_range(SHENGHONG_DATA)
                .expect("Cannot find sheet range");

            let mut max_row_count = sheet.rows().len();

            if max_row_count > 4 {
                max_row_count = max_row_count - 2;
            } else {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((max_row_count, 0))
                .expect("读取失败：错误的数据格式,请检查是否为盛洪BTS生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((max_row_count, 2))
                .expect("读取失败：错误的数据格式,请检查是否为盛洪BTS生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity))
        }
        Sources::Nebula => {
            let sheet = reader
                .worksheet_range(NEBULA_DATA)
                .expect("Cannot find sheet range");

            let mut max_row_count = sheet.rows().len();

            if max_row_count > 4 {
                max_row_count = max_row_count - 2;
            } else {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((max_row_count, 0))
                .expect("读取失败：错误的数据格式,请检查是否为星云生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((max_row_count, 5))
                .expect("读取失败：错误的数据格式,请检查是否为星云生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity.abs()))//nebula is -1
        },
        Sources::Weichuang => {
            let sheet = reader
                .worksheet_range(WEICHUANG_DATA)
                .expect("Cannot find sheet range");

            let mut max_row_count = sheet.rows().len();

            if max_row_count > 4 {
                max_row_count = max_row_count - 2;
            } else {
                return Err(Error::msg("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count: u32 = sheet
                .get((max_row_count, 0))
                .expect("读取失败：错误的数据格式,请检查是否为伟创生成")
                .to_string().parse()?;

            let output_capacity: f32 = sheet
                .get((max_row_count, 5))
                .expect("读取失败：错误的数据格式,请检查是否为伟创生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity))
        }
    };
}

// 预编译正则表达式
lazy_static::lazy_static! {
    static ref WEICHUANG_PATTERN: Regex = Regex::new(r"_(\d+)_(\d+)$").unwrap();
}

const SHENGHONG_KEY: &str = "数据表";
const NEBULA_KEY: &str = "测试信息";
const WEICHUANG_KEY: &str = "通道信息表";

pub(crate) fn form_info(file_path: &PathBuf) -> Result<(Sources, String, String)> {
    let mut reader: Xlsx<_> = open_workbook(file_path).expect("Cannot open file");

    let sheet_names = reader.sheet_names();

    for sheet_name in sheet_names {
        if sheet_name.eq(SHENGHONG_KEY) {
            let sheet = reader.worksheet_range(SHENGHONG_KEY)?;
            let unit = sheet
                .get((2, 1))
                .expect("读取失败：错误的元数据格式")
                .to_string();
            let channel = sheet
                .get((3, 1))
                .expect("读取失败：错误的元数据格式")
                .to_string();

            println!("查询成功，通道号：{}-{}", unit, channel);
            return Ok((Sources::Shenghong, unit, channel));
        } else if sheet_name.eq(NEBULA_KEY) {
            let sheet = reader.worksheet_range(NEBULA_KEY)?;
            let unit = sheet
                .get((7, 0))
                .expect("读取失败：错误的元数据格式")
                .to_string();
            let channel = sheet
                .get((7, 2))
                .expect("读取失败：错误的元数据格式")
                .to_string();

            let unit = unit
                .split('.')
                .last()
                .map(|s| s.to_string())
                .unwrap_or(unit); //for ip spilt

            println!("查询成功，通道号：{}-{}", unit, channel);

            return Ok((Sources::Nebula, unit, channel));
        } else if sheet_name.eq(WEICHUANG_KEY) {
            let sheet = reader.worksheet_range(WEICHUANG_KEY)?;
            let cell = sheet
                .get((1, 1))
                .expect("读取失败：错误的元数据格式")
                .to_string();

            let result = WEICHUANG_PATTERN
                .captures(&cell)
                .map(|caps| (caps[1].to_string(), caps[2].to_string()));

            return match result {
                Some((unit, channel)) => {
                    println!("查询成功，通道号：{}-{}", unit, channel);

                    Ok((Sources::Weichuang, unit, channel))
                }
                None => Err(Error::msg("通道号匹配失败，请检查导出格式是否正确")),
            };
        }
    }

    return Err(Error::msg(format!(
        "Unable to find sheet in workbook: {}",
        file_path.display()
    )));
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{Data, Reader, Xlsx, open_workbook};
    #[test]
    fn test_form_info() {
        use std::fs;
        use std::time::Instant;

        let dir_path = PathBuf::from(r"C:\Users\admin\Desktop\循环导出测试\样本");

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

                match form_info(&path) {
                    Ok(info) => {
                        println!("  ✓ 成功: {:?}", info);
                        success_count += 1;
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
    #[test]
    fn test_form_info_2() {
        let path = PathBuf::from(r"C:\Users\admin\Desktop\循环导出测试\5\1.xlsx");
        // opens a new workbook
        let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");

        // Read whole worksheet data and provide some statistics
        if let Ok(range) = workbook.worksheet_range(NEBULA_KEY) {
            let total_cells = range.get_size().0 * range.get_size().1;
            let non_empty_cells: usize = range.used_cells().count();
            println!(
                "Found {total_cells} cells in 'Sheet1', including {non_empty_cells} non empty cells"
            );
            // alternatively, we can manually filter rows
            assert_eq!(
                non_empty_cells,
                range
                    .rows()
                    .flat_map(|r| r.iter().filter(|&c| c != &Data::Empty))
                    .count()
            );
        }
    }
}
