use std::hash::{Hash, Hasher, SipHasher};
use std::str::FromStr;

use bytes::BufMut;
use egui::{Button, Color32, Context, Frame, TextEdit, Vec2};
use msgbox::IconType;

use crate::engine::network::NetworkMessage;
use crate::engine::StateData;
use crate::ext::PacketWriteExt;
use crate::state::room::bank::{BankUi, BankUiRenderArg};

#[derive(Default)]
pub struct BankMenu {}

#[derive(Default)]
pub struct Login {
    id: String,
    password: String,
}

#[derive(Default)]
pub struct Register {
    id: String,
    password: String,
    name: String,
    phone: String,
}

impl BankUi for BankMenu {
    fn render(&mut self, s: &mut StateData, ctx: &Context, _: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default()).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900
                //
                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let login = Button::new("登录").min_size(size);
                let reg = Button::new("注册").min_size(size);

                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y);

                    if ui.add_sized(size, login).clicked() {
                        ret = Some(Box::new(Login::default()) as _);
                    }
                    if ui.add_sized(size, reg).clicked() {
                        ret = Some(Box::new(Register::default()) as _);
                    }
                });
            });
        });
        ret
    }
}

impl BankUi for Login {
    fn render(&mut self, s: &mut StateData, ctx: &Context, arg: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default().fill(Color32::BLACK)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900
                //
                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let login = Button::new("登录").min_size(size);
                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y * 0.5);
                    ui.label("账号：");
                    ui.text_edit_singleline(&mut self.id);
                    ui.label("密码：");
                    ui.add(TextEdit::singleline(&mut self.password).password(true));
                    ui.label("");

                    if ui.add_sized(size, login).clicked() {
                        if self.id.is_empty() || self.password.is_empty() {
                            msgbox::create("错误", "账号密码不能为空", IconType::Error).expect("panic!");
                            return;
                        }
                        let mut data = Vec::<u8>::new();
                        data.add_header();
                        // format:
                        // Register packet: (id: u32) (password: u32) (name: String) (phone_number: String)

                        let id = match u64::from_str(&self.id) {
                            Ok(id) => {
                                id
                            }
                            Err(_) => {
                                msgbox::create("错误", "账号需要为数字", IconType::Error).expect("panic!");
                                return;
                            }
                        };

                        let mut hasher = SipHasher::new_with_keys(233, 9961);
                        self.password.hash(&mut hasher);
                        let pswd = hasher.finish();

                        data.put_u32(id as u32);
                        data.put_u32(pswd as u32);
                        let peer = arg.target;
                        peer.sender.send(NetworkMessage::Rely(data)).expect("how send error");
                    }
                });
            });
        });
        ret
    }
}

impl BankUi for Register {
    fn render(&mut self, s: &mut StateData, ctx: &Context, arg: BankUiRenderArg) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default().fill(Color32::BLACK)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900

                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let login = Button::new("注册").min_size(size);
                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y * 0.5 - size.y);
                    ui.label("账号：");

                    ui.text_edit_singleline(&mut self.id);
                    ui.label("密码：");
                    ui.add(TextEdit::singleline(&mut self.password).password(true));
                    ui.label("姓名：");
                    ui.text_edit_singleline(&mut self.name);
                    ui.label("手机号：");
                    ui.text_edit_singleline(&mut self.phone);
                    ui.label("");
                    if ui.add_sized(size, login).clicked() {
                        if self.id.is_empty() || self.password.is_empty() {
                            msgbox::create("错误", "账号密码不能为空", IconType::Error).expect("panic!");
                            return;
                        }
                        let mut data = Vec::<u8>::new();
                        data.add_header();
                        // format:
                        // Register packet: (id: u32) (password: u32) (name: String) (phone_number: String)

                        let id = match u64::from_str(&self.id) {
                            Ok(id) => {
                                id
                            }
                            Err(e) => {
                                msgbox::create("错误", "账号需要为数字", IconType::Error).expect("panic!");
                                return;
                            }
                        };

                        let mut hasher = SipHasher::new_with_keys(233, 9961);
                        self.password.hash(&mut hasher);
                        let pswd = hasher.finish();

                        data.put_u32(id as u32);
                        data.put_u32(pswd as u32);
                        data.write_string(&self.name);
                        data.write_string(&self.phone);

                        let peer = arg.target;
                        peer.sender.send(NetworkMessage::Rely(data)).expect("how send error");
                    }
                });
            });
        });
        ret
    }
}