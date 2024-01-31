#![allow(unused)]

use std::ops::Deref;
use std::sync::atomic::Ordering;

use egui::{Color32, Key, ScrollArea, TextBuffer, TopBottomPanel, Ui};
use egui::{Context, Frame, Slider, Vec2};
use egui::WidgetType::SelectableLabel;

use crate::engine::{GameState, LoopState, StateData, Trans};

#[derive(Default)]
pub struct TestState {
    your_data_here: String,
    message: Vec<String>,
}

impl GameState for TestState {
    fn update(&mut self, _: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::WAIT)
    }

    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                ui.heading("尽情测试！");
                ScrollArea::vertical()
                    .max_width(ui.available_width())
                    .max_height(ui.available_height() - 200.0)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        for item in &self.message {
                            ui.label(item);
                        }
                    });


                TopBottomPanel::bottom("text_edit_blank")
                    .resizable(false)
                    .min_height(100.0)
                    .max_height(ui.available_height())
                    .show_inside(ui, |ui| {
                        let mut edit = egui::TextEdit::multiline(&mut self.your_data_here);
                        if ui.add_sized(
                            ([ui.available_width(), 80.0]),
                            edit,
                        ).has_focus() {
                            s.app.window.set_ime_allowed(true);
                        } else {
                            s.app.window.set_ime_allowed(false);
                        }
                        if ui.button("send").clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
                            if !&self.your_data_here.is_empty() {
                                self.message.push(self.your_data_here.take());
                            }
                        }
                    });
            });

        Trans::None
    }
}
