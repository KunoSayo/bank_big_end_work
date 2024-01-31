use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use chrono::Utc;
use log::info;
use sqlx::{Executor, MySqlPool, query};
use sqlx::mysql::MySqlQueryResult;

use crate::bank::BankConnection;
use crate::network::{DataHandler, DataHandlerGenerator};

pub struct Inner {
    pub sql_pool: MySqlPool,
}

#[derive(Clone)]
pub struct BankServer(pub(crate) Arc<Inner>);

impl BankServer {
    pub async fn new() -> anyhow::Result<Self> {
        let sql_pool = sqlx::mysql::MySqlPoolOptions::new()
            .connect(std::env::var("sql_url").unwrap().as_ref())
            .await.unwrap();

        // init sql table
        let result = sql_pool.execute(r#"CREATE TABLE IF NOT EXISTS `bank_user` (
  `id` INTEGER PRIMARY KEY,
  `password` INTEGER NOT NULL,
  `balance` INTEGER UNSIGNED NOT NULL DEFAULT '0',
  `name` VARCHAR(90),
  `phone_number` varchar(20));

    CREATE TABLE IF NOT EXISTS `trade_logs` (`tid` int NOT NULL AUTO_INCREMENT PRIMARY KEY,`receiver` INTEGER NOT NULL, `sender` VARCHAR(30) NOT NULL, `time` DATETIME NOT NULL, `amount` INTEGER NOT NULL);

    CREATE EVENT IF NOT EXISTS interest_calculator
ON SCHEDULE EVERY 1 DAY
DO
BEGIN
    UPDATE bank_user SET balance = balance * (1 + 1 / 12);
END;

  "#).await?;
        info!("SQL init execute result: {:?}", result);

        let inner = Inner { sql_pool };
        info!("Connected sql and got bank server instance");
        Ok(Self {
            0: inner.into(),
        })
    }

    pub async fn insert_trade_log(&self, receiver: u32, sender: &str, amount: i32) -> anyhow::Result<()> {
        let now = chrono::DateTime::<Utc>::from(SystemTime::now());
        let mut con = self.0.sql_pool.acquire().await?;
        let statement = query("INSERT INTO trade_logs(receiver, sender, time, amount) VALUES(?, ?, ?, ?);")
            .bind(receiver)
            .bind(sender)
            .bind(now)
            .bind(amount);
        let result = statement.execute(con.as_mut()).await?;
        info!("Inserted trade log {:?}", result);
        Ok(())
    }

    pub async fn add_balance(&self, who: u32, sender: &str, amount: u32) -> anyhow::Result<MySqlQueryResult> {
        let mut con = self.0.sql_pool.acquire().await?;
        let result = query("UPDATE bank_user SET balance=balance+? WHERE id=?")
            .bind(amount)
            .bind(who)
            .execute(con.as_mut()).await?;
        self.insert_trade_log(who, sender, amount as i32).await?;

        Ok(result)
    }
}

impl DataHandlerGenerator for BankServer {
    fn generate(&self, _: SocketAddr) -> Box<dyn DataHandler> {
        Box::new(BankConnection::new(self.clone()))
    }
}