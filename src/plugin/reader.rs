use anyhow::{anyhow, Context, Error, Result};
use calamine::{Reader, Xlsx, open_workbook};
use regex::Regex;
use std::path::PathBuf;

use crate::plugin::parser::Sources;

const SHENGHONG_DATA: &str = "循环数据表";
const NEBULA_DATA: &str = "循环层";
const WEICHUANG_DATA:&str = "循环数据表";
const WEICHUANG_INFO:&str = "数据表";

pub(crate) fn cycle_first(file_path: &PathBuf, data_type: &Sources) -> Result<(u32, f32)> {
    let mut reader: Xlsx<_> = open_workbook(file_path)
        .with_context(|| format!("无法打开文件: {}", file_path.display()))?;

    match data_type {
        Sources::Shenghong => {
            let sheet = reader
                .worksheet_range(SHENGHONG_DATA)
                .with_context(|| "无法找到工作表范围，请检查是否为盛洪BTS生成")?;

            let max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(anyhow!("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count_cell = sheet.get((3, 0))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第1列为空，请检查是否为盛洪BTS生成"))?;
            let cycle_count: u32 = cycle_count_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析循环次数失败: {}", cycle_count_cell))?;

            let output_capacity_cell = sheet.get((3, 2))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第3列为空，请检查是否为盛洪BTS生成"))?;
            let output_capacity: f32 = output_capacity_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析输出容量失败: {}", output_capacity_cell))?;

            Ok((cycle_count, output_capacity))
        }
        Sources::Nebula => {
            let sheet = reader
                .worksheet_range(NEBULA_DATA)
                .with_context(|| "无法找到工作表范围，请检查是否为星云生成")?;

            let max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(anyhow!("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count_cell = sheet.get((3, 0))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第1列为空，请检查是否为星云生成"))?;
            let cycle_count: u32 = cycle_count_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析循环次数失败: {}", cycle_count_cell))?;

            let output_capacity_cell = sheet.get((3, 5))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第6列为空，请检查是否为星云生成"))?;
            let output_capacity: f32 = output_capacity_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析输出容量失败: {}", output_capacity_cell))?;

            Ok((cycle_count, output_capacity.abs()))
        }
        Sources::Weichuang => {
            let sheet = reader
                .worksheet_range(WEICHUANG_DATA)
                .with_context(|| "无法找到工作表范围，请检查是否为伟创生成")?;

            let max_row_count = sheet.rows().len();

            if max_row_count < 4 {
                return Err(anyhow!("循环次数过少数据失真，取消获取过程"));
            }

            let cycle_count_cell = sheet.get((3, 0))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第1列为空，请检查是否为伟创生成"))?;
            let cycle_count: u32 = cycle_count_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析循环次数失败: {}", cycle_count_cell))?;

            let output_capacity_cell = sheet.get((3, 2))
                .ok_or_else(|| anyhow!("读取失败：错误的数据格式，第4行第3列为空，请检查是否为伟创生成"))?;
            let output_capacity: f32 = output_capacity_cell
                .to_string()
                .parse()
                .with_context(|| format!("解析输出容量失败: {}", output_capacity_cell))?;

            Ok((cycle_count, output_capacity))
        }
    }
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

            // let cycle_count = sheet
            //     .get((max_row_count, 1))
            //     .expect("读取失败：错误的数据格式,请检查是否为星云生成")
            //     .to_string();
            //
            // println!("output:{}",cycle_count);

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
                .get((max_row_count, 2))
                .expect("读取失败：错误的数据格式,请检查是否为伟创生成")
                .to_string().parse()?;

            Ok((cycle_count, output_capacity))
        }
    };
}

pub(crate) fn get_run_time(file_path: &PathBuf, data_type: &Sources) -> Result<String> {
    let mut reader: Xlsx<_> = open_workbook(file_path).expect("Cannot open file");
    return match data_type {
        Sources::Shenghong => {
            let sheet = reader.worksheet_range(WEICHUANG_INFO).expect("Cannot find sheet range");

            let time:f32 = sheet.get((6, 1)).expect("读取失败：错误的数据格式,请检查是否为盛洪BTS生成").to_string().parse()?;

            let time = time * 24.0;

            //cut
            let time = time.floor().to_string();

            Ok(time)
        }
        _ => {
            Ok(String::new())
        }
    }
}

// 预编译正则表达式
lazy_static::lazy_static! {
    static ref WEICHUANG_PATTERN: Regex = Regex::new(r"_(\d+)_(\d+)$").unwrap();
}

const SHENGHONG_KEY: &str = "数据表";
const NEBULA_KEY: &str = "测试信息";
const WEICHUANG_KEY: &str = "通道信息表";

pub(crate) fn form_info(file_path: &PathBuf) -> Result<(Sources, String, u32)> {
    let mut reader: Xlsx<_> = open_workbook(file_path).expect("Cannot open file");

    let sheet_names = reader.sheet_names();

    for sheet_name in sheet_names {
        if sheet_name.eq(SHENGHONG_KEY) {
            let sheet = reader.worksheet_range(SHENGHONG_KEY)?;
            let unit = sheet
                .get((2, 1))
                .expect("读取失败：错误的元数据格式")
                .to_string().parse()?;
            let channel = sheet
                .get((3, 1))
                .expect("读取失败：错误的元数据格式")
                .to_string().parse()?;

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
                .to_string().parse()?;

            let unit = unit
                .split('.')
                .last()
                .map(|s| s.to_string())
                .unwrap_or(unit).parse()?; //for ip spilt

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
                    let unit = unit.parse()?;
                    let channel = channel.parse()?;
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