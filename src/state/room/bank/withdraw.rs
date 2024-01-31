use std::str::FromStr;

use bytes::BufMut;
use egui::{Button, Color32, Context, Frame, Vec2};
use msgbox::IconType;

use crate::engine::network::NetworkMessage;
use crate::engine::StateData;
use crate::ext::PacketWriteExt;
use crate::state::room::bank::{BankUi, BankUiRenderArg};
use crate::state::room::bank::index::{Index, User};

pub struct Withdraw {
    pub(crate) user: User,
    pub(crate) amount: String,

}

impl Withdraw {
    pub fn new(user: User) -> Self {
        Self { user, amount: Default::default() }
    }
}


impl BankUi for Withdraw {
    fn render(&mut self, s: &mut StateData, ctx: &Context, args: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default().fill(Color32::BLACK)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900
                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let withdraw = Button::new("取款").min_size(size);
                let back = Button::new("返回").min_size(size);
                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y * 0.5);
                    ui.label("数量：");
                    ui.text_edit_singleline(&mut self.amount);
                    ui.label("");
                    if ui.add_sized(size, withdraw).clicked() {
                        let amount = match u32::from_str(&self.amount) {
                            Ok(id) => {
                                id
                            }
                            Err(_) => {
                                msgbox::create("错误", "需要为数字", IconType::Error).expect("panic!");
                                return;
                            }
                        };
                        if amount > self.user.balance {
                            msgbox::create("错误", "你没钱还想取款😓", IconType::Error).expect("panic!");
                            return;
                        }
                        if amount > 0 {
                            let mut data = Vec::<u8>::new();
                            data.add_header();
                            data.put_u8(1);
                            data.put_u32(amount);
                            let peer = args.target;
                            peer.sender.send(NetworkMessage::Rely(data)).expect("how send error");
                        }
                    }
                    if ui.add_sized(size, back).clicked() {
                        ret = Some(Box::new(Index {
                            user: self.user.clone(),
                        }) as _);
                    }
                });
            });
        });
        ret
    }
}