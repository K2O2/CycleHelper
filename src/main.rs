use anyhow::{Context, Error, Result};
use dialoguer::Select;
use edit_xlsx;
use edit_xlsx::{Read, WorkSheet, Workbook, Write};
use regex::Regex;
use rfd::FileDialog;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[tokio::main]
async fn main() -> Result<()> {
    let _result = interactive_mode().await;

    Ok(())
}

async fn interactive_mode() -> Result<()> {
    println!("🔋 电芯数据管理工具");
    println!("===================");

    loop {
        let options = vec![
            "1. 执行完整流程",
            "2. 创建输出目录",
            "3. 写入循环数据并复制文件",
            "4. 显示帮助",
            "5. 退出",
        ];

        let selection = Select::new()
            .with_prompt("请选择操作")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => run_complete_interactive().await?,
            1 => create_output_dir_interactive().await?,
            2 => write_cycle_interactive().await?,
            3 => print_manual(),
            4 => {
                println!("再见！");
                break;
            }
            _ => println!("无效选择"),
        }

        println!(); // 空行分隔
    }

    Ok(())
}
async fn create_output_dir_interactive() -> Result<()> {
    println!("📁 选择数据表文件");

    let path = if let Some(file) = file_dialog_select("选择数据表", &["xlsx"], true) {
        file
    } else {
        println!("❌ 未选择文件");
        return Ok(());
    };

    let cycle_manager = ChannelManager::load_manage_list(path).await?;
    println!("✅ 成功加载管理列表");

    println!("📁 选择数据导出目录生成位置");

    let path = if let Some(dir) = file_dialog_select("选择数据导出目录生成位置", &[], false)
    {
        dir
    } else {
        println!("❌ 未选择目录");
        return Ok(());
    };

    let _result = cycle_manager.crate_output_dir(path).await?;
    println!("✅ 成功创建输出目录");
    Ok(())
}

async fn write_cycle_interactive() -> Result<()> {
    println!("📁 选择数据表文件");

    let list_path = file_dialog_select("选择数据表", &["xlsx"], true).unwrap();
    let mut cycle_manager = ChannelManager::load_manage_list(list_path).await?;
    println!("✅ 成功加载管理列表");

    let data_dir = file_dialog_select("选择数据目录", &[], false).unwrap();
    let _result = cycle_manager.search_data_dir(data_dir).await?;
    println!("搜索数据目录完成");

    println!("💾 选择存储位置");

    let save_path =
        file_dialog_save("选择存储位置", &["xlsx"], "循环电芯跟踪表(已合并).xlsx").unwrap();
    let _result = cycle_manager.write_cycle(&save_path).await?;
    println!("✅ 成功写入循环数据到: {}", save_path.display());

    println!("📁 选择数据移动目标目录");

    let move_target = file_dialog_select("选择数据复制目标目录", &[], false).unwrap();
    let _result = cycle_manager.move_data(&move_target).await?;
    println!("✅ 成功移动数据到: {}", move_target.display());

    Ok(())
}
async fn run_complete_interactive() -> Result<()> {
    println!("🚀 开始执行完整流程");

    // 简化实现，实际中可能需要更复杂的路径收集逻辑
    let list_path = file_dialog_select("选择数据表", &["xlsx"], true).unwrap();
    let mut cycle_manager = ChannelManager::load_manage_list(list_path).await?;
    println!("✅ 步骤1/5: 加载管理列表完成");

    let output_dir = file_dialog_select("选择数据导出目录生成位置", &[], false).unwrap();
    let _result = cycle_manager.crate_output_dir(output_dir).await?;
    println!("✅ 步骤2/5: 创建输出目录完成");

    let data_dir = file_dialog_select("选择数据目录", &[], false).unwrap();
    let _result = cycle_manager.search_data_dir(data_dir).await?;
    println!("✅ 步骤3/5: 搜索数据目录完成");

    let save_path =
        file_dialog_save("选择存储位置", &["xlsx"], "循环电芯跟踪表(已合并).xlsx").unwrap();
    let _result = cycle_manager.write_cycle(&save_path).await?;
    println!("✅ 步骤4/5: 写入循环数据完成");

    let move_target = file_dialog_select("选择数据复制目标目录", &[], false).unwrap();
    let _result = cycle_manager.move_data(&move_target).await?;
    println!("✅ 步骤5/5: 复制数据完成");

    println!("🎉 所有步骤执行完成！");
    Ok(())
}

