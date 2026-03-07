use anyhow::{bail, Context, Error, Result};
use calamine::{open_workbook, Data, Reader, Xlsx};
use chrono::Local;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use regex::Regex;
use rfd::FileDialog;
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fmt, fs};
use std::str::FromStr;
use tokio::sync::RwLock;

use cycle_helper::input::form::DataForm;

const CYCLE_SHEET: &str = "循环中";

#[tokio::main]
async fn main() -> Result<()> {
    let _result = interactive_mode().await;

    Ok(())
}

async fn interactive_mode() -> Result<()> {
    println!("🔋 电芯数据管理工具 Alpha-0.2.3 NO.264323");
    println!("===================");

    let main_data = Arc::from(RwLock::from(Vec::new()));

    loop {
        let options = vec![
            "1. 导入数据表",
            "2. 创建数据采集目录",
            "3. 读取并更新循环数据",
            "4. 输出数据表",
            "5. 整理数据文件到指定目录",
            "6. 显示循环数据",
            "7. 退出",
            "8. 读取并更新循环数据V2(Beta)",
            "",
            "0.2.3更新日志",
            "-----------------------------------",
            "1.更新了星云的数据文件匹配",
            "2.完成的星云导出的Beta测试，目前已经可以读取相关数据文件",
            "",
            "0.2.2更新日志",
            "-----------------------------------",
            "1.全新重构的更新函数进入Beta测试",
            "2.增加了圈数手动处理选项，可根据需要自行选择",
            "如遇其他问题，请联系hyperflex@qq.com"
        ];

        let selection = Select::new()
            .with_prompt("请选择操作")
            .items(&options)
            .default(0)
            .interact()?;

        match selection {
            0 => import_data_from_list(main_data.clone()).await?,
            1 => create_import_dir(main_data.clone()).await?,
            2 => read_update_cycle(main_data.clone()).await?,
            3 => write_to_xlsx(main_data.clone()).await?,
            4 => form_manage(main_data.clone()).await?,
            5 => print_data(main_data.clone()).await,
            6 => {
                println!("再见！NO.264323");
                break;
            }
            7 => read_update_cycle_v2(main_data.clone()).await?,
            _ => println!("无效选择"),
        }

        println!(); // 空行分隔
    }

    Ok(())
}

async fn form_manage(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    let channel_list = data.read().await;

    let mut manage_list = Vec::new();

    for channel in channel_list.iter() {
        let form_path = channel.form_path.clone();
        if form_path.is_some() {
            //organize file name
            let new_filename = format!(
                "{} {}-{}.xlsx",
                channel.sample_id, channel.device, channel.channel
            );
            let new_path = format!(
                "{}//{} {}//{}//{}",
                channel.sample_model,
                channel.project_code,
                channel.sample_person.clone().unwrap_or(String::new()),
                channel.test_temperature,
                new_filename
            );
            manage_list.push((form_path.unwrap(), new_path));
        }
    }

    if manage_list.len() == 0 {
        println!("❌ 列表为空,停止整理,请确认已完成数据采集");
        return Ok(());
    }

    println!("📁 数据准备完成,选择数据整理目标位置");
    let dialog = FileDialog::new()
        .set_title("选择数据整理到")
        .set_directory("/");

    let current_date = Local::now().format("%Y%m%d").to_string();
    let default_folder_name = format!("{}循环数据整理", current_date);

    let dialog = dialog.pick_folder();

    let manage_dir = if let Some(folder) = dialog {
        folder.join(&default_folder_name)
    } else {
        println!("❌ 未选择目录");
        return Ok(());
    };

    for (form_path, new_path) in manage_list {
        let new_path = manage_dir.join(&new_path);
        // println!("{:?},{:?}",form_path, new_path);
        let _result = copy_file_with_checks(&form_path, &new_path);
        // println!("{:?}",_result);
    }

    Ok(())
}

async fn write_to_xlsx(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    let channel_list = data.read().await;

    println!("📁 选择数据表输出目录");
    let current_date = Local::now().format("%Y%m%d").to_string();
    let default_list_name = format!("{}循环数据表", current_date);
    let dialog = FileDialog::new()
        .set_title("选择数据表文件")
        .set_directory("/")
        .add_filter("Excel表格", &["xlsx"])
        .set_file_name(default_list_name)
        .save_file();

    let path = if let Some(file) = dialog {
        file
    } else {
        println!("❌ 未选择文件");
        return Ok(());
    };

    let _result = write_channel_info_to_xlsx(&channel_list, &path);

    Ok(())
}

