use anyhow::{Error, Result};
use regex::Regex;

// 预编译正则表达式
lazy_static::lazy_static! {
    static ref SHENGHONG_PATTERN1: Regex = Regex::new(r"^UN(\d+)_CN(\d+)_").unwrap();
    static ref SHENGHONG_PATTERN2: Regex = Regex::new(r"^CN(\d+)_UN(\d+)_").unwrap();

    static ref WEICHUANG_PATTERN1: Regex = Regex::new(r"^SN(\d+)_Ch(\d+)_").unwrap();
    static ref WEICHUANG_PATTERN2: Regex = Regex::new(r"^Ch(\d+)_SN(\d+)_").unwrap();

    static ref NEBULA_PATTERN: Regex = Regex::new(r"^(\d+)_(\d+)_").unwrap();
}

#[derive(Debug, PartialEq)]
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

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_weichuang() {
        // 测试SN在前，Ch在后的格式
        let result = form_name("SN101_Ch03_满充软包50Ah_CN3_20250926095254_CY1.xlsx").unwrap();
        println!("{:?}", result);

        // 测试Ch在前，SN在后的格式
        let result = form_name("Ch05_SN202_测试文件.xlsx").unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_parse_shenghong() {
        // 测试SN在前，Ch在后的格式
        let result = form_name("UN153_CN01_20251022153541_CY165.xlsx").unwrap();
        println!("{:?}", result);

        // 测试Ch在前，SN在后的格式
        let result = form_name("CN02_UN255_测试文件.xlsx").unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_parse_nebula() {
        let result = form_name("100_2_202509043814 45℃循环.xlsx").unwrap();
        println!("{:?}", result);

        // 测试多位数
        let result = form_name("1000_25_测试文件.xlsx").unwrap();
        println!("{:?}", result);
    }

    #[test]
    fn test_parse_errors() {
        // 测试无效格式
        assert!(form_name("invalid_file.txt").is_err());
        assert!(form_name("SN101_XX03_测试.xlsx").is_err()); // XX不是Ch
        assert!(form_name("UN153_XX01_测试.xlsx").is_err()); // XX不是CN
        assert!(form_name("100_测试.xlsx").is_err()); // 缺少下划线分隔
        assert!(form_name("100_2A_测试.xlsx").is_err()); // 通道号包含非数字

        //test
        let result = form_name("UN153_XX01_测试.xlsx");
        println!("{:?}", result);
    }
}
