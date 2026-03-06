use crate::data::format::{SampleModel, TestStatus, TestTemperature};
use std::fmt;

#[derive(Debug)]
pub struct ChannelInfo {
    pub device: String,
    pub channel: String,
    pub project_code: String,
    pub sample_person: String,
    pub sample_model: SampleModel,
    pub sample_id: String,
    pub test_status: TestStatus,
    pub test_temperature: TestTemperature,
    pub initial_capacity: Option<f32>,
    pub current_capacity: Option<f32>,
    pub previous_cycles: Option<usize>,
    pub current_cycles: Option<usize>,
}

impl ChannelInfo {
    pub fn update_capacity(&mut self, new: f32) {
        self.current_capacity = Some(new);
    }
    pub fn update_cycles(&mut self, new: usize) {

        let prev = self.previous_cycles.unwrap_or(0);
        let curt = self.current_cycles.unwrap_or(0);

        let cont = curt - prev;

        if new > curt {
            //新的圈数大于现有圈数
            self.current_cycles = Some(new);
            self.previous_cycles = Some(prev);//如果为空就自动变为0
        } else {
            if new > cont {
                //但是新的圈数大于接续后的圈数
                self.current_cycles = Some(prev + new);//获取参数，故使用new
            } else {
                //新的圈数也小于接续后的圈数，说明开启新的200圈DCR了
                self.previous_cycles = Some(prev + 200);
                self.current_cycles = Some(prev + 200 + new);
            }
        }
    }
}

impl fmt::Display for ChannelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let capacity_str =
            |cap: &Option<f32>| cap.map_or("N/A".to_string(), |v| format!("{:.2} mAh", v));

        let cycles_str =
            |cycles: &Option<usize>| cycles.map_or("N/A".to_string(), |v| format!("{} 次", v));

        write!(
            f,
            "\n┌─────────────────────────────────────────────────────┐"
        )?;
        write!(f, "\n│ {:^47} │", "通道信息")?;
        write!(
            f,
            "\n├─────────────────────────────────────────────────────┤"
        )?;
        write!(f, "\n│ 设备: {:40} │", self.device)?;
        write!(f, "\n│ 通道: {:40} │", self.channel)?;
        write!(f, "\n│ 项目编号: {:37} │", self.project_code)?;
        write!(f, "\n│ 样品负责人: {:35} │", self.sample_person)?;
        write!(f, "\n│ 样品型号: {:37} │", self.sample_model)?;
        write!(f, "\n│ 样品ID: {:39} │", self.sample_id)?;
        write!(f, "\n│ 测试状态: {:37} │", self.test_status)?;
        write!(f, "\n│ 测试温度: {:37} │", self.test_temperature)?;
        write!(
            f,
            "\n│ 初始容量: {:37} │",
            capacity_str(&self.initial_capacity)
        )?;
        write!(
            f,
            "\n│ 当前容量: {:37} │",
            capacity_str(&self.current_capacity)
        )?;
        write!(
            f,
            "\n│ 先前循环: {:35} │",
            cycles_str(&self.previous_cycles)
        )?;
        write!(f, "\n│ 当前循环: {:37} │", cycles_str(&self.current_cycles))?;
        write!(
            f,
            "\n└─────────────────────────────────────────────────────┘"
        )
    }
}