async fn read_update_cycle(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    let data_read = data.read().await;

    if data_read.len() == 0 {
        println!("❌ 列表为空,停止导出");
        return Ok(());
    }

    let mut import_list = HashMap::new();

    for channel_info in data_read.iter() {
        let channel_active = channel_info.get_active();
        if channel_active.is_some() {
            let device = channel_info.get_device();
            import_list
                .entry(device.to_string())
                .or_insert_with(Vec::new)
                .push(channel_active.unwrap());
        } else {
            continue;
        }
    }

    // println!("{:?}",import_list);

    drop(data_read);

    println!("📁 数据准备完成,选择数据采集目录");
    let dialog = FileDialog::new()
        .set_title("选择数据采集目录")
        .set_directory("/");

    let dialog = dialog.pick_folder();

    let data_dir = if let Some(folder) = dialog {
        folder
    } else {
        println!("❌ 未选择目录");
        return Ok(());
    };

    let device_list: Vec<String> = import_list.keys().cloned().collect();

    let mut data_dir_list = HashMap::new();

    for device in device_list.iter() {
        let data_path = data_dir.join(&device);

        if let Ok(dir_read) = fs::read_dir(data_path) {
            let xlsx_forms: Vec<DataForm> = dir_read
                .flatten() // 过滤掉错误的entry
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .filter(|path| path.extension().map_or(false, |ext| ext == "xlsx"))
                .filter_map(|path| DataForm::init(&path).ok())
                .collect();

            if !xlsx_forms.is_empty() {
                data_dir_list
                    .entry(device.to_string())
                    .or_insert_with(Vec::new)
                    .extend(xlsx_forms);
            }
        }
    }

    // println!("{:?}",data_dir_list);

    println!("📁 数据文件已完成索引,开始匹配数据");

    let mut tasks = Vec::new();

    // 预收集所有需要处理的数据，避免在异步闭包中引用外部数据
    for device in device_list.iter() {
        // 获取设备对应的数据
        let Some(device_data) = data_dir_list.get(device) else {
            continue;
        };

        // 获取设备对应的通道列表
        let Some(device_channels) = import_list.get(device) else {
            continue;
        };

        // 为每个通道匹配对应的数据
        for channel_active in device_channels {
            let channel_to_match = channel_active.get_channel();
            let mut matched = false;

            // 首先用 channel 匹配
            for data in device_data.iter() {
                if data.channel_eq(&channel_to_match) {
                    // 克隆需要移动到异步任务中的数据
                    let device_clone = channel_active.device.clone();
                    let channel_clone = channel_active.get_channel();
                    let first_run = channel_active.first_run;
                    let data_clone = data.clone(); // 假设 Data 实现了 Clone

                    // 创建异步任务
                    let task = tokio::spawn(async move {
                        // println!("{}-{}匹配成功,正在搜索...",device_clone,channel_clone);
                        let (cycle_num, capacity) = data_clone.get_cycle_last()?;
                        let form_path = data_clone.get_form_path();

                        if first_run {
                            let (_, first_capacity) = data_clone.get_cycle_first()?;
                            // println!("{}-{}搜索完成,循环次数{},当前容量{}",device_clone,channel_clone,cycle_num,capacity);
                            return Ok::<_, Error>((
                                device_clone,
                                channel_clone,
                                cycle_num,
                                capacity,
                                form_path,
                                Some(first_capacity),
                            ));
                        }

                        Ok((
                            device_clone,
                            channel_clone,
                            cycle_num,
                            capacity,
                            form_path,
                            None,
                        ))
                    });

                    tasks.push(task);
                    matched = true;
                    break; // 找到一个匹配就跳出内层循环，避免重复处理
                }
            }

            // 如果 channel 没有匹配成功，尝试使用 prv 匹配
            if !matched {
                let channel_prv_to_match = channel_active.get_channel_prv();

                for data in device_data.iter() {
                    if data.channel_eq(&channel_prv_to_match) {
                        // 克隆需要移动到异步任务中的数据
                        let device_clone = channel_active.device.clone();
                        let channel_clone = channel_active.get_channel_prv();
                        let first_run = channel_active.first_run;
                        let data_clone = data.clone();

                        // 创建异步任务
                        let task = tokio::spawn(async move {
                            // println!("{}-{}匹配成功,正在搜索...",device_clone,channel_clone);
                            let (cycle_num, capacity) = data_clone.get_cycle_last()?;
                            let form_path = data_clone.get_form_path();

                            if first_run {
                                let (_, first_capacity) = data_clone.get_cycle_first()?;
                                // println!("{}-{}搜索完成,循环次数{},当前容量{}",device_clone,channel_clone,cycle_num,capacity);
                                return Ok::<_, Error>((
                                    device_clone,
                                    channel_clone,
                                    cycle_num,
                                    capacity,
                                    form_path,
                                    Some(first_capacity),
                                ));
                            }

                            Ok((
                                device_clone,
                                channel_clone,
                                cycle_num,
                                capacity,
                                form_path,
                                None,
                            ))
                        });

                        tasks.push(task);
                        break; // 找到一个匹配就跳出内层循环，避免重复处理
                    }
                }
            }
        }
    }

    //catch result
    // 收集所有任务结果
    let mut results = Vec::new();
    for task in tasks {
        let task = task.await;
        match task {
            Ok(Ok(result)) => {
                // 任务成功且内部操作成功
                println!(
                    "{}-{}搜索完成,循环次数{},当前容量{}",
                    result.0, result.1, result.2, result.3
                );
                results.push(result);
            }
            Ok(Err(e)) => {
                // 任务成功但内部操作失败
                eprintln!("Task error: {}", e);
            }
            Err(e) => {
                // 任务本身失败（如 panic）
                eprintln!("Join error: {}", e);
            }
        }
        // println!("ping");
    }

    println!("读取完成，开始更新...");

    //update data into channel info
    let mut data_write = data.write().await;
    for (device, channel, cycle_num, capacity, form_path, first_run) in results {
        let index = find_channel_info_index(&data_write, &device, &channel);
        match index {
            Some(index) => {
                data_write[index].update_capacity(capacity);
                data_write[index].update_cycles(cycle_num as usize);
                data_write[index].update_form_path(form_path);
                if first_run.is_some() {
                    data_write[index].update_initial_capacity(first_run.unwrap());
                }
            }
            None => {
                continue;
            }
        }
    }

    println!("✅ 数据采集与更新完成");

    Ok(())
}