fn print_manual() {
    println!();
    println!("📖 电芯数据管理工具 - 帮助文档");
    println!("==============================");
    println!();
    println!("使用方法:");
    println!("  cycle_manager [OPTIONS] [COMMAND]");
    println!();
    println!("选项:");
    println!("  -i, --interactive  进入交互模式");
    println!();
    println!("命令:");
    println!("  load              加载管理列表");
    println!("  create-output     创建输出目录");
    println!("  search-data       搜索数据目录");
    println!("  write-cycle       写入循环数据");
    println!("  move-data         移动数据");
    println!("  run               执行完整流程");
    println!("  help              显示此帮助信息");
    println!();
    println!("示例:");
    println!("  cycle_manager -i                    # 进入交互模式");
    println!("  cycle_manager load --file data.xlsx # 直接加载文件");
    println!("  cycle_manager run --list-file data.xlsx --output-dir ./output ...");
    println!();
}

// 文件选择对话框辅助函数
fn file_dialog_select(title: &str, filters: &[&str], is_file: bool) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title(title.to_string())
        .set_directory("/");

    let dialog = filters
        .iter()
        .fold(dialog, |d, filter| d.add_filter("", &[filter]));

    if is_file {
        dialog.pick_file()
    } else {
        dialog.pick_folder()
    }
}

fn file_dialog_save(title: &str, filters: &[&str], default_name: &str) -> Option<PathBuf> {
    let dialog = FileDialog::new()
        .set_title(title.to_string())
        .set_directory("/")
        .set_file_name(default_name);

    let dialog = filters
        .iter()
        .fold(dialog, |d, filter| d.add_filter("", &[filter]));

    dialog.save_file()
}

struct ChannelManager {
    workbook: Workbook,
    channels: HashMap<String, Vec<ChannelInfo>>,
    data_dir: PathBuf,
    data_info: Vec<DataInfo>,
}

