use rusqlite::Error as SqlError;

#[derive(Debug)]
pub struct HandlingError {
    pub message: String,
    pub code: i32,
}

impl From<SqlError> for HandlingError {
    fn from(_error: SqlError) -> Self {
        HandlingError {
            message: "Database error".to_string(),
            code: 300,
        }
    }
}
