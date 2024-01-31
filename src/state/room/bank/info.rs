use chrono::Utc;
use egui::{Button, Color32, Context, Frame, Label, ScrollArea, Sense, Ui, Vec2};
use nalgebra::DimAdd;

use crate::engine::StateData;
use crate::state::room::bank::{BankUi, BankUiRenderArg};
use crate::state::room::bank::index::{Index, User};

pub struct TradeInfo {
    pub tid: i32,
    pub receiver: u32,
    pub sender: String,
    pub time: chrono::DateTime<Utc>,
    pub amount: i32,
}

impl TradeInfo {
    pub fn new(tid: i32, receiver: u32, sender: String, time: chrono::DateTime<Utc>, amount: i32) -> Self {
        Self { tid, receiver, sender, time, amount }
    }

}

pub struct InfoUi {
    pub(crate) user: User,
    info: Vec<TradeInfo>,

}

impl InfoUi {
    pub fn new(user: User, mut info: Vec<TradeInfo>) -> Self {
        info.sort_unstable_by(|a, b| {
            a.time.cmp(&b.time)
        });
        Self { user, info }
    }
}


impl BankUi for InfoUi {
    fn render(&mut self, s: &mut StateData, ctx: &Context, args: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>> {
        let mut ret = None;
        egui::CentralPanel::default().frame(Frame::default().fill(Color32::BLACK)).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // 1600 900
                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let back = Button::new("返回").min_size(size);
                ui.vertical_centered(|ui| {
                    ui.heading("交易流水记录");
                    let max = ui.max_rect().height();
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.max_rect().width(), max - size.y),
                                                           Sense::click_and_drag());
                    ui.allocate_ui_at_rect(rect, |ui| {
                        ScrollArea::vertical()
                            .show(ui, |ui| {
                                let w = ui.max_rect().width();
                                let item_w = w / 5.0;
                                let add_ui = |ui: &mut Ui, widget: Label| {
                                    let result = ui.add(widget);
                                    let mut width_left = item_w - result.rect.width();
                                    if width_left > 0.0 {
                                        ui.add_space(width_left);
                                    }
                                    result
                                };

                                ui.horizontal(|ui| {
                                    add_ui(ui, Label::new("交易编号"));
                                    add_ui(ui, Label::new("接收方"));
                                    add_ui(ui, Label::new("来源"));
                                    add_ui(ui, Label::new("时间"));
                                    add_ui(ui, Label::new("交易额"));
                                });
                                for info in &self.info {
                                    ui.horizontal(|ui| {
                                        add_ui(ui, Label::new(info.tid.to_string()));
                                        add_ui(ui, Label::new(info.receiver.to_string()));
                                        add_ui(ui, Label::new(&info.sender));
                                        add_ui(ui, Label::new(&info.time.to_string()));
                                        add_ui(ui, Label::new(info.amount.to_string()));
                                    });
                                }
                            });
                    });
                    if ui.add_sized(size, back).clicked() {
                        ret = Some(Box::new(Index {
                            user: self.user.clone()
                        }) as _);
                    }
                });
            });
        });
        ret
    }
}