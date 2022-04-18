mod parse_soldiers;
pub use self::parse_soldiers::*;
use crate::heb_cal::HebDate;
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{ser::SerializeStruct, Serialize};

#[derive(Debug)]
pub struct Row {
    pub soldier: Soldier,
    pub date: NaiveDate,
}

impl Serialize for Row {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Row", 2)?;
        state.serialize_field("soldier", &serde_json::to_value(&self.soldier).unwrap())?;
        state.serialize_field("date", &self.date.to_string())?;
        state.end()
    }
}

pub fn get_dates(
    soldiers: Vec<Soldier>,
    holidays: Vec<HebDate>,
    start_date: NaiveDate,
    time_period: usize,
) -> Vec<Row> {
    //remove holidays from the dates list
    let mut dates = get_dates_list(start_date, time_period);
    let holidays: Vec<NaiveDate> = holidays.iter().map(|f| f.date).collect();
    dates.retain(|d| !holidays.contains(d));

    let mut rows = vec![];
    let mut iter = soldiers.iter();
    for date in dates {
        if let Some(soldier) = iter.next() {
            rows.push(Row {
                soldier: soldier.clone(),
                date,
            });
        } else {
            iter = soldiers.iter();
            if let Some(soldier) = iter.next() {
                rows.push(Row {
                    soldier: soldier.clone(),
                    date,
                });
            } else {
                panic!("Soldiers' vector is empty");
            }
        }
    }
    rows
}

fn get_dates_list(start_date: NaiveDate, time_period: usize) -> Vec<NaiveDate> {
    start_date
        .iter_days()
        .take(time_period)
        .filter(|p| p.weekday() != Weekday::Fri && p.weekday() != Weekday::Sat)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_list() {
        let start_date = NaiveDate::from_ymd(***REMOVED***, 1, 1);
        let time_period = 7;
        let res: Vec<NaiveDate> = start_date.iter_days().take(time_period).collect();
        assert!(res.iter().any(|p| p.weekday() == Weekday::Sat));
        let res = get_dates_list(start_date, time_period);
        assert!(!res.iter().any(|p| p.weekday() == Weekday::Sat))
    }

    #[test]
    fn dates_filtering() {
        let soldiers = vec![
            Soldier {
                name: "amichai".to_string(),
                phone: "***REMOVED***".to_string(),
            },
            Soldier {
                name: "Joe".to_string(),
                phone: "***REMOVED***".to_string(),
            },
        ];
        let test_date = NaiveDate::from_ymd(***REMOVED***, 3, 1);
        let holidays = vec![HebDate {
            date: test_date,
            title: "purim".to_string(),
        }];
        let start_date = NaiveDate::from_ymd(***REMOVED***, 1, 1);
        let time_period = ***REMOVED***;
        let res = get_dates(soldiers, holidays, start_date, time_period);
        assert!(!res.iter().any(|p| p.date == test_date));
        assert!(res.iter().next().unwrap().soldier.name == "amichai".to_string());
    }
}
