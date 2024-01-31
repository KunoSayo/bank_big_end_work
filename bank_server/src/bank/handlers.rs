use std::future::Future;

use anyhow::anyhow;
use bytes::{Buf, BufMut};
use chrono::{DateTime, Utc};
use log::info;
use sqlx::{MySql, query, Row};
use sqlx::pool::PoolConnection;

use crate::bank::{BankServer, UserInputError};
use crate::bank::ext::{PacketReadExt, PacketWriteExt};
use crate::bank::user::User;
use crate::network::NetworkMessage;
use crate::network::peer::Peer;

pub trait BankDataHandler: Send + 'static {
    fn handle<'a, 'b: 'a>(&'b mut self, server: &'a BankServer, src: &'a Peer, data: &'a [u8])
                          -> Box<dyn Future<Output=anyhow::Result<Option<Box<dyn BankDataHandler>>>> + Send + Unpin + 'a>;
}

/// Login packet (from client to server): (id: u32 be) (password: u32 be)
///
/// Register packet: (id: u32) (password: u32) (name: String) (phone_number: String)
#[derive(Default)]
pub struct HandleLogin {}


async fn get_user_login(mut sql: PoolConnection<MySql>, id: u32, pswd: i32) -> anyhow::Result<User> {
    let result = query("SELECT * FROM bank_user WHERE id=? AND password=?").bind(id).bind(pswd)
        .fetch_optional(sql.as_mut()).await?;
    let result = if let Some(result) = result {
        result
    } else {
        Err(UserInputError::new("账号或密码错误"))?
    };

    let user = User {
        id,
        balance: result.get("balance"),
        name: result.get::<Option<&str>, _>("name").unwrap_or("").to_string(),
        phone: result.get::<Option<&str>, _>("phone_number").unwrap_or("").to_string(),
    };

    Ok(user)
}

async fn get_user(mut sql: &mut PoolConnection<MySql>, id: u32) -> anyhow::Result<User> {
    let result = query("SELECT * FROM bank_user WHERE id=?").bind(id)
        .fetch_optional(sql.as_mut()).await?;
    let result = if let Some(result) = result {
        result
    } else {
        Err(UserInputError::new("找不到账号"))?
    };

    let user = User {
        id,
        balance: result.get("balance"),
        name: result.get::<Option<&str>, _>("name").unwrap_or("").to_string(),
        phone: result.get::<Option<&str>, _>("phone_number").unwrap_or("").to_string(),
    };

    Ok(user)
}

