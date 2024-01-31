use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::atomic::Ordering;

use anyhow::anyhow;
use bytes::Buf;
use chrono::DateTime;
use egui::Context;
use log::info;
use msgbox::IconType;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use winit::event::VirtualKeyCode;

use crate::engine::{GameState, LoopState, StateData, Trans};
use crate::engine::network::peer::Peer;
use crate::engine::window::EventLoopMessage;
use crate::ext::{CURRENT_VERSION, PacketReadExt};
use crate::state::room::{bank, ReceiverType};
use crate::state::room::bank::{BankUi, BankUiRenderArg};
use crate::state::room::bank::index::{Index, User};
use crate::state::room::bank::info::{InfoUi, TradeInfo};
use crate::state::room::client::Client;

pub struct ConnectingState {
    /// The tokio runtime to run tasks
    pub(crate) rt: Runtime,
    /// Connecting to the host server peer
    pub(crate) target: Peer,
    bank: Box<dyn BankUi>,
    change_ui: UnboundedReceiver<Box<dyn BankUi>>,
}

// build runtime and new host state and then new peer

impl ConnectingState {
    pub async fn create(addr: String, is_join: bool) -> anyhow::Result<Self> {
        let rt = Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()?;
        let addr = rt.spawn(async move {
            tokio::net::lookup_host(&addr).await?.next().ok_or(anyhow!("Get ip failed"))
        }).await??;


        info!("Listening on {}", addr);

        let connect_ip = if is_join {
            addr
        } else {
            match addr {
                SocketAddr::V4(addr) => {
                    SocketAddr::from(SocketAddrV4::new(Ipv4Addr::LOCALHOST, addr.port()))
                }
                SocketAddr::V6(addr) => {
                    SocketAddr::from(SocketAddrV6::new(Ipv6Addr::LOCALHOST, addr.port(),
                                                       addr.flowinfo(), addr.scope_id()))
                }
            }
        };
        let client = rt.spawn(Client::new(connect_ip)).await??;

        let (tx, rx) = unbounded_channel();
        let this = Self {
            rt,
            target: client.target,
            bank: Box::new(bank::menu::BankMenu::default()),
            change_ui: rx,
        };

        this.get_msg(client.receiver, tx);
        Ok(this)
    }
}

impl GameState for ConnectingState {
    fn start(&mut self, s: &mut StateData) {
        let _ = s.wd.elp.send_event(EventLoopMessage::WakeUp(s.app.window.id()));
    }

    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        while let Ok(ui) = self.change_ui.try_recv() {
            self.bank = ui;
        }
        if s.app.inputs.is_pressed(&[VirtualKeyCode::Escape]) || !self.target.listening.load(Ordering::Relaxed) {
            (Trans::Pop, LoopState::WAIT)
        } else {
            (Trans::None, LoopState::WAIT)
        }
    }


    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        let tran = Trans::None;
        let ret = self.bank.render(s, ctx, BankUiRenderArg {
            rt: &self.rt,
            target: &self.target,
        });
        if let Some(ret) = ret {
            self.bank = ret;
            s.app.window.request_redraw();
        }
        tran
    }
}

impl ConnectingState {
    fn get_msg(&self, mut receiver: ReceiverType, sender: UnboundedSender<Box<dyn BankUi>>) {
        self.rt.spawn(async move {
            while let Some((_, data)) = receiver.recv().await {
                if data.len() < 12 || &data[0..4] != b"rPtm" {
                    continue;
                }
                let version = (&data[4..8]).get_u32();

                if version != CURRENT_VERSION {
                    continue;
                }

                let r#type = &data[8..12];
                let mut data = &data[12..];
                match r#type {
                    b"msgb" => {
                        let msg = data.read_packet_string().unwrap();
                        msgbox::create("Tip!", &msg, IconType::Info).unwrap();
                    }
                    b"menu" => {
                        info!("Menu packet!");
                        let id = data.get_u32();
                        let name = data.read_packet_string().unwrap();
                        let balance = data.get_u32();
                        let phone = data.read_packet_string().unwrap();
                        let _ = sender.send(Box::new(Index {
                            user: User {
                                id,
                                balance,
                                name,
                                phone,
                            },
                        }));
                    }
                    b"info" => {
                        info!("Info packet!");
                        let _cur_page = data.get_u32();
                        let _total_page = data.get_u32();
                        let info_count = data.get_u32();

                        let id = data.get_u32();
                        let name = data.read_packet_string().unwrap();
                        let balance = data.get_u32();
                        let phone = data.read_packet_string().unwrap();

                        let mut info = vec![];
                        info!("Got info count: {}", info_count);
                        for _ in 0..info_count {
                            let tid = data.get_i32();
                            let receiver = data.get_u32();
                            let sender = data.read_packet_string().unwrap();
                            let secs = data.get_i64();
                            let nanos = data.get_u32();
                            let date_time = DateTime::from_timestamp(secs, nanos)
                                .unwrap();
                            let amount = data.get_i32();

                            info.push(TradeInfo::new(tid, receiver, sender, date_time, amount));
                        }
                        let _ = sender.send(Box::new(InfoUi::new(User {
                            id,
                            balance,
                            name,
                            phone,
                        }, info)) as _);
                    }
                    _ => {
                        info!("Receive unknown packet: {:?}", r#type);
                    }
                }
            }
        });
    }
}