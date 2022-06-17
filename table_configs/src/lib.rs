mod templates {
    pub const CONFIG_TEMPLATE: &str = r#"{
        "start_date": "YYYY-MM-DD",
        "range": uint,
        "output_file_name":"name",
        "send_time":"HH:MM:SS",
        "reset_time":"HH:MM:SS",
        "maintainer":"phone_number",
        "alert_day":5, //1=Sunday 7=Saturday
        "weekend":[x,y,z]//1=Sunday 7=Saturday[6,7]=Friday and Saturday
    }"#;

    pub const NAMES_TEMPLATE: &str = r#"name,phone
John,9725130465
Maddy,972541235467
Kaladin,972468578448
"#;

    pub const EXCLUDED_HOLIDAYS_TEMPLATE: &str = r#"names
Purim
Yom HaShoah
Yom HaZikaron
Pesach Sheni
Yom Yerushalayim"#;
}

pub mod paths {
    use platform_dirs::UserDirs;
    use std::{io::Write, path::Path};

    use crate::templates;
    const EXCLUDED_DATES_PATH_UNIX: &str = "config/excluded_hebcal.csv";
    const EXCLUDED_DATES_PATH_WIN: &str = "config\\excluded_hebcal.csv";
    const NAMES_PATH_UNIX: &str = "config/names.csv";
    const NAMES_PATH_WIN: &str = "config\\names.csv";
    const HEBDATE_PATH_UNIX: &str = "config/heb_date.json";
    const HEBDATE_PATH_WIN: &str = "config\\heb_date.json";
    const CONFIG_PATH_UNIX: &str = "config/config.json";
    const CONFIG_PATH_WIN: &str = "config\\config.json";
    const OUTPUT_DIR_PATH_UNIX: &str = "output/";
    const OUTPUT_DIR_PATH_WIN: &str = "output\\";

    fn get_app_dir() -> String {
        let path = UserDirs::new().unwrap();
        if cfg!(windows) {
            format!(
                "{}\\{}\\",
                path.document_dir.display(),
                "cleaning_managment"
            )
        } else if cfg!(unix) {
            format!("{}/{}/", path.document_dir.display(), "cleaning_managment")
        } else {
            panic!()
        }
    }

    pub fn get_root_dir_path() -> String {
        get_app_dir()
    }
    pub fn get_excluded_holidays_path() -> String {
        if cfg!(windows) {
            format!("{}{}", get_app_dir(), EXCLUDED_DATES_PATH_WIN)
        } else if cfg!(unix) {
            format!("{}{}", get_app_dir(), EXCLUDED_DATES_PATH_UNIX)
        } else {
            panic!()
        }
    }
    pub fn get_names_path() -> String {
        if cfg!(windows) {
            format!("{}{}", get_app_dir(), NAMES_PATH_WIN)
        } else if cfg!(unix) {
            format!("{}{}", get_app_dir(), NAMES_PATH_UNIX)
        } else {
            panic!()
        }
    }
    pub fn get_hebdate_path() -> String {
        if cfg!(windows) {
            format!("{}{}", get_app_dir(), HEBDATE_PATH_WIN)
        } else if cfg!(unix) {
            format!("{}{}", get_app_dir(), HEBDATE_PATH_UNIX)
        } else {
            panic!()
        }
    }
    pub fn get_config_path() -> String {
        if cfg!(windows) {
            format!("{}{}", get_app_dir(), CONFIG_PATH_WIN)
        } else if cfg!(unix) {
            format!("{}{}", get_app_dir(), CONFIG_PATH_UNIX)
        } else {
            panic!()
        }
    }
    pub fn get_output_path(filename: &str) -> String {
        if cfg!(windows) {
            format!("{}{}{}", get_app_dir(), OUTPUT_DIR_PATH_WIN, filename)
        } else if cfg!(unix) {
            format!("{}{}{}", get_app_dir(), OUTPUT_DIR_PATH_UNIX, filename)
        } else {
            panic!()
        }
    }
    pub fn init() -> Result<bool, Box<dyn std::error::Error>> {
        let mut all_init = true;
        all_init =
            create_if_doesnt_exists(&get_names_path(), templates::NAMES_TEMPLATE)? && all_init;
        all_init =
            create_if_doesnt_exists(&get_config_path(), templates::CONFIG_TEMPLATE)? && all_init;
        all_init = create_if_doesnt_exists(
            &get_excluded_holidays_path(),
            templates::EXCLUDED_HOLIDAYS_TEMPLATE,
        )? && all_init;
        std::fs::create_dir_all(get_output_path(""))?;
        Ok(all_init)
    }
    fn create_if_doesnt_exists(
        path: &str,
        default_text: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let path = Path::new(path);
        if !path.exists() {
            let path = std::path::Path::new(path);
            let parent = path.parent().unwrap();
            std::fs::create_dir_all(parent)?;
            std::fs::write(path, default_text)?;
            println!("Create template in: {}", path.to_str().unwrap());
            std::io::stdout().flush()?;
            return Ok(false);
        }
        Ok(true)
    }
}
pub mod config {
    use crate::paths;
    use chrono::{NaiveDate, NaiveTime};
    use serde::Deserialize;
    use serde_json;

    pub fn load_config() -> Config {
        let config =
            std::fs::read_to_string(paths::get_config_path()).expect("Config file not found.");
        let config: ConfigRaw =
            serde_json::from_str(&config).expect("Could not parse config file into json.");
        let config = Config::from(config);
        config
    }

    #[derive(Deserialize)]
    pub struct ConfigRaw {
        pub start_date: String,
        pub range: usize,
        pub output_file_name: String,
        pub send_time: String,
        pub reset_time: String,
        pub maintainer: String,
        pub alert_day: usize,
        pub weekend: Vec<usize>,
    }

    #[derive(Debug, Clone)]
    pub struct Config {
        pub start_date: NaiveDate,
        pub range: usize,
        pub output_file_name: String,
        pub send_time: NaiveTime,
        pub reset_time: NaiveTime,
        pub maintainer: String,
        pub alert_day: chrono::Weekday,
        pub weekend: Vec<chrono::Weekday>,
    }
    impl Config {
        pub fn from(config: ConfigRaw) -> Self {
            Self {
                output_file_name: format!("{}.csv", config.output_file_name),
                range: config.range,
                start_date: NaiveDate::parse_from_str(&config.start_date, "%Y-%m-%d").unwrap(),
                send_time: NaiveTime::parse_from_str(&config.send_time, "%H:%M:%S").unwrap(),
                reset_time: NaiveTime::parse_from_str(&config.reset_time, "%H:%M:%S").unwrap(),
                maintainer: config.maintainer,
                alert_day: int_to_weekday(config.alert_day),
                weekend: config.weekend.iter().map(|x| int_to_weekday(*x)).collect(),
            }
        }
    }
    //sunday=1,saturday = 7
    pub fn int_to_weekday(i: usize) -> chrono::Weekday {
        use chrono::Weekday;
        let weekday = [
            Weekday::Sun,
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
        ];
        if let Some(day) = weekday.get(i - 1) {
            return day.clone();
        }
        panic!("weekday config has wrong number. Sun = 1.");
    }
}
