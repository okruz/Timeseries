use chrono::{DateTime, Utc};
use serde_json::{json, Value};

pub struct TimeSeries {
    pub id: i64,
    pub name: String,
    pub unit: String,
    pub time_points: Vec<DateTime<Utc>>,
    pub values: Vec<f64>,
}

pub struct Plot {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub time_series: Vec<TimeSeries>,
}

fn to_f64_vec(json: &Value) -> Option<Vec<f64>> {
    let mut ret_val: Vec<f64> = Vec::new();
    if let Some(array) = json.as_array() {
        for entry in array {
            if let Some(float) = entry.as_f64() {
                ret_val.push(float);
            } else {
                return None;
            }
        }
        return Some(ret_val);
    }
    None
}

fn to_datetime_vec(json: &Value) -> Option<Vec<DateTime<Utc>>> {
    let mut ret_val: Vec<DateTime<Utc>> = Vec::new();
    if let Some(array) = json.as_array() {
        for entry in array {
            let mut added = false;

            if let Some(string) = entry.as_str() {
                if let Ok(time_point) = DateTime::parse_from_rfc3339(string) {
                    added = true;
                    ret_val.push(time_point.into());
                }
            }

            if !added {
                return None;
            }
        }
        return Some(ret_val);
    }
    None
}

fn to_json_array(time_points: &Vec<DateTime<Utc>>) -> Value {
    let mut ret_val: Vec<String> = Vec::new();
    for time_point in time_points {
        ret_val.push(time_point.to_rfc3339());
    }
    json!(ret_val)
}

impl TryFrom<&Value> for TimeSeries {
    type Error = ();

    fn try_from(item: &Value) -> Result<Self, Self::Error> {
        let id = item["Id"].as_i64().ok_or(())?;
        let name = item["Name"].as_str().ok_or(())?;
        let unit = item["Unit"].as_str().ok_or(())?;
        let time_points = to_datetime_vec(&item["TimePoints"]).ok_or(())?;
        let values = to_f64_vec(&item["Values"]).ok_or(())?;
        Ok(TimeSeries {
            id: id,
            name: name.to_string(),
            unit: unit.to_string(),
            time_points: time_points,
            values: values,
        })
    }
}

impl<'a> Into<Value> for &'a TimeSeries {
    fn into(self) -> Value {
        json!( {
        "Id": self.id,
        "Name": self.name,
        "Unit": self.unit,
        "TimePoints": to_json_array(&self.time_points),
        "Values": json!(self.values)
        })
    }
}

fn to_timeseries_vec(json: &Value) -> Option<Vec<TimeSeries>> {
    let mut ret_val: Vec<TimeSeries> = Vec::new();
    if let Some(array) = json.as_array() {
        for entry in array {
            let time_series: Result<TimeSeries, _> = entry.try_into();
            if let Ok(time_series) = time_series {
                ret_val.push(time_series);
            } else {
                return None;
            }
        }
        return Some(ret_val);
    }
    None
}

impl TryFrom<&Value> for Plot {
    type Error = ();

    fn try_from(item: &Value) -> Result<Self, Self::Error> {
        let id = item["Id"].as_i64().ok_or(())?;
        let name = item["Name"].as_str().ok_or(())?;
        let description = item["Description"].as_str().ok_or(())?;
        let time_series = to_timeseries_vec(&item["TimeSeries"]).ok_or(())?;

        Ok(Plot {
            id: id,
            name: name.to_string(),
            description: description.to_string(),
            time_series: time_series,
        })
    }
}

impl<'a> Into<Value> for &'a Plot {
    fn into(self) -> Value {
        let time_series_jsons: Vec<Value> = self
            .time_series
            .iter()
            .map(|series| series.into())
            .collect();
        json!( {
        "Id": self.id,
        "Name": self.name,
        "Description": self.description,
        "TimeSeries": time_series_jsons
        })
    }
}
