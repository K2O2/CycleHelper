use anyhow::{Context, Result};
// config_reader.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceConfig {
    /// 计算机名，格式：DX-XX
    pub computer_name: String,
    /// 设备通道数
    pub channel_count: u32,
    /// 设备通道名列表，格式：XXX-XXX（任意数字）
    pub channel_names: Vec<String>,
}

impl DeviceConfig {
    /// 从指定路径读取TOML配置文件
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件: {}", path.as_ref().display()))?;

        let config: DeviceConfig = toml::from_str(&content)
            .with_context(|| format!("TOML解析失败: {}", path.as_ref().display()))?;

        // 验证配置数据
        config.validate()?;

        Ok(config)
    }

    /// 将配置写入到指定文件
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        // 写入前再次验证数据
        self.validate()?;

        let toml_content = toml::to_string_pretty(self)
            .context("TOML序列化失败")?;

        fs::write(&path, toml_content)
            .with_context(|| format!("无法写入配置文件: {}", path.as_ref().display()))?;

        println!("配置已成功写入: {}", path.as_ref().display());
        Ok(())
    }

    /// 创建新的配置实例
    pub fn new(computer_name: String, channel_names: Vec<String>) -> Self {
        let channel_count = channel_names.len() as u32;
        Self {
            computer_name,
            channel_count,
            channel_names,
        }
    }

    /// 更新计算机名
    pub fn with_computer_name(mut self, computer_name: String) -> Self {
        self.computer_name = computer_name;
        self.channel_count = self.channel_names.len() as u32;
        self
    }

    /// 更新通道名列表
    pub fn with_channel_names(mut self, channel_names: Vec<String>) -> Self {
        self.channel_names = channel_names;
        self.channel_count = self.channel_names.len() as u32;
        self
    }

    /// 添加通道名
    pub fn add_channel_name(&mut self, channel_name: String) {
        self.channel_names.push(channel_name);
        self.channel_count = self.channel_names.len() as u32;
    }

    /// 验证配置数据的有效性
    fn validate(&self) -> Result<()> {
        // 验证计算机名格式
        // if !self.computer_name.starts_with("DX-") {
        //     anyhow::bail!("计算机名格式错误，必须以 'DX-' 开头: {}", self.computer_name);
        // }

        // 验证通道数匹配
        if self.channel_count as usize != self.channel_names.len() {
            anyhow::bail!(
                "通道数不匹配: 配置的通道数为 {}，但提供的通道名数量为 {}",
                self.channel_count,
                self.channel_names.len()
            );
        }

        // 验证通道名格式
        for (index, name) in self.channel_names.iter().enumerate() {
            if !name.chars().all(|c| c.is_ascii_digit() || c == '-') {
                anyhow::bail!("通道名格式错误，只能包含数字和破折号: {}", name);
            }

            let parts: Vec<&str> = name.split('-').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                anyhow::bail!("通道名格式错误，必须是 XXX-XXX 格式: {}", name);
            }

            if !parts[0].chars().all(|c| c.is_ascii_digit()) ||
                !parts[1].chars().all(|c| c.is_ascii_digit()) {
                anyhow::bail!("通道名格式错误，必须全部为数字: {}", name);
            }
        }

        Ok(())
    }

    /// 获取指定通道的名称
    pub fn get_channel_name(&self, channel_index: u32) -> Option<&str> {
        if channel_index < self.channel_count {
            self.channel_names.get(channel_index as usize).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// 打印配置信息
    pub fn print_info(&self) {
        println!("计算机名: {}", self.computer_name);
        println!("设备通道数: {}", self.channel_count);
        println!("设备通道名:");
        for (i, name) in self.channel_names.iter().enumerate() {
            println!("  通道 {}: {}", i, name);
        }
    }

    /// 将配置转换为TOML字符串
    pub fn to_toml_string(&self) -> Result<String> {
        self.validate()?;
        toml::to_string_pretty(self).context("TOML序列化失败")
    }
}

/// 配置管理模块
// pub mod config_manager {
//     use super::*;
//     use std::collections::HashMap;
//     use std::path::PathBuf;
//
//     pub struct ConfigManager {
//         configs: HashMap<PathBuf, DeviceConfig>,
//     }
//
//     impl ConfigManager {
//         pub fn new() -> Self {
//             Self {
//                 configs: HashMap::new(),
//             }
//         }
//
//         /// 加载指定目录下的所有TOML配置文件
//         pub fn load_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<Vec<PathBuf>> {
//             let mut loaded_files = Vec::new();
//             let dir = fs::read_dir(&dir_path)
//                 .with_context(|| format!("无法读取目录: {}", dir_path.as_ref().display()))?;
//
//             for entry in dir {
//                 let entry = entry?;
//                 let path = entry.path();
//
//                 if path.extension().map_or(false, |ext| ext == "toml") {
//                     match DeviceConfig::from_file(&path) {
//                         Ok(config) => {
//                             self.configs.insert(path.clone(), config);
//                             loaded_files.push(path);
//                         }
//                         Err(e) => {
//                             eprintln!("警告: 无法加载配置文件 {}: {}", path.display(), e);
//                         }
//                     }
//                 }
//             }
//
//             Ok(loaded_files)
//         }
//
//         /// 添加配置到管理器
//         pub fn add_config<P: AsRef<Path>>(&mut self, path: P, config: DeviceConfig) -> Result<()> {
//             config.validate()?;
//             self.configs.insert(path.as_ref().to_path_buf(), config);
//             Ok(())
//         }
//
//         /// 保存指定路径的配置
//         pub fn save_config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
//             if let Some(config) = self.configs.get(path.as_ref()) {
//                 config.to_file(path)
//             } else {
//                 anyhow::bail!("配置不存在: {}", path.as_ref().display())
//             }
//         }
//
//         /// 保存所有配置到文件
//         pub fn save_all_configs(&self) -> Result<()> {
//             for (path, config) in &self.configs {
//                 config.to_file(path)?;
//             }
//             println!("所有配置已保存");
//             Ok(())
//         }
//
//         /// 获取指定路径的配置
//         pub fn get_config<P: AsRef<Path>>(&self, path: P) -> Option<&DeviceConfig> {
//             self.configs.get(path.as_ref())
//         }
//
//         /// 获取可变引用配置（用于修改）
//         pub fn get_config_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut DeviceConfig> {
//             self.configs.get_mut(path.as_ref())
//         }
//
//         /// 获取所有配置
//         pub fn get_all_configs(&self) -> &HashMap<PathBuf, DeviceConfig> {
//             &self.configs
//         }
//
//         /// 从管理器中移除配置
//         pub fn remove_config<P: AsRef<Path>>(&mut self, path: P) -> Option<DeviceConfig> {
//             self.configs.remove(path.as_ref())
//         }
//     }
// }

// 配置构建器，提供更流畅的API
pub struct ConfigBuilder {
    computer_name: String,
    channel_names: Vec<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            computer_name: "DX-01".to_string(),
            channel_names: Vec::new(),
        }
    }

    pub fn computer_name(mut self, name: &str) -> Self {
        self.computer_name = name.to_string();
        self
    }

    pub fn add_channel(mut self, channel_name: &str) -> Self {
        self.channel_names.push(channel_name.to_string());
        self
    }

    pub fn build(self) -> Result<DeviceConfig> {
        let config = DeviceConfig::new(self.computer_name, self.channel_names);
        config.validate()?;
        Ok(config)
    }
}