async fn read_update_cycle_v2(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    // 1. 读取并处理数据
    let import_list = {
        let data_read = data.read().await;

        if data_read.is_empty() {
            println!("❌ 列表为空,停止导出");
            return Ok(());
        }

        // 使用 HashMap 分组收集活跃通道
        let mut import_list = HashMap::<String, Vec<ChannelActive>>::new();

        for channel_info in data_read.iter() {
            if let Some(channel_active) = channel_info.get_active() {
                let device = channel_info.get_device();
                import_list
                    .entry(device.to_string())
                    .or_default()
                    .push(channel_active);
            }
        }

        import_list
    };

    if import_list.is_empty() {
        println!("❌ 没有活跃通道需要处理");
        return Ok(());
    }

    // 2. 选择数据目录
    println!("📁 数据准备完成,选择数据采集目录");

    let data_dir = FileDialog::new()
        .set_title("选择数据采集目录")
        .set_directory("/")
        .pick_folder()
        .context("未选择目录")?;

    // 3. 读取和索引数据文件
    let data_dir_list = build_data_dir_list(&import_list, &data_dir).await?;

    if data_dir_list.is_empty() {
        println!("❌ 未找到任何数据文件");
        return Ok(());
    }

    println!("📁 数据文件已完成索引,开始匹配数据");

    // 4. 创建并执行匹配任务
    let tasks = create_matching_tasks(&import_list, &data_dir_list);

    // 5. 收集任务结果
    let results = collect_task_results(tasks).await;

    if results.is_empty() {
        println!("⚠️  未找到任何匹配的数据");
        return Ok(());
    }

    println!("读取完成，开始更新...");

    // 6. 更新数据
    update_channel_data(data, &results).await;

    println!("✅ 数据采集与更新完成");
    Ok(())
}

/// 收集任务结果
async fn collect_task_results(
    tasks: Vec<tokio::task::JoinHandle<Result<MatchResult>>>,
) -> Vec<MatchResult> {
    let mut results = Vec::new();

    for task in tasks {
        match task.await {
            Ok(Ok(result)) => {
                println!(
                    "{}-{}搜索完成,循环次数{},当前容量{}",
                    result.0, result.1, result.2, result.3
                );
                results.push(result);
            }
            Ok(Err(e)) => {
                eprintln!("任务处理错误: {}", e);
            }
            Err(e) => {
                eprintln!("任务执行失败: {}", e);
            }
        }
    }

    results
}

/// 创建匹配任务的辅助函数
fn create_matching_tasks(
    import_list: &HashMap<String, Vec<ChannelActive>>,
    data_dir_list: &HashMap<String, Vec<DataForm>>,
) -> Vec<tokio::task::JoinHandle<Result<MatchResult>>> {
    let mut tasks = Vec::new();

    for (device, device_channels) in import_list {
        let Some(device_data) = data_dir_list.get(device) else {
            continue;
        };

        // 为设备数据建立索引以提高匹配效率
        let channel_index = build_channel_index(device_data);

        for channel_active in device_channels {
            // 尝试匹配通道
            if let Some(task) = try_match_channel(device, channel_active, &channel_index) {
                tasks.push(task);
                continue;
            }

            // 尝试匹配通道PRV
            if let Some(task) = try_match_channel_prv(device, channel_active, &channel_index) {
                tasks.push(task);
            }
        }
    }

    tasks
}

/// 构建通道索引
fn build_channel_index(data_forms: &[DataForm]) -> HashMap<String, &DataForm> {
    let mut index = HashMap::new();
    for data in data_forms {
        // 这里需要根据 DataForm 的实际字段调整
        // 假设 DataForm 有 channel_id 字段
        index.insert(data.get_channel(), data);
    }
    index
}

/// 尝试匹配通道
fn try_match_channel(
    device: &str,
    channel_active: &ChannelActive,
    channel_index: &HashMap<String, &DataForm>,
) -> Option<tokio::task::JoinHandle<Result<MatchResult>>> {
    let channel_to_match = channel_active.get_channel();

    // 使用索引快速查找
    // 这里需要根据实际数据结构调整
    if let Some(data) = channel_index.get(&channel_to_match) {
        return Some(create_task(device.to_string(), channel_to_match, channel_active.first_run, data));
    }

    None // 临时返回，需要根据实际情况实现
}

/// 尝试匹配通道PRV
fn try_match_channel_prv(
    device: &str,
    channel_active: &ChannelActive,
    channel_index: &HashMap<String, &DataForm>,
) -> Option<tokio::task::JoinHandle<Result<MatchResult>>> {
    let channel_prv_to_match = channel_active.get_channel_prv();

    // 使用索引快速查找
    // 这里需要根据实际数据结构调整
    if let Some(data) = channel_index.get(&channel_prv_to_match) {
        return Some(create_task(device.to_string(), channel_prv_to_match, channel_active.first_run, data));
    }

    None // 临时返回，需要根据实际情况实现
}


/// 创建异步任务
fn create_task(
    device: String,
    channel: String,
    first_run: bool,
    data: &DataForm,
) -> tokio::task::JoinHandle<Result<MatchResult>> {
    let data_clone = data.clone(); // 假设 DataForm 实现了 Clone

    tokio::spawn(async move {
        let (cycle_num, capacity) = data_clone.get_cycle_last()?;
        let form_path = data_clone.get_form_path();

        let first_capacity = if first_run {
            Some(data_clone.get_cycle_first()?.1)
        } else {
            None
        };

        Ok((device, channel, cycle_num, capacity, form_path, first_capacity))
    })
}

