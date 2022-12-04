use crate::data_model::{time_point_to_string, Plot, TimeSeries, TimeSeriesEntry};
use crate::errors::HandlingError;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Error, Result};

pub struct Dao {
    conn: Connection,
}

impl Dao {
    fn set_up(&self) -> Result<(), Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS plot (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL
        )",
            (), // empty list of parameters.
        )?;

        self.conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS index_plot_name
            ON plot ( name )",
            (), // empty list of parameters.
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS time_series (
            id INTEGER PRIMARY KEY,
            plot_id INTEGER,
            name TEXT NOT NULL,
            unit TEXT NOT NULL,
            FOREIGN KEY(plot_id) REFERENCES plot(id)
        )",
            (), // empty list of parameters.
        )?;
        self.conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS index_time_series_name
            ON time_series ( plot_id, name )",
            (), // empty list of parameters.
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS time_series_entry (
            time_series_id INTEGER,
            date TEXT NOT NULL,
            value REAL NOT NULL,
            FOREIGN KEY(time_series_id) REFERENCES time_series(id)
        )",
            (), // empty list of parameters.
        )?;

        self.conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS index_time_series_entry
            ON time_series_entry ( time_series_id, date)",
            (), // empty list of parameters.
        )?;

        Ok(())
    }

    pub fn new_in_memory() -> Result<Self, Error> {
        let dao = Self {
            conn: Connection::open_in_memory()?,
        };
        dao.set_up()?;
        Ok(dao)
    }

    pub fn get_entries_for_time_series(
        &self,
        time_series: &mut TimeSeries,
        start_date: Option<DateTime<Utc>>,
    ) -> Result<(), HandlingError> {
        let date_string = start_date
            .map(|val| time_point_to_string(&val))
            .unwrap_or("".to_string());

        let mut stmt = self.conn.prepare(
            "SELECT date, value FROM time_series_entry WHERE time_series_id = (?1) AND date >= (?2) ORDER BY date",
        )?;

        let entry_iter = stmt.query_map(params![time_series.id, date_string], |row| {
            Ok(TimeSeriesEntry::new_from_string(
                &(row.get::<_, String>(0)?),
                row.get(1)?,
            ))
        })?;

        for entry in entry_iter {
            let entry = entry??;
            time_series.time_points.push(entry.time_point);
            time_series.values.push(entry.value);
        }
        Ok(())
    }

    pub fn get_time_series(&self, id: i64) -> Result<TimeSeries, HandlingError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, unit FROM time_series WHERE id = (?1)")?;

        let time_series = stmt.query_row(params![id], |row| {
            Ok(TimeSeries {
                id: row.get(0)?,
                name: row.get(1)?,
                unit: row.get(2)?,
                time_points: vec![],
                values: vec![],
            })
        })?;

        Ok(time_series)
    }

    pub fn get_time_series_for_plot(&self, plot: &mut Plot) -> Result<(), HandlingError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, unit FROM time_series WHERE plot_id = (?1)")?;
        let time_series_iter = stmt.query_map(params![plot.id], |row| {
            Ok(TimeSeries {
                id: row.get(0)?,
                name: row.get(1)?,
                unit: row.get(2)?,
                time_points: vec![],
                values: vec![],
            })
        })?;

        for time_series in time_series_iter {
            let time_series = time_series?;
            plot.time_series.push(time_series);
        }
        Ok(())
    }

    pub fn get_plot(&self, id: i64) -> Result<Plot, HandlingError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description FROM plot WHERE id = (?1)")?;

        let plot = stmt.query_row(params![id], |row| {
            Ok(Plot {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                time_series: vec![],
            })
        })?;
        Ok(plot)
    }

    pub fn get_plot_with_data(
        &self,
        id: i64,
        start_date: Option<DateTime<Utc>>,
    ) -> Result<Plot, HandlingError> {
        let mut plot = self.get_plot(id)?;

        self.get_time_series_for_plot(&mut plot)?;

        for time_series in plot.time_series.iter_mut() {
            self.get_entries_for_time_series(time_series, start_date);
        }

        Ok(plot)
    }

    pub fn get_all_plots(&self) -> Result<Vec<Plot>, HandlingError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description FROM plot ORDER BY id")?;

        let iter = stmt.query_map(params![], |row| {
            Ok(Plot {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                time_series: vec![],
            })
        })?;

        let mut ret_val: Vec<Plot> = vec![];
        for plot in iter {
            ret_val.push(plot?);
        }
        Ok(ret_val)
    }

    pub fn add_time_series(
        &mut self,
        plot_id: i64,
        time_series: &TimeSeries,
    ) -> Result<TimeSeries, HandlingError> {
        if time_series.time_points.len() != time_series.values.len() {
            return Err(HandlingError {
                message: "Inconsistent number of time_points and values in TimeSeries.".to_string(),
                code: 420,
            });
        }

        let mut ret_val = time_series.clone();
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO time_series (name, plot_id, unit) VALUES (?1, ?2, ?3)",
            params![time_series.name.clone(), plot_id, time_series.unit.clone()],
        )?;

        let new_id: i64 = tx.last_insert_rowid();

        for iter in time_series
            .values
            .iter()
            .zip(time_series.time_points.iter())
        {
            let (value, time_point) = iter;
            let time_point = time_point_to_string(time_point);
            tx.execute(
                "INSERT INTO time_series_entry (time_series_id, date, value) VALUES (?1, ?2, ?3)",
                params![new_id, time_point, value],
            )?;
        }

        tx.commit()?;
        ret_val.id = new_id;
        Ok(ret_val)
    }

    pub fn add_plot(&mut self, plot: &Plot) -> Result<Plot, HandlingError> {
        let mut ret_val = plot.clone();
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO plot (name, description) VALUES (?1, ?2)",
            params![plot.name, plot.description],
        )?;

        let new_id: i64 = tx.last_insert_rowid();
        tx.commit()?;

        for iter in plot.time_series.iter().zip(ret_val.time_series.iter_mut()) {
            let (time_series, new_time_series) = iter;
            let added_series = self.add_time_series(new_id, &time_series)?;
            new_time_series.id = added_series.id;
        }

        ret_val.id = new_id;
        Ok(ret_val)
    }

    pub fn add_entry(
        &mut self,
        time_series_id: i64,
        entry: &TimeSeriesEntry,
    ) -> Result<(), HandlingError> {
        self.conn.execute(
            "INSERT INTO time_series_entry (time_series_id, date, value) VALUES (?1, ?2, ?3)",
            params![
                time_series_id,
                time_point_to_string(&entry.time_point),
                entry.value
            ],
        )?;
        Ok(())
    }
}
