use regex::Regex;
use std::fmt;

#[derive(Debug)]
pub enum TestStatus {
    Long,
    None,
}

impl TestStatus {
    pub fn from(text: &str) -> TestStatus {
        match text {
            "长期" => TestStatus::Long,
            _ => TestStatus::None,
        }
    }
}

#[derive(Debug)]
pub enum TestTemperature {
    T25,
    T35,
    T45,
    T55,
    None,
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
            TestTemperature::None => write!(f, "N/A"),
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
            SampleModel::None => write!(f, "N/A"),
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