/// 构建数据目录列表
async fn build_data_dir_list(
    import_list: &HashMap<String, Vec<ChannelActive>>,
    data_dir: &PathBuf,
) -> Result<HashMap<String, Vec<DataForm>>> {
    use tokio::fs;

    let mut data_dir_list = HashMap::new();

    for device in import_list.keys() {
        let data_path = data_dir.join(device);

        if let Ok(mut dir_read) = fs::read_dir(&data_path).await {
            let mut xlsx_forms = Vec::new();

            while let Some(entry) = dir_read.next_entry().await? {
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                if path.extension().map_or(false, |ext| ext == "xlsx") {
                    if let Ok(data_form) = DataForm::init(&path) {
                        xlsx_forms.push(data_form);
                    }
                }
            }

            if !xlsx_forms.is_empty() {
                data_dir_list.insert(device.clone(), xlsx_forms);
            }
        }
    }

    Ok(data_dir_list)
}

/// 通道匹配结果类型
type MatchResult = (String, String, u32, f32, PathBuf, Option<f32>);

/// 更新通道数据
async fn update_channel_data(
    data: Arc<RwLock<Vec<ChannelInfo>>>,
    results: &[MatchResult],
) {
    let mut data_write = data.write().await;

    for (device, channel, cycle_num, capacity, form_path, first_capacity) in results {
        if let Some(index) = find_channel_info_index(&data_write, device, channel) {
            data_write[index].update_capacity(*capacity);
            data_write[index].update_cycles(*cycle_num as usize);
            data_write[index].update_form_path(form_path.clone());

            if let Some(first_cap) = first_capacity {
                data_write[index].update_initial_capacity(*first_cap);
            }
        }
    }
}

fn copy_file_with_checks(source: &PathBuf, destination: &PathBuf) -> Result<()> {
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

fn find_channel_info_index<'a>(
    infos: &'a Vec<ChannelInfo>,
    device: &str,
    channel: &str,
) -> Option<usize> {
    infos.iter().enumerate().find_map(|(idx, info)| {
        if info.device == device
            && (info.channel == channel || info.channel_prv.as_deref() == Some(channel))
        {
            Some(idx)
        } else {
            None
        }
    })
}

async fn create_import_dir(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    let data_read = data.read().await;

    if data_read.len() == 0 {
        println!("❌ 列表为空,停止导出");
        return Ok(());
    }

    let mut import_list = HashMap::new();

    for channel_info in data_read.iter() {
        if channel_info.test_status.is_long() {
            let device = &channel_info.device;
            let channel = &channel_info.channel;
            let channel_prv = &channel_info
                .channel_prv
                .clone()
                .unwrap_or(String::from("N/A"));
            let project_code = &channel_info.project_code;

            if !project_code.is_empty() {
                let info = format!(
                    "通道号：{} 项目编号：{} 曾用通道号：{}",
                    channel, project_code, channel_prv
                );

                import_list
                    .entry(device.to_string())
                    .or_insert_with(Vec::new)
                    .push(info);
            }
        }
    }

    println!("📁 数据准备完成,选择目录生成位置");
    let current_date = Local::now().format("%Y%m%d").to_string();
    let default_folder_name = format!("循环数据采集{}", current_date);

    let dialog = FileDialog::new()
        .set_title("选择目录生成位置")
        .set_directory("/");

    let dialog = dialog.pick_folder();

    let crate_data_dir = if let Some(folder) = dialog {
        folder.join(&default_folder_name)
    } else {
        println!("❌ 未选择目录");
        return Ok(());
    };

    let device_list: Vec<String> = import_list.keys().cloned().collect();

    for device_name in device_list {
        //crate device folder
        let device_dir = crate_data_dir.join(&device_name);
        fs::create_dir_all(&device_dir)?; //crate folder

        //crate output list
        let channels = import_list.get(&device_name).unwrap();
        let output_channels = channels
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<String>>()
            .join("\n");

        let output = format!("导出通道：\n{}", output_channels);

        let file_name = format!("{}导出清单.txt", device_name);

        let file_path = device_dir.join(file_name);

        fs::write(&file_path, output)?;
    }

    println!("✅ 数据采集目录生成完成");
    Ok(())
}

async fn print_data(data: Arc<RwLock<Vec<ChannelInfo>>>) {
    let data_read = data.read().await;

    print_channel_list_table(&data_read);
}

async fn import_data_from_list(data: Arc<RwLock<Vec<ChannelInfo>>>) -> Result<()> {
    println!("📁 选择数据表文件");
    let dialog = FileDialog::new()
        .set_title("选择数据表文件")
        .set_directory("/")
        .add_filter("Excel表格", &["xlsx"])
        .pick_file();

    let path = if let Some(file) = dialog {
        file
    } else {
        println!("❌ 未选择文件");
        return Ok(());
    };

    let new = xlsx_reader(&path);

    let new_data: Vec<ChannelInfo> = match new {
        Ok(new_data) => new_data,
        Err(e) => {
            println!("❌ 文件读取错误:{:?}", e);
            return Ok(());
        }
    };

    let mut data_write = data.write().await;

    data_write.clear();
    data_write.extend(new_data);

    println!("✅ 成功加载管理列表");

    Ok(())
}

#[derive(Debug)]
struct ChannelActive {
    device: String,
    channel: String,
    channel_prv: Option<String>,
    first_run: bool,
}

impl ChannelActive {
    pub(crate) fn get_channel_prv(&self) -> String {
        self.channel_prv.clone().unwrap_or(String::from("N/A"))
    }
}

impl ChannelActive {
    fn get_channel(&self) -> String {
        self.channel.clone()
    }
}

