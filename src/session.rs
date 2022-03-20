use crate::settings::Settings;

use postgres::{types::ToSql, Client, Error as PostgresError, NoTls, Row, ToStatement};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Error with database: {0}")]
    PostgresError(#[from] PostgresError),
}

pub struct Session {
    client: Client,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session {{ client: <hidden> }}")
    }
}

impl Session {
    pub fn build() -> Result<Self, SessionError> {
        let settings = Settings::build();
        let client = Client::connect(&settings.message_db_url, NoTls)?;

        Ok(Self { client })
    }

    // TODO: Better way to handle this? Seems odd to "expose" implementation types though what else could I do other then wrap them ...
    pub fn query<T>(
        &mut self,
        query: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, SessionError>
    where
        T: ?Sized + ToStatement,
    {
        self.client.query(query, params).map_err(SessionError::from)
    }
}
