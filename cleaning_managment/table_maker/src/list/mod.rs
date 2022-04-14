mod parse_soldiers;
pub use self::parse_soldiers::Soldier;
use chrono::NaiveDate;

struct Row{
    soldier:Soldier,
    date:NaiveDate,
}