#[derive(Debug)]
struct ColumnIndices {
    device: usize,
    channel: usize,
    project_code: usize,
    sample_person: Option<usize>, // 可选
    sample_model: usize,
    sample_id: usize,
    test_status: usize,
    test_temperature: usize,
    initial_capacity: usize,
    current_capacity: usize,
    previous_cycles: usize,
    current_cycles: usize,
    channel_prv: Option<usize>, // 可选
}

impl ColumnIndices {
    fn from_header_row(row: &[Data]) -> Result<Self> {
        let mut indices = Self {
            device: usize::MAX,
            channel: usize::MAX,
            project_code: usize::MAX,
            sample_person: None,
            sample_model: usize::MAX,
            sample_id: usize::MAX,
            test_status: usize::MAX,
            test_temperature: usize::MAX,
            initial_capacity: usize::MAX,
            current_capacity: usize::MAX,
            previous_cycles: usize::MAX,
            current_cycles: usize::MAX,
            channel_prv: None,
        };

        for (index, cell) in row.iter().enumerate() {
            let text = cell.to_string();
            match text.trim() {
                "上位机" => indices.device = index,
                "测试通道" => indices.channel = index,
                "申请单号" => indices.project_code = index,
                "送样人" => indices.sample_person = Some(index),
                "电芯型号" => indices.sample_model = index,
                "电芯编号" => indices.sample_id = index,
                "测试状态" => indices.test_status = index,
                "测试温度" | "测试方法" => indices.test_temperature = index,
                "初始容量" => indices.initial_capacity = index,
                "当前容量" => indices.current_capacity = index,
                "接续前圈数" => indices.previous_cycles = index,
                "当前圈数" => indices.current_cycles = index,
                "先前测试通道" => indices.channel_prv = Some(index),
                _ => {}
            }
        }

        // 检查必填列是否存在
        let required_fields = [
            ("上位机", indices.device),
            ("测试通道", indices.channel),
            ("申请单号", indices.project_code),
            ("电芯型号", indices.sample_model),
            ("电芯编号", indices.sample_id),
            ("测试状态", indices.test_status),
            ("测试温度/测试方法", indices.test_temperature),
            ("初始容量", indices.initial_capacity),
            ("当前容量", indices.current_capacity),
            ("接续前圈数", indices.previous_cycles),
            ("当前圈数", indices.current_cycles),
        ];

        for (field_name, index) in required_fields {
            if index == usize::MAX {
                bail!("表格格式非法：没有{}信息", field_name);
            }
        }

        Ok(indices)
    }
}
// 辅助函数
fn cell_to_string(row: &[Data], index: usize) -> String {
    row.get(index)
        .map(|data| data.to_string())
        .unwrap_or_default()
}

fn parse_cell_f32(row: &[Data], index: usize) -> Option<f32> {
    row.get(index)
        .and_then(|data| data.to_string().parse::<f32>().ok())
}

fn parse_cell_usize(row: &[Data], index: usize) -> Option<usize> {
    row.get(index)
        .and_then(|data| data.to_string().parse::<usize>().ok())
}

// 写入 ChannelInfo 到 Excel 文件
pub fn write_channel_info_to_xlsx(
    channels: &[ChannelInfo],
    output_path: &Path,
) -> Result<(), XlsxError> {
    if channels.is_empty() {
        return Err(XlsxError::ParameterError("没有通道数据可写入".to_string()));
    }

    // 创建新的 Excel 文件
    let mut workbook = Workbook::new();

    // 创建工作表
    let worksheet = workbook.add_worksheet();

    // 设置列宽
    worksheet.set_column_width(0, 10)?; // 上位机
    worksheet.set_column_width(1, 10)?; // 测试通道
    worksheet.set_column_width(2, 15)?; // 申请单号
    worksheet.set_column_width(3, 10)?; // 送样人
    worksheet.set_column_width(4, 10)?; // 电芯型号
    worksheet.set_column_width(5, 12)?; // 电芯编号
    worksheet.set_column_width(6, 10)?; // 测试状态
    worksheet.set_column_width(7, 12)?; // 测试温度
    worksheet.set_column_width(8, 12)?; // 初始容量
    worksheet.set_column_width(9, 12)?; // 当前容量
    worksheet.set_column_width(10, 12)?; // 接续前圈数
    worksheet.set_column_width(11, 12)?; // 当前圈数
    worksheet.set_column_width(12, 15)?; // 先前测试通道

    // 创建表头格式
    let header_format = Format::new()
        .set_bold()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    // 创建数值格式
    let number_format = Format::new().set_num_format("0.00");
    let integer_format = Format::new().set_num_format("0");

    // 写入表头
    let headers = [
        "上位机",
        "测试通道",
        "申请单号",
        "送样人",
        "电芯型号",
        "电芯编号",
        "测试状态",
        "测试温度",
        "初始容量",
        "当前容量",
        "接续前圈数",
        "当前圈数",
        "先前测试通道",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *header, &header_format)?;
    }

    // 写入数据行
    for (row_idx, channel) in channels.iter().enumerate() {
        let row = (row_idx + 1) as u32; // 从第2行开始（表头在第1行）

        // 上位机
        worksheet.write(row, 0, &channel.device)?;

        // 测试通道
        worksheet.write(row, 1, &channel.channel)?;

        // 申请单号
        worksheet.write(row, 2, &channel.project_code)?;

        // 送样人（可选）
        if let Some(person) = &channel.sample_person {
            worksheet.write(row, 3, person)?;
        }

        // 电芯型号
        worksheet.write(row, 4, channel.sample_model.to_string())?;

        // 电芯编号
        worksheet.write(row, 5, &channel.sample_id)?;

        // 测试状态
        worksheet.write(row, 6, channel.test_status.to_string())?;

        // 测试温度
        worksheet.write(row, 7, channel.test_temperature.to_string())?;

        // 初始容量（可选，带格式）
        if let Some(capacity) = channel.initial_capacity {
            worksheet.write_with_format(row, 8, capacity, &number_format)?;
        }

        // 当前容量（可选，带格式）
        if let Some(capacity) = channel.current_capacity {
            worksheet.write_with_format(row, 9, capacity, &number_format)?;
        }

        // 接续前圈数（可选，带格式）
        if let Some(cycles) = channel.previous_cycles {
            worksheet.write_with_format(row, 10, cycles as i64, &integer_format)?;
        }

        // 当前圈数（可选，带格式）
        if let Some(cycles) = channel.current_cycles {
            worksheet.write_with_format(row, 11, cycles as i64, &integer_format)?;
        }

        // 先前测试通道（可选）
        if let Some(channel_prv) = &channel.channel_prv {
            worksheet.write(row, 12, channel_prv)?;
        }
    }

    // 添加自动筛选
    worksheet.autofilter(0, 0, 0, 12)?;

    // 冻结第一行（表头）
    worksheet.set_freeze_panes(1, 0)?;

    // 保存文件
    workbook.save(output_path)?;

    println!("✅ 数据已成功写入到: {}", output_path.display());
    Ok(())
}

