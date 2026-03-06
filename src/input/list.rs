use anyhow::Result;
use calamine::{Data, Reader, Xlsx, open_workbook};
use std::path::PathBuf;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::data::format::{SampleModel, TestStatus, TestTemperature};
use crate::data::list::ChannelInfo;

const CYCLE_SHEET: &str = "循环中";

fn xlsx_reader(path: &PathBuf) -> Result<Vec<ChannelInfo>> {
    let mut reader: Xlsx<_> = open_workbook(path).expect("Cannot open file");

    let sheet = reader
        .worksheet_range(CYCLE_SHEET)
        .expect("Cannot find sheet range");

    let mut channel_list: Vec<ChannelInfo> = Vec::new();

    //device,channel,project_code,sample_person,sample_model,sample_id,test_status,test_temperature,initial_capacity,current_capacity,previous_cycles,current_cycles
    let mut option_code = [
        None, None, None, None, None, None, None, None, None, None, None, None, None,
    ];

    for (index, row) in sheet.rows().enumerate() {
        //init read sheet format
        //resign for title
        if index == 0 {
            for (index, cell) in row.iter().enumerate() {
                let text = cell.to_string();
                match text.as_str() {
                    "上位机" => option_code[0] = Some(index),
                    "测试通道" => option_code[1] = Some(index),
                    "申请单号" => option_code[2] = Some(index),
                    "送样人" => option_code[3] = Some(index),
                    "电芯型号" => option_code[4] = Some(index),
                    "电芯编号" => option_code[5] = Some(index),
                    "测试状态" => option_code[6] = Some(index),
                    "测试温度" => option_code[7] = Some(index),
                    "测试方法" => option_code[7] = Some(index), //self match
                    "初始容量" => option_code[8] = Some(index),
                    "当前容量" => option_code[9] = Some(index),
                    "接续前圈数" => option_code[10] = Some(index),
                    "当前圈数" => option_code[11] = Some(index),
                    _ => {}
                }
            }

            println!("{:?}", option_code);
        } else {
            let device = row.get(option_code[0].expect("表格格式非法：没有上位机信息"));
            if device.is_some() {
                let device = device.unwrap().to_string();
                let channel = row
                    .get(option_code[1].expect("表格格式非法：没有测试通道信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let project_code = row
                    .get(option_code[2].expect("表格格式非法：没有申请单号信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let sample_person = row
                    .get(option_code[3].expect("表格格式非法：没有送样人信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let sample_model = row
                    .get(option_code[4].expect("表格格式非法：没有电芯型号信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let sample_id = row
                    .get(option_code[5].expect("表格格式非法：没有电芯编号信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let test_status = row
                    .get(option_code[6].expect("表格格式非法：没有测试状态信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let test_temperature = row
                    .get(option_code[7].expect("表格格式非法：没有测试温度信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let initial_capacity = row
                    .get(option_code[8].expect("表格格式非法：没有初始容量信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let current_capacity = row
                    .get(option_code[9].expect("表格格式非法：没有当前容量信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let previous_cycles = row
                    .get(option_code[10].expect("表格格式非法：没有接续前圈数信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();
                let current_cycles = row
                    .get(option_code[11].expect("表格格式非法：没有当前圈数信息"))
                    .unwrap_or(&Data::Empty)
                    .to_string();

                let sample_model = SampleModel::from(&sample_model);
                let test_status = TestStatus::from(&test_status);
                let test_temperature = TestTemperature::from(&test_temperature);

                let initial_capacity = initial_capacity.parse::<f32>().ok();
                let current_capacity = current_capacity.parse::<f32>().ok();
                let previous_cycles = previous_cycles.parse::<usize>().ok();
                let current_cycles = current_cycles.parse::<usize>().ok();

                let channel_info = ChannelInfo {
                    device,
                    channel,
                    project_code,
                    sample_person,
                    sample_model,
                    sample_id,
                    test_status,
                    test_temperature,
                    initial_capacity,
                    current_capacity,
                    previous_cycles,
                    current_cycles,
                };

                channel_list.push(channel_info);
            } else {
                break;
            }
        }
    }

    println!("{}", channel_list.pretty_print());

    Ok(channel_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_list() {
        let path =
            PathBuf::from(r"C:\Users\admin\Desktop\循环导出20251113\西侧循环电芯跟踪表.xlsx");
        let result = xlsx_reader(&path);
        assert!(result.is_ok());
    }
}

// 为 Vec<ChannelInfo> 提供漂亮的表格打印
pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}

impl PrettyPrint for Vec<ChannelInfo> {
    fn pretty_print(&self) -> String {
        if self.is_empty() {
            return "无通道数据".to_string();
        }

        // 定义各列显示宽度（以中文字符宽度为基准）
        let col_widths = [4, 12, 8, 14, 12, 10, 12, 12, 12, 12, 12, 12];

        // 计算表格总宽度
        let total_width: usize = col_widths.iter().sum::<usize>() + col_widths.len() + 1;

        let mut output = String::new();

        // 顶部边框
        output.push('┌');
        for (i, &width) in col_widths.iter().enumerate() {
            output.push_str(&"─".repeat(width));
            if i < col_widths.len() - 1 {
                output.push('┬');
            }
        }
        output.push_str("┐\n");

        // 表格标题
        let headers = [
            "ID",
            "设备",
            "通道",
            "项目编号",
            "样品负责人",
            "样品型号",
            "测试状态",
            "测试温度",
            "初始容量",
            "当前容量",
            "已循环次数",
        ];

        output.push('│');
        for (i, (header, &width)) in headers.iter().zip(col_widths.iter()).enumerate() {
            let header_width = header.width();
            let padding = width.saturating_sub(header_width);
            let left_padding = padding / 2;
            let right_padding = padding - left_padding;

            output.push_str(&" ".repeat(left_padding));
            output.push_str(header);
            output.push_str(&" ".repeat(right_padding));
            output.push('│');
        }
        output.push('\n');

        // 标题分隔线
        output.push('├');
        for (i, &width) in col_widths.iter().enumerate() {
            output.push_str(&"─".repeat(width));
            if i < col_widths.len() - 1 {
                output.push('┼');
            }
        }
        output.push_str("┤\n");

        // 表格数据行
        for (row_idx, info) in self.iter().enumerate() {
            // 准备每一列的数据
            let columns = [
                (row_idx + 1).to_string(),
                info.device.clone(),
                info.channel.clone(),
                info.project_code.clone(),
                info.sample_person.clone(),
                info.sample_model.to_string(),
                info.test_status.to_string(),
                info.test_temperature.to_string(),
                info.initial_capacity
                    .map_or("N/A".to_string(), |v| format!("{:.1}mAh", v)),
                info.current_capacity
                    .map_or("N/A".to_string(), |v| format!("{:.1}mAh", v)),
                info.previous_cycles
                    .map_or("N/A".to_string(), |v| format!("{}次", v)),
            ];

            output.push('│');
            for (i, ((col_data, &width), header)) in columns
                .iter()
                .zip(col_widths.iter())
                .zip(headers.iter())
                .enumerate()
            {
                let data_str = col_data.as_str();
                let data_width = data_str.width();

                // 如果数据过长，进行截断
                let display_str = if data_width > width {
                    // 找到合适的截断位置
                    let mut chars = data_str.chars();
                    let mut display_width = 0;
                    let mut truncated = String::new();

                    while let Some(c) = chars.next() {
                        let char_width = c.width().unwrap_or(0);
                        if display_width + char_width + 3 > width {
                            // +3 为 "..."
                            truncated.push_str("...");
                            break;
                        }
                        truncated.push(c);
                        display_width += char_width;
                    }
                    truncated
                } else {
                    data_str.to_string()
                };

                let display_width = display_str.width();
                let padding = width.saturating_sub(display_width);

                // 不同列的对齐方式：ID右对齐，数值右对齐，其他左对齐
                match i {
                    0 | 8 | 9 | 10 | 11 => {
                        // ID和数值列：右对齐
                        output.push_str(&" ".repeat(padding));
                        output.push_str(&display_str);
                    }
                    _ => {
                        // 文本列：左对齐
                        output.push_str(&display_str);
                        output.push_str(&" ".repeat(padding));
                    }
                }
                output.push('│');
            }
            output.push('\n');
        }

        // 底部边框
        output.push('└');
        for (i, &width) in col_widths.iter().enumerate() {
            output.push_str(&"─".repeat(width));
            if i < col_widths.len() - 1 {
                output.push('┴');
            }
        }
        output.push_str("┘\n");

        // 统计信息
        let long_count = self
            .iter()
            .filter(|i| matches!(i.test_status, TestStatus::Long))
            .count();
        let offline_count = self
            .iter()
            .filter(|i| matches!(i.test_status, TestStatus::None))
            .count();
        let total_capacity: f32 = self.iter().filter_map(|i| i.current_capacity).sum();

        output.push_str(&format!(
            "\n📊 统计信息: 共 {} 个通道 | 长期测试: {} 个 | 离线: {} 个 | 总容量: {:.2} mAh\n",
            self.len(),
            long_count,
            offline_count,
            total_capacity
        ));

        // 温度分布统计
        let mut temp_stats = std::collections::HashMap::new();
        for info in self {
            let temp_str = info.test_temperature.to_string();
            *temp_stats.entry(temp_str).or_insert(0) += 1;
        }

        if !temp_stats.is_empty() {
            output.push_str("🌡️ 温度分布: ");
            let temp_list: Vec<String> = temp_stats
                .iter()
                .map(|(temp, count)| format!("{}×{}", temp, count))
                .collect();
            output.push_str(&temp_list.join(" | "));
            output.push('\n');
        }

        // 样品型号分布统计
        let mut model_stats = std::collections::HashMap::new();
        for info in self {
            let model_str = info.sample_model.to_string();
            *model_stats.entry(model_str).or_insert(0) += 1;
        }

        if !model_stats.is_empty() {
            output.push_str("📦 样品型号分布: ");
            let model_list: Vec<String> = model_stats
                .iter()
                .map(|(model, count)| format!("{}×{}", model, count))
                .collect();
            output.push_str(&model_list.join(" | "));
            output.push('\n');
        }

        output
    }
}