impl ChannelManager {
    async fn load_manage_list(file_path: PathBuf) -> Result<Self> {
        // println!("Loading Channel:{:?}", file_path);

        let workbook = Workbook::from_path(&file_path)
            .with_context(|| format!("文件读取失败 {:?}", file_path))?;

        let work_sheet = workbook.get_worksheet_by_name("循环中").with_context(|| {
            "Excel表格式错误，没有找到名为循环中的工作簿，请检查文件格式是否正确".to_string()
        })?;

        let mut channels = HashMap::new();

        //获取有效通道
        for row in 2..=work_sheet.max_row() {

            // println!("read for {}",row);
            if work_sheet.read_cell((row, 4))?.text.is_some() {
                if work_sheet.read_cell((row, 3))?.text == Some(String::from("长期")) {
                    let channel_info = ChannelInfo::new(work_sheet, row as u32)?;
                    // println!("{}", channel_info);
                    channels
                        .entry(channel_info.device_name.clone())
                        .or_insert_with(Vec::new)
                        .push(channel_info);
                }
            } else if work_sheet.read_cell((row, 1))?.text.is_none() {
                //exit for exception
                break;
            }
        }

        let channel_manager = ChannelManager {
            workbook,
            channels,
            data_dir: PathBuf::new(),
            data_info: Vec::new(),
        };

        Ok(channel_manager)
    }
    async fn crate_output_dir(&self, crate_data_dir: PathBuf) -> Result<()> {
        let device_list = self.get_device_list();

        for device_name in device_list {
            //crate device folder
            let device_dir = crate_data_dir.join(&device_name);
            fs::create_dir_all(&device_dir)?; //crate folder

            //crate output list
            let channels = self.channels.get(&device_name).unwrap();
            let output_channels = channels
                .iter()
                .map(|x| x.channel_name.clone())
                .collect::<Vec<String>>()
                .join("\n");
            let output_info = channels
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join("\n");

            let output = format!("导出通道：\n{}\n\n{}", output_channels, output_info);

            let file_name = format!("{}导出清单.txt", device_name);
            // let file_name = "导出清单.txt";

            let file_path = device_dir.join(file_name);

            fs::write(&file_path, output)?;
        }
        Ok(())
    }
    async fn search_data_dir(&mut self, data_dir: PathBuf) -> Result<()> {
        //开始搜索数据目录，获取基础信息
        let mut data_info = Vec::new();
        let device_list = self.get_device_list();

        // println!("Loading Data:{:?}", device_list);

        for device_name in device_list {
            let data_files = match DataInfo::get_data_info(&data_dir, &device_name).await {
                Ok(d) => d,
                Err(e) => {
                    println!("ERROR:{}", e);
                    continue;
                }
            };
            data_info.extend(data_files);
        }

        self.data_dir = data_dir;
        self.data_info = data_info;

        println!("数据目录读取完成！");

        Ok(())
    }
    async fn write_cycle(&mut self, save_path: &PathBuf) -> Result<()> {
        let work_sheet = self
            .workbook
            .get_worksheet_mut_by_name("循环中")
            .with_context(|| {
                "Excel表格式错误，没有找到名为循环中的工作簿，请检查文件格式是否正确".to_string()
            })?;

        for data_info in &self.data_info {
            let device_name = &data_info.device_name;
            let channel_name = &data_info.channel_name;
            let data_path = &data_info.data_path;

            println!("正在搜索... 设备号:{} 通道号:{}", device_name, channel_name);
            for row in 2..=work_sheet.max_row() {
                if work_sheet.read_cell((row, 1))?.text == Some(device_name.to_string()) {
                    if work_sheet.read_cell((row, 2))?.text == Some(channel_name.to_string()) {
                        //read data from files
                        let (cycle_num, cycle_capacity) = DataInfo::get_shenghong_data(&data_path)
                            .await
                            .unwrap_or((0, 0.0));
                        //write into it
                        work_sheet.write((row, 22), cycle_num)?;
                        work_sheet.write((row, 23), cycle_capacity)?;
                        // println!("write to sheet with {},{}", cycle_num, cycle_capacity);
                        continue;
                    }
                }
            }
        }

        self.workbook.save_as(save_path)?;

        println!("写入完成，文件已保存到：{}", save_path.display());
        Ok(())
    }
    async fn move_data(&self, new_data_path: &PathBuf) -> Result<()> {
        //match datainfo with channelinfo
        for data_info in &self.data_info {
            let device_name = &data_info.device_name;
            let channel_name = &data_info.channel_name;
            let data_path = &data_info.data_path;

            let channels = self.channels.get(device_name).unwrap();
            //match channel name
            for channel in channels {
                if channel.channel_name.eq(channel_name) {
                    let new_path = new_data_path.join(channel.crate_path());
                    let _result = move_file_with_checks(data_path, &new_path);

                    continue;
                }
            }
        }
        Ok(())
    }
    fn get_device_list(&self) -> Vec<String> {
        let device_list: Vec<_> = self.channels.keys().cloned().collect(); //提取设备编号
        device_list
    }
}
struct DataInfo {
    data_path: PathBuf,
    device_name: String,
    channel_name: String,
}

impl DataInfo {
    async fn get_data_info(dir_path: &PathBuf, device_name: &str) -> Result<Vec<DataInfo>> {
        let full_path = dir_path.join(device_name);
        let path = Path::new(&full_path);
        let mut data_info = Vec::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            // 检查是否是文件且有xlsx后缀
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "xlsx" {
                        let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                        let channel_name = match match_channel_name(&file_name) {
                            Some(c) => c,
                            None => {
                                println!("通道名获取失败，尝试进入文件查询");
                                DataInfo::get_shenghong_channel(&path)
                                    .await
                                    .unwrap_or(String::new())
                            }
                        };

                        data_info.push(DataInfo {
                            data_path: path,
                            device_name: device_name.to_string(),
                            channel_name,
                        })
                    }
                }
            }
        }
        Ok(data_info)
    }

    async fn get_shenghong_data(file_path: &PathBuf) -> Result<(u32, f32)> {
        let read_book = Workbook::from_path(file_path)
            .with_context(|| format!("文件读取失败 {}", file_path.to_str().unwrap()))?;

        let cycle_sheet = read_book
            .get_worksheet_by_name("循环数据表")
            .with_context(|| {
                "Excel表格式错误，没有找到名为循环数据表的工作簿，请检查文件是否为盛洪BTS生成"
                    .to_string()
            })?;

        let max_cycle = cycle_sheet.max_row() - 1; //for last for no use
        let cycle_num = cycle_sheet.read_cell((max_cycle, 1))?;
        let cycle_capacity = cycle_sheet.read_cell((max_cycle, 3))?;

        //parse cycle data
        let cycle_num: u32 = cycle_num.text.unwrap().parse()?;
        let cycle_capacity: f32 = cycle_capacity.text.unwrap().parse()?;
        println!("循环计数:{},放电容量:{}", cycle_num, cycle_capacity);

        Ok((cycle_num, cycle_capacity))
    }

    async fn get_shenghong_channel(file_path: &PathBuf) -> Result<String> {
        let read_book = Workbook::from_path(file_path)
            .with_context(|| format!("文件读取失败 {}", file_path.to_str().unwrap()))?;

        let data_sheet = read_book.get_worksheet_by_name("数据表").with_context(|| {
            "Excel表格式错误，没有找到名为数据表的工作簿，请检查文件是否为盛洪BTS生成".to_string()
        })?;

        let cell_num = data_sheet.read_cell((3, 2))?;
        let channel_num = data_sheet.read_cell((4, 2))?;

        let channel_name = format!("{}-{}", cell_num.text.unwrap(), channel_num.text.unwrap());

        println!("查询成功，通道号：{}", channel_name);

        Ok(channel_name)
    }
}
#[derive(Debug)]
struct ChannelInfo {
    device_name: String,
    channel_name: String,
    sample_num: String,
    data_name: String,
    test_code: String,
    sample_fmt: SampleFmt,
    test_temperature: TestTemperature,
}