fn xlsx_reader(path: &PathBuf) -> Result<Vec<ChannelInfo>> {
    let mut reader: Xlsx<_> = open_workbook(path).context("无法打开文件")?;

    let sheet = reader
        .worksheet_range(CYCLE_SHEET)
        .context("找不到指定工作表")?;

    let rows: Vec<_> = sheet.rows().collect();
    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let indices = ColumnIndices::from_header_row(rows[0])?;

    let channel_list = rows
        .iter()
        .skip(1)
        .filter_map(|row| {
            // 跳过设备列为空的行
            row.get(indices.device)?;

            Some(ChannelInfo {
                device: cell_to_string(row, indices.device),
                channel: cell_to_string(row, indices.channel),
                channel_prv: indices
                    .channel_prv
                    .and_then(|i| Some(cell_to_string(row, i))),
                project_code: cell_to_string(row, indices.project_code),
                sample_person: indices
                    .sample_person
                    .and_then(|i| Some(cell_to_string(row, i))),
                sample_model: SampleModel::from(&cell_to_string(row, indices.sample_model)),
                sample_id: cell_to_string(row, indices.sample_id),
                test_status: TestStatus::from(&cell_to_string(row, indices.test_status)),
                test_temperature: TestTemperature::from(&cell_to_string(
                    row,
                    indices.test_temperature,
                )),
                initial_capacity: parse_cell_f32(row, indices.initial_capacity),
                current_capacity: parse_cell_f32(row, indices.current_capacity),
                previous_cycles: parse_cell_usize(row, indices.previous_cycles),
                current_cycles: parse_cell_usize(row, indices.current_cycles),
                form_path: None,
            })
        })
        .collect();

    Ok(channel_list)
}

// 打印整个 Vec<ChannelInfo> 的表格形式
fn print_channel_list_table(channel_list: &[ChannelInfo]) {
    if channel_list.is_empty() {
        println!("{}", "没有找到通道数据".yellow().bold());
        return;
    }

    // 打印表格标题
    println!("\n{}", "电池测试通道信息".bold().cyan().on_black());
    println!("{}", "=".repeat(120).cyan());

    // 表格头部
    println!(
        "{:<5} {:<6} {:<5} {:<8} {:<8} {:<6} {:<5} {:<5} {:<8} {:<10} {:<8}",
        "序号".bold(),
        "设备".bold(),
        "通道".bold(),
        "状态".bold(),
        "申请单号".bold(),
        "电芯型号".bold(),
        "电芯编号".bold(),
        "测试温度".bold(),
        "当前容量".bold(),
        "循环数".bold(),
        "送样人".bold()
    );
    println!("{}", "─".repeat(120).dimmed());

    // 表格内容
    for (i, channel) in channel_list.iter().enumerate() {
        let status_str = format!("{:?}", channel.test_status);
        let status_display = match channel.test_status {
            TestStatus::Long => status_str.green().bold(),
            // TestStatus::Completed => status_str.blue(),
            TestStatus::None => status_str.dimmed(),
            // TestStatus::Error => status_str.red().bold(),
            // _ => status_str.normal(),
        };

        let capacity_display = if let Some(cap) = channel.current_capacity {
            format!("{:.2}", cap).green()
        } else {
            "N/A".dimmed()
        };

        let cycles_display = format!(
            "{} / {}",
            channel.previous_cycles.unwrap_or(0),
            channel.current_cycles.unwrap_or(0)
        );

        let sample_person_display = channel
            .sample_person
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("N/A")
            .dimmed();

        println!(
            "{:<6} {:<8} {:<8} {:<8} {:<15} {:<8} {:<12} {:<8} {:<10} {:<15} {:<10}",
            format!("{}", i + 1).bold(),
            channel.device,
            channel.channel.cyan(),
            status_display,
            channel.project_code,
            format!("{:?}", channel.sample_model),
            channel.sample_id.yellow(),
            format!("{:?}", channel.test_temperature),
            capacity_display,
            cycles_display.magenta(),
            sample_person_display
        );
    }

    println!("{}", "=".repeat(120).cyan());
    println!(
        "{}: {}",
        "总计通道数".bold(),
        channel_list.len().to_string().green().bold()
    );
}

