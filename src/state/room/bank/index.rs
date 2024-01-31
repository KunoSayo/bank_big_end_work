use bytes::BufMut;
use egui::{Button, Color32, Context, Frame, Vec2};

use crate::engine::network::NetworkMessage;
use crate::engine::StateData;
use crate::ext::PacketWriteExt;
use crate::state::room::bank::{BankUi, BankUiRenderArg};
use crate::state::room::bank::deposit::Deposit;
use crate::state::room::bank::transfer::Transfer;
use crate::state::room::bank::withdraw::Withdraw;

#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub balance: u32,
    pub name: String,
    pub phone: String,
}

pub struct Index {
    pub(crate) user: User,
}


impl BankUi for Index {
    fn render(&mut self, s: &mut StateData, ctx: &Context, args: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default().fill(Color32::BLACK)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900

                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let deposit = Button::new("存款").min_size(size);
                let withdraw = Button::new("取款").min_size(size);
                let transfer = Button::new("转账").min_size(size);
                let log = Button::new("记录").min_size(size);
                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y * 2.0);
                    ui.label(format!("账号: {}，余额：{}.{}，姓名：{}，联系电话：{}",
                                     self.user.id, self.user.balance / 100, self.user.balance % 100, self.user.name, self.user.phone));

                    if ui.add_sized(size, deposit).clicked() {
                        ret = Some(Box::new(Deposit::new(self.user.clone())) as Box<dyn BankUi>);
                    }
                    if ui.add_sized(size, withdraw).clicked() {
                        ret = Some(Box::new(Withdraw::new(self.user.clone())) as Box<dyn BankUi>);

                    }
                    if ui.add_sized(size, transfer).clicked() {
                        ret = Some(Box::new(Transfer::new(self.user.clone())) as Box<dyn BankUi>);
                    }
                    if ui.add_sized(size, log).clicked() {
                        let mut data = Vec::<u8>::new();
                        data.add_header();
                        data.put_u8(3);
                        let peer = args.target;
                        peer.sender.send(NetworkMessage::Rely(data)).expect("how send error");
                    }
                });
            });
        });
        ret
    }
}