pub mod templates{
    const config_template:&str=r#"{
        "start_date": "YYYY-MM-DD",
        "range": uint,
        "output_path":"",
        "send_time":"HH:MM:SS",
        "reset_time":"HH:MM:SS",
        "maintainer":"phone_number",
        "alert_day":"1-7"//1=Sunday
    }"#;

    const names_template:&str = 
r#"name,phone
John,***REMOVED***
Maddy,***REMOVED***
Kaladin,***REMOVED***
"#;

    const names:&str = 
r#"names
Purim
Yom HaShoah
Yom HaZikaron
Pesach Sheni
Yom Yerushalayim"#;
}

pub mod paths{
    const EXCLUDED_DATES_PATH_UNIX: &str = "./config/excluded_hebcal.csv";
    const EXCLUDED_DATES_PATH_WIN: &str = "./config/excluded_hebcal.csv";
    const SOLDIERS_PATH: &str = "./config/names.csv";
    const HEBDATE_PATH: &str = "./config/heb_date.json";
    pub const CONFIG_PATH: &str = "./config/config.json";
}