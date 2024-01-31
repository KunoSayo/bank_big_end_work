use egui::{Button, Color32, Context, Frame, RichText};
use futures::task::SpawnExt;

use crate::engine::{GameState, LoopState, StateData, Trans, WaitFutureState, WaitResult};
use crate::engine::global::IO_POOL;
use crate::engine::window::EventLoopMessage;
use crate::state::room::connecting::ConnectingState;

pub struct JoiningRoom {
    addr: String,
    is_join: bool,
}

impl Default for JoiningRoom {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:5555".to_string(),
            is_join: false,
        }
    }
}


impl JoiningRoom {
    pub fn join() -> Self {
        Self {
            addr: "[::1]:1234".to_string(),
            is_join: true,
        }
    }
}


impl GameState for JoiningRoom {
    fn start(&mut self, s: &mut StateData) {
        let _ = s.wd.elp.send_event(EventLoopMessage::WakeUp(s.app.window.id()));
    }


    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::WAIT)
    }

    fn render(&mut self, _: &mut StateData, ctx: &Context) -> Trans {
        let mut tran = Trans::None;
        egui::CentralPanel::default().frame(Frame::none()).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 4.0);
                let style = ui.style_mut();
                style.visuals.extreme_bg_color = Color32::from_gray(64);

                ui.heading(["监听IP地址:", "加入IP地址:"][self.is_join as usize]);
                ui.text_edit_singleline(&mut self.addr);

                ui.add_space((ui.available_height() / 4.0).min(100.0));

                ui.style_mut().spacing.button_padding *= 8.0;

                let create = Button::new(RichText::new(["创建", "加入"][self.is_join as usize]).heading()).fill(Color32::from_rgb(128, 32, 32));
                let ret = Button::new(RichText::new("返回").heading());
                if ui.add(create).clicked() {
                    let addr = self.addr.clone();
                    let is_join = self.is_join;
                    match IO_POOL.spawn_with_handle(async move {
                        let result = ConnectingState::create(addr, is_join).await;
                        match result {
                            Ok(ok) => {
                                WaitResult::Switch(Box::new(ok))
                            }
                            Err(e) => {
                                log::warn!("Create udp socket failed for {:?}", e);
                                WaitResult::Pop
                            }
                        }
                    }) {
                        Ok(r) => {
                            tran = Trans::Push(WaitFutureState::from_wait_thing(r));
                        }
                        Err(e) => {
                            log::error!("Spawn io task failed for {:?}", e);
                        }
                    }
                }
                if ui.add(ret).clicked() {
                    tran = Trans::Pop;
                }
            });
        });
        tran
    }
}