impl BankDataHandler for HandleLogin {
    fn handle<'a, 'b: 'a>(&'b mut self, server: &'a BankServer, src: &'a Peer, mut data: &'a [u8])
                          -> Box<dyn Future<Output=anyhow::Result<Option<Box<dyn BankDataHandler>>>> + Send + Unpin + 'a>
    {
        match data.len() {
            8 => {
                // login
                let id = data.get_u32();
                let pswd = data.get_i32();
                let task = async move {
                    let sql = server.0.sql_pool.acquire().await?;
                    let user = get_user_login(sql, id, pswd).await?;

                    let mut data = vec![];
                    data.add_header();
                    // Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
                    data.extend_from_slice(b"menu");
                    data.extend_from_slice(&id.to_be_bytes());
                    data.write_string(&user.name);
                    data.extend_from_slice(&user.balance.to_be_bytes());
                    data.write_string(&user.phone);
                    src.sender.send(NetworkMessage::Rely(data))?;

                    info!("Logged user: {}", &user.name);
                    Ok(Some(Box::new(LoggedHandler::new(user)) as _))
                };
                Box::new(Box::pin(task))
            }
            n if n > 8 => {
                // register
                let id = data.get_u32();
                let pswd = data.get_i32();


                let task = async move {
                    let name = data.read_packet_string()?;
                    let phone = data.read_packet_string()?;
                    if name.len() > 60 || phone.len() > 20 {
                        Err(UserInputError::new("输入长度错误"))?
                    }

                    let mut sql_connection = server.0.sql_pool.acquire().await?;
                    let result = query("SELECT * FROM bank_user WHERE id=?").bind(id)
                        .fetch_optional(sql_connection.as_mut()).await?;

                    if result.is_some() {
                        Err(UserInputError::new("该银行账号存在"))?;
                    }
                    info!("Now insert id {} into sql", id);

                    query("INSERT INTO bank_user VALUES(?, ?, ?, ?, ?);")
                        .bind(id)
                        .bind(pswd)
                        .bind(0)
                        .bind(&name)
                        .bind(&phone)
                        .execute(sql_connection.as_mut()).await?;


                    let mut data = vec![];
                    data.add_header();
                    // Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
                    data.extend_from_slice(b"menu");
                    data.extend_from_slice(&id.to_be_bytes());
                    data.write_string(&name);
                    data.extend_from_slice(bytemuck::cast_slice(&[0u32]));
                    data.write_string(&phone);
                    src.sender.send(NetworkMessage::Rely(data))?;


                    info!("Register user: {}", name);
                    Ok(Some(Box::new(LoggedHandler::new(User {
                        id,
                        balance: 0,
                        name,
                        phone,
                    })) as _))
                };
                Box::new(Box::pin(task))
            }
            _ => {
                Box::new(Box::pin(async { Err(anyhow!("Not correct len")) }))
            }
        }
    }
}


/// Client to server:
/// * Deposit packet: \0 amount:u32
/// * Withdraw packet: \1 amount: u32
/// * transfer packet: \2 target: u32, amount: u32
/// * info packet: \3
///
/// Server to client
/// * b"info" current_page:u32, total_page:u32, info_cnt: u32 <User>
/// * * info: tid: i32, receiver: u32 sender: String, time: (i64 u32), amount: i32
///
pub struct LoggedHandler {
    user: User,
}