impl ChannelInfo {
    fn new(work_sheet: &WorkSheet, row_num: u32) -> Result<Self> {
        let device_name = work_sheet.read_cell((row_num, 1))?.text.unwrap();

        let channel_name = work_sheet.read_cell((row_num, 2))?.text.unwrap();

        let test_code = work_sheet.read_cell((row_num, 4))?.text.unwrap();

        let sample_num = work_sheet
            .read_cell((row_num, 9))?
            .text
            .unwrap_or(String::from("无编号"));

        let data_name = work_sheet.read_cell((row_num, 10))?.text.unwrap();

        let sample_fmt = SampleFmt::from(
            &work_sheet
                .read_cell((row_num, 6))?
                .text
                .unwrap_or(String::new()),
        );

        let test_temperature = TestTemperature::from(
            &work_sheet
                .read_cell((row_num, 14))?
                .text
                .unwrap_or(String::new()),
        );

        let channel_info = ChannelInfo {
            device_name,
            channel_name,
            sample_num,
            data_name,
            test_code,
            sample_fmt,
            test_temperature,
        };

        Ok(channel_info)
    }
    fn crate_path(&self) -> String {
        let file_name = format!(
            "{}/{}/{}/{}.xlsx",
            self.sample_fmt, self.test_code, self.test_temperature, self.data_name
        );
        file_name
    }
}

impl fmt::Display for TestTemperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestTemperature::T25 => write!(f, "25℃"),
            TestTemperature::T35 => write!(f, "35℃"),
            TestTemperature::T45 => write!(f, "45℃"),
            TestTemperature::T55 => write!(f, "55℃"),
            TestTemperature::None => write!(f, "未设置"),
        }
    }
}

impl fmt::Display for SampleFmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SampleFmt::H50 => write!(f, "铝壳50"),
            SampleFmt::H65 => write!(f, "铝壳65"),
            SampleFmt::H100 => write!(f, "铝壳100"),
            SampleFmt::H150 => write!(f, "铝壳150"),
            SampleFmt::H180 => write!(f, "铝壳180"),
            SampleFmt::H280 => write!(f, "铝壳280"),
            SampleFmt::S50 => write!(f, "软包50"),
            SampleFmt::Na165 => write!(f, "钠电165"),
            SampleFmt::Na170 => write!(f, "钠电170"),
            SampleFmt::None => write!(f, "未设置"),
        }
    }
}

// 为 ChannelInfo 实现 Display trait
impl fmt::Display for ChannelInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "通道信息")?;
        writeln!(f, "┌{}", "─".repeat(50))?;
        writeln!(f, "│ 设备名称: {}", self.device_name)?;
        writeln!(f, "│ 通道名称: {}", self.channel_name)?;
        writeln!(f, "│ 样品编号: {}", self.sample_num)?;
        writeln!(f, "│ 数据名称: {}", self.data_name)?;
        writeln!(f, "│ 测试代码: {}", self.test_code)?;
        writeln!(f, "│ 样品规格: {}", self.sample_fmt)?;
        writeln!(f, "│ 测试温度: {}", self.test_temperature)?;
        write!(f, "└{}", "─".repeat(50))
    }
}

