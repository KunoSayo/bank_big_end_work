use egui::{Button, Context, Frame, Vec2};
use winit::window::WindowId;

use crate::engine::{GameState, LoopState, StateData, StateEvent, Trans};
use crate::engine::invert_color::InvertColorRenderer;
use crate::engine::point::PointRenderer;
use crate::state::room::JoiningRoom;

pub struct MainMenu {
    option_window: Option<WindowId>,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self { option_window: None }
    }
}


impl GameState for MainMenu {
    fn start(&mut self, _s: &mut StateData) {}

    fn update(&mut self, _s: &mut StateData) -> (Trans, LoopState) {
        (Trans::None, LoopState::WAIT)
    }


    fn render(&mut self, s: &mut StateData, ctx: &Context) -> Trans {
        let mut ret = Trans::None;
        egui::CentralPanel::default()
            .frame(Frame::default())
            .show(ctx, |ui| {
                // 1600 900
                //
                let scale = s.app.gpu.as_ref().unwrap().size_scale;
                let size = Vec2::new(320.0, 180.0) * Vec2::from(scale);
                let join = Button::new("加入银行").min_size(size);

                ui.vertical_centered(|ui| {
                    let max = ui.max_rect().height();
                    ui.add_space(max * 0.5 - size.y * 0.5);

                    if ui.add_sized(size, join).clicked() {
                        ret = Trans::Push(Box::new(JoiningRoom::join()))
                    }
                });
            });

        ret
    }

    fn shadow_render(&mut self, _: &mut StateData, _ctx: &Context) {}


    fn on_event(&mut self, s: &mut StateData, e: StateEvent) {
        if matches!(e, StateEvent::ReloadGPU) {
            if !s.app.world.has_value::<InvertColorRenderer>() {
                if let Some(gpu) = &s.app.gpu {
                    s.app.world.insert(InvertColorRenderer::new(gpu));
                }
            }
            if !s.app.world.has_value::<PointRenderer>() {
                if let Some(gpu) = &s.app.gpu {
                    s.app.world.insert(PointRenderer::new(gpu));
                }
            }
        }
    }
}