#[derive(Debug)]
pub struct ChannelInfo {
    device: String,
    channel: String,
    channel_prv: Option<String>,
    project_code: String,
    sample_person: Option<String>,
    sample_model: SampleModel,
    sample_id: String,
    test_status: TestStatus,
    test_temperature: TestTemperature,
    initial_capacity: Option<f32>,
    current_capacity: Option<f32>,
    previous_cycles: Option<usize>,
    current_cycles: Option<usize>,
    form_path: Option<PathBuf>,
}

impl ChannelInfo {
    fn get_active(&self) -> Option<ChannelActive> {
        if self.test_status.is_long() && !self.project_code.is_empty() {
            let device = self.device.clone();
            let channel = self.channel.clone();
            let channel_prv = self.channel_prv.clone();
            let first_run = if self.initial_capacity.is_none() {
                true
            } else {
                false
            };
            let channel_active = ChannelActive {
                device,
                channel,
                channel_prv,
                first_run,
            };
            return Some(channel_active);
        }
        None
    }
    fn get_device(&self) -> String {
        self.device.clone()
    }
    fn update_initial_capacity(&mut self, capacity: f32) {
        if self.initial_capacity.is_none() {
            self.initial_capacity = Some(capacity);
        }
    }
    fn update_capacity(&mut self, new: f32) {
        self.current_capacity = Some(new);
    }
    fn update_cycles(&mut self, new: usize) {
        let prev = self.previous_cycles.unwrap_or(0);
        let curt = self.current_cycles.unwrap_or(0);

        // let cont = curt - prev;

        if new > curt {
            //新的圈数大于现有圈数
            self.current_cycles = Some(new);
            self.previous_cycles = Some(prev); //如果为空就自动变为0
        } else {
            let new_total = prev + new;
            if new_total < curt {
                // let project_code = self.project_code.clone();
                // let sample_id = self.sample_id.clone();
                //启动对话框，询问实际圈数
                match self.show_cycle_dialog(curt, new_total) {
                    DialogResult::Cancel => {
                        println!("用户取消，放弃更新");
                        return; // 放弃更新
                    }
                    DialogResult::Ignore => {
                        println!("用户选择忽略，强行更新数据");
                        self.current_cycles = Some(new_total);
                    }
                    DialogResult::Confirm(actual_cycles) => {
                        println!("用户确认，实际圈数为: {}", actual_cycles);

                        // 根据你的业务逻辑，这里可能需要调整
                        // 如果是没加200圈的情况
                        if new_total < curt {
                            // 可以选择加200圈
                            // self.current_cycles = Some(actual_cycles + 200);

                            // 或者直接使用用户输入的值
                            self.current_cycles = Some(actual_cycles);
                        } else {
                            self.current_cycles = Some(actual_cycles);
                        }
                    }
                }
                // self.current_cycles = Some(new_total + 200); //接续后小于现有，说明没加200圈
            } else {
                self.current_cycles = Some(new_total);
            }
            // if new > cont {
            //     //但是新的圈数大于接续后的圈数
            //     self.current_cycles = Some(prev + new); //获取参数，故使用new
            // } else {
            //     //新的圈数也小于接续后的圈数，说明开启新的200圈DCR了
            //     self.previous_cycles = Some(prev + 200);
            //     self.current_cycles = Some(prev + 200 + new);
            // }
        }
    }
    fn show_cycle_dialog(&self, current: usize, calculated: usize) -> DialogResult {
        println!("\n=== 圈数更新确认 ===");
        println!("项目: {}", self.project_code);
        println!("样本ID: {}", self.sample_id);
        println!("当前记录圈数: {}", current);
        println!("计算得到的新圈数: {}", calculated);
        println!("计算值小于当前值，请确认实际圈数。\n");

        loop {
            // 显示选项
            let options = vec!["1. 取消更新", "2. 忽略并强行更新", "3. 输入实际圈数"];

            let selection = Select::new()
                .with_prompt("请选择操作:")
                .items(&options)
                .default(0)
                .interact()
                .unwrap_or(0);

            match selection {
                0 => {
                    // 取消更新
                    if Confirm::new()
                        .with_prompt("确定要取消更新吗？")
                        .default(false)
                        .interact()
                        .unwrap_or(false)
                    {
                        return DialogResult::Cancel;
                    }
                    // 如果不确定，继续循环
                }
                1 => {
                    // 忽略并强行更新
                    if Confirm::new()
                        .with_prompt("确定要忽略并强行更新数据吗？")
                        .default(false)
                        .interact()
                        .unwrap_or(false)
                    {
                        return DialogResult::Ignore;
                    }
                    // 如果不确定，继续循环
                }
                2 => {
                    // 输入实际圈数
                    match self.get_valid_cycle_input() {
                        Ok(cycles) => return DialogResult::Confirm(cycles),
                        Err(_) => {
                            println!("输入无效，请重新选择操作");
                            // 继续循环
                        }
                    }
                }
                _ => {
                    println!("无效选择，请重试");
                }
            }
        }
    }

    fn get_valid_cycle_input(&self) -> Result<usize, String> {
        loop {
            let input: String = Input::new()
                .with_prompt("请输入实际圈数 (正整数，0-4294967295)")
                .allow_empty(false)
                .interact_text()
                .map_err(|e| format!("输入错误: {}", e))?;

            // 尝试解析为 u32
            match u32::from_str(&input.trim()) {
                Ok(value) => {
                    // 检查是否为正数（根据业务需求，0可能有效也可能无效）
                    if value == 0 {
                        println!("警告: 圈数为0，请确认是否正确");
                        if Confirm::new()
                            .with_prompt("确认使用0作为圈数吗？")
                            .default(false)
                            .interact()
                            .unwrap_or(false)
                        {
                            return Ok(0);
                        } else {
                            continue; // 重新输入
                        }
                    }
                    return Ok(value as usize);
                }
                Err(e) => {
                    println!("输入无效: {}，请重新输入", e);
                    // 继续循环
                }
            }
        }
    }
    fn update_form_path(&mut self, path: PathBuf) {
        self.form_path = Some(path);
    }
}