impl LoggedHandler {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

impl BankDataHandler for LoggedHandler {
    fn handle<'a, 'b: 'a>(&'b mut self, server: &'a BankServer, src: &'a Peer, mut data: &'a [u8])
                          -> Box<dyn Future<Output=anyhow::Result<Option<Box<dyn BankDataHandler>>>> + Send + Unpin + 'a>
    {
        if data.len() < 1 {
            return Box::new(Box::pin(async {
                Err(anyhow!("Wrong packet length"))
            }));
        }
        let packet_type = data.get_u8();

        let task = async move {
            match packet_type {
                0 if data.len() == 4 => {
                    // deposit
                    let amount = data.get_u32();
                    info!("Deposit {}", amount);
                    if self.user.balance + amount > 10000 {
                        Err(UserInputError::new("超出存款上限"))?
                    }
                    let mut sql_connection = server.0.sql_pool.acquire().await?;
                    let result = query("UPDATE bank_user SET balance=balance+? WHERE id=?")
                        .bind(amount)
                        .bind(self.user.id)
                        .execute(sql_connection.as_mut()).await?;
                    info!("Deposit result: {:?}", result);
                    server.insert_trade_log(self.user.id, "存款", amount as i32).await?;

                    self.user = get_user(&mut sql_connection, self.user.id).await?;
                    let mut data = vec![];
                    data.add_header();
                    // Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
                    data.extend_from_slice(b"menu");
                    data.extend_from_slice(&self.user.id.to_be_bytes());
                    data.write_string(&self.user.name);
                    data.put_u32(self.user.balance);
                    data.write_string(&self.user.phone);
                    src.sender.send(NetworkMessage::Rely(data))?;
                    Ok(None)
                }
                1 if data.len() == 4 => {
                    // Withdraw
                    let amount = data.get_u32();
                    info!("Withdraw {}", amount);
                    if self.user.balance < amount {
                        Err(UserInputError::new("超出存款上限"))?
                    }
                    let mut sql_connection = server.0.sql_pool.acquire().await?;
                    let result = query("UPDATE bank_user SET balance=balance-? WHERE id=?")
                        .bind(amount)
                        .bind(self.user.id)
                        .execute(sql_connection.as_mut()).await?;
                    info!("withdraw result: {:?}", result);
                    server.insert_trade_log(self.user.id, "取款", -(amount as i32)).await?;


                    self.user = get_user(&mut sql_connection, self.user.id).await?;
                    let mut data = vec![];
                    data.add_header();
                    // Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
                    data.extend_from_slice(b"menu");
                    data.extend_from_slice(&self.user.id.to_be_bytes());
                    data.write_string(&self.user.name);
                    data.put_u32(self.user.balance);
                    data.write_string(&self.user.phone);
                    src.sender.send(NetworkMessage::Rely(data))?;
                    Ok(None)
                }
                2 if data.len() == 8 => {
                    let target = data.get_u32();
                    let amount = data.get_u32();
                    let mut sql_connection = server.0.sql_pool.acquire().await?;
                    let target_user = get_user(&mut sql_connection, target).await?;
                    if target_user.balance + amount > 10000 {
                        Err(UserInputError::new("对方存款到达上限"))?
                    }
                    if self.user.balance < amount {
                        Err(UserInputError::new("我方存款不足"))?
                    }
                    let _ = server.add_balance(target, &self.user.id.to_string(), amount).await?;

                    let result = query("UPDATE bank_user SET balance=balance-? WHERE id=?")
                        .bind(amount)
                        .bind(self.user.id)
                        .execute(sql_connection.as_mut()).await?;

                    // Copied code from packet type 1
                    {
                        self.user = get_user(&mut sql_connection, self.user.id).await?;
                        let mut data = vec![];
                        data.add_header();
                        // Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
                        data.extend_from_slice(b"menu");
                        data.extend_from_slice(&self.user.id.to_be_bytes());
                        data.write_string(&self.user.name);
                        data.put_u32(self.user.balance);
                        data.write_string(&self.user.phone);
                        src.sender.send(NetworkMessage::Rely(data))?;
                    }
                    Ok(None)
                }
                3 if data.len() == 0 => {
                    let mut sql_connection = server.0.sql_pool.acquire().await?;
                    let result = query("SELECT * FROM trade_logs WHERE receiver = ? OR sender = ?")
                        .bind(self.user.id)
                        .bind(&format!("{}", self.user.id))
                        .fetch_all(sql_connection.as_mut()).await?;

                    let mut packet_data: Vec<u8> = vec![];
                    packet_data.add_header();
                    packet_data.extend_from_slice(b"info");
                    packet_data.put_u32(1);
                    packet_data.put_u32(1);
                    packet_data.put_u32(result.len() as u32);

                    // user data
                    packet_data.extend_from_slice(&self.user.id.to_be_bytes());
                    packet_data.write_string(&self.user.name);
                    packet_data.put_u32(self.user.balance);
                    packet_data.write_string(&self.user.phone);

                    for row in result {
                        let tid: i32 = row.get("tid");
                        let receiver = row.get::<i32, _>("receiver") as u32;
                        let sender: String = row.get("sender");
                        let time: DateTime<Utc> = row.get("time");
                        let amount: i32 = row.get("amount");

                        packet_data.put_i32(tid);
                        packet_data.put_u32(receiver);
                        packet_data.write_string(&sender);
                        packet_data.put_i64(time.timestamp());
                        packet_data.put_u32(time.timestamp_subsec_nanos());
                        packet_data.put_i32(amount);
                    }
                    src.sender.send(NetworkMessage::Rely(packet_data))?;
                    Ok(None)
                }
                _ => {
                    Err(anyhow!("Wrong packet type in logged state."))
                }
            }
        };
        Box::new(Box::pin(task))
    }
}