#[derive(Debug)]
enum TestTemperature {
    T25,
    T35,
    T45,
    T55,
    None,
}

impl TestTemperature {
    fn from(text: &str) -> Self {
        let pattern = r"(25℃|35℃|45℃|55℃)";
        let re = Regex::new(pattern).unwrap();

        if let Some(mat) = re.find(text) {
            match mat.as_str() {
                "25℃" => TestTemperature::T25,
                "35℃" => TestTemperature::T35,
                "45℃" => TestTemperature::T45,
                "55℃" => TestTemperature::T55,
                _ => TestTemperature::None,
            }
        } else {
            TestTemperature::None
        }
    }
}

#[derive(Debug)]
enum SampleFmt {
    H50,
    H65,
    H100,
    H150,
    H180,
    H280,
    S50,
    Na165,
    Na170,
    None,
}

impl SampleFmt {
    fn from(text: &str) -> Self {
        match text {
            "铝壳50" => SampleFmt::H50,
            "65" => SampleFmt::H65,
            "100" => SampleFmt::H100,
            "150" => SampleFmt::H150,
            "180" => SampleFmt::H180,
            "280" => SampleFmt::H280,
            "软包50" => SampleFmt::S50,
            "钠电165" => SampleFmt::Na165,
            "钠电170" => SampleFmt::Na170,
            _ => SampleFmt::None,
        }
    }
}

fn match_channel_name(input: &str) -> Option<String> {
    // 匹配 UN 后面的数字和 CN 后面的数字，顺序无关
    let re = Regex::new(r"UN(\d+).*?CN(\d+)|CN(\d+).*?UN(\d+)").unwrap();

    let caps = re.captures(input)?;

    let (un_num, cn_num) = if caps.get(1).is_some() {
        // UN在前，CN在后的情况
        (caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
    } else {
        // CN在前，UN在后的情况
        (caps.get(4).unwrap().as_str(), caps.get(3).unwrap().as_str())
    };

    // 去除前导零
    let un_clean = un_num.trim_start_matches('0');
    let cn_clean = cn_num.trim_start_matches('0');

    // 如果去除零后为空字符串，则使用 "0"
    let un_final = if un_clean.is_empty() { "0" } else { un_clean };
    let cn_final = if cn_clean.is_empty() { "0" } else { cn_clean };

    Some(format!("{}-{}", un_final, cn_final))
}

fn move_file_with_checks(source: &PathBuf, destination: &PathBuf) -> Result<()> {
    // let source = "source_file.txt";
    // let destination = "target_dir/moved_file.txt";

    // 检查源文件是否存在
    if !Path::new(source).exists() {
        return Err(Error::msg("源文件不存在"));
    }

    // 检查目标目录是否存在，不存在则创建
    if let Some(parent) = Path::new(destination).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            println!("创建了目标目录: {:?}", parent);
        }
    }

    // 移动文件
    fs::copy(source, destination)?;
    println!("成功将 '{:?}' 复制到 '{:?}'", source, destination);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_check_channel_by_file_name() -> Result<()> {
        let test_cases = vec![
            "UN152_CN01_456446545321",
            "CN01_UN152_456446545321", // 顺序互换
            "UN001_CN005_123456",
            "CN005_UN001_123456", // 顺序互换
            "UN0_CN00_123456",
            "CN10_UN20_123456",
        ];

        for test_case in test_cases {
            if let Some(result) = match_channel_name(test_case) {
                println!("输入: {} -> 输出: {}", test_case, result);
            } else {
                println!("输入: {} -> 无法匹配", test_case);
            }
        }

        Ok(())
    }

    #[test]
    fn test_check_device_integration() -> Result<()> {
        // 这是一个集成测试的示例框架
        // 在实际测试中，你需要：
        // 1. 设置测试用的Excel文件
        // 2. 创建CycleManageList实例
        // 3. 调用check_device方法

        // 伪代码：
        // let test_dir = create_test_directory();
        // let test_file = create_test_excel_file();
        // let cycle_list = CycleManageList::new(&test_dir)?;
        // let result = cycle_list.check_device()?;

        println!("Integration test framework ready");
        Ok(())
    }
}