// 定义对话框结果枚举
enum DialogResult {
    Cancel,      // 取消，放弃更新
    Ignore,      // 忽略，强行更新
    Confirm(usize), // 确认，使用输入的圈数
}

#[derive(Debug)]
pub enum TestStatus {
    Long,
    None,
}

#[derive(Debug)]
pub enum TestTemperature {
    T25,
    T35,
    T45,
    T55,
    None,
}

#[derive(Debug)]
pub enum SampleModel {
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

impl TestStatus {
    pub fn from(text: &str) -> TestStatus {
        match text {
            "长期" => TestStatus::Long,
            _ => TestStatus::None,
        }
    }
    pub fn is_long(&self) -> bool {
        if let TestStatus::Long = self {
            true
        } else {
            false
        }
    }
}

impl TestTemperature {
    pub fn from(text: &str) -> Self {
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

impl SampleModel {
    pub fn from(text: &str) -> Self {
        match text {
            "铝壳50" => SampleModel::H50,
            "65" => SampleModel::H65,
            "100" => SampleModel::H100,
            "150" => SampleModel::H150,
            "180" => SampleModel::H180,
            "280" => SampleModel::H280,
            "软包50" => SampleModel::S50,
            "钠电165" => SampleModel::Na165,
            "钠电170" => SampleModel::Na170,
            _ => SampleModel::None,
        }
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

impl fmt::Display for SampleModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SampleModel::H50 => write!(f, "铝壳50"),
            SampleModel::H65 => write!(f, "铝壳65"),
            SampleModel::H100 => write!(f, "铝壳100"),
            SampleModel::H150 => write!(f, "铝壳150"),
            SampleModel::H180 => write!(f, "铝壳180"),
            SampleModel::H280 => write!(f, "铝壳280"),
            SampleModel::S50 => write!(f, "软包50"),
            SampleModel::Na165 => write!(f, "钠电165"),
            SampleModel::Na170 => write!(f, "钠电170"),
            SampleModel::None => write!(f, "未设置"),
        }
    }
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestStatus::Long => write!(f, "长期"),
            TestStatus::None => write!(f, "离线"),
        }
    }
}

// 为 ChannelInfo 实现 Display trait
impl fmt::Display for ChannelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "┌{0:─<50}┐\n", "")
            .and_then(|_| write!(f, "│ {:<48} │\n", "通道信息".bold()))
            .and_then(|_| write!(f, "├{0:─<50}┤\n", ""))
            .and_then(|_| write!(f, "│ {:<16}: {:<30} │\n", "设备", self.device))
            .and_then(|_| write!(f, "│ {:<16}: {:<30} │\n", "通道", self.channel))
            .and_then(|_| {
                if let Some(channel_prv) = &self.channel_prv {
                    write!(f, "│ {:<16}: {:<30} │\n", "先前通道", channel_prv)
                } else {
                    write!(f, "│ {:<16}: {:<30} │\n", "先前通道", "未设置".dimmed())
                }
            })
            .and_then(|_| {
                write!(
                    f,
                    "│ {:<16}: {:<30} │\n",
                    "申请单号",
                    self.project_code.cyan()
                )
            })
            .and_then(|_| {
                if let Some(sample_person) = &self.sample_person {
                    write!(f, "│ {:<16}: {:<30} │\n", "送样人", sample_person)
                } else {
                    write!(f, "│ {:<16}: {:<30} │\n", "送样人", "未设置".dimmed())
                }
            })
            .and_then(|_| write!(f, "│ {:<16}: {:<30} │\n", "电芯型号", self.sample_model))
            .and_then(|_| {
                write!(
                    f,
                    "│ {:<16}: {:<30} │\n",
                    "电芯编号",
                    self.sample_id.yellow()
                )
            })
            .and_then(|_| {
                write!(
                    f,
                    "│ {:<16}: {:<30} │\n",
                    "测试状态",
                    format!("{:?}", self.test_status)
                )
            })
            .and_then(|_| write!(f, "│ {:<16}: {:<30} │\n", "测试温度", self.test_temperature))
            .and_then(|_| {
                if let Some(ic) = self.initial_capacity {
                    write!(f, "│ {:<16}: {:<9.2} Ah{:<21} │\n", "初始容量", ic, "")
                } else {
                    write!(f, "│ {:<16}: {:<30} │\n", "初始容量", "未设置".dimmed())
                }
            })
            .and_then(|_| {
                if let Some(cc) = self.current_capacity {
                    write!(f, "│ {:<16}: {:<9.2} Ah{:<21} │\n", "当前容量", cc, "")
                } else {
                    write!(f, "│ {:<16}: {:<30} │\n", "当前容量", "未设置".dimmed())
                }
            })
            .and_then(|_| {
                let cycles = format!(
                    "{} / {}",
                    self.previous_cycles.unwrap_or(0),
                    self.current_cycles.unwrap_or(0)
                );
                write!(f, "│ {:<16}: {:<30} │\n", "循环次数", cycles.magenta())
            })
            .and_then(|_| write!(f, "└{0:─<50}┘\n", ""))
    }
}
