use std::sync::Arc;
use std::sync::atomic::Ordering;

use futures::task::SpawnExt;
use log::error;
use toml_edit::{Item, Value};
use wgpu::{Device, Queue};

use crate::config::consts::USER_NAME_KEY;
use crate::engine::{GameState, LoopState, ResourceManager, StateData, StateEvent, Trans, WaitFutureState, WaitResult};
use crate::engine::global::{GLOBAL_DATA, INITED, IO_POOL};

pub struct InitState {
    start_state: Option<Box<dyn GameState + Send + 'static>>,
}

impl InitState {
    pub fn new(state: Box<dyn GameState + Send + 'static>) -> Self {
        Self {
            start_state: Some(state),
        }
    }
}

async fn load_texture(_a_d: Arc<Device>, _a_q: Arc<Queue>, _a_r: Arc<ResourceManager>) -> anyhow::Result<()> {
    anyhow::Ok(())
}


impl GameState for InitState {
    fn update(&mut self, s: &mut StateData) -> (Trans, LoopState) {
        if let Some(gpu) = s.app.gpu.as_ref() {
            let state = self.start_state.take().unwrap();
            let device = gpu.device.clone();
            let queue = gpu.queue.clone();
            let res = s.app.res.clone();
            let handle = IO_POOL.spawn_with_handle(async move {
                let device = device;
                let queue = queue;
                let res = res;
                let task = async move {
                    let handle = if !INITED.load(Ordering::Acquire) {
                        let handle = IO_POOL.spawn_with_handle(async {
                            let mut cfg = GLOBAL_DATA.cfg_data.write().unwrap();
                            cfg.toml_mut().entry(USER_NAME_KEY).or_insert(Item::Value(Value::from("guest")));
                            if cfg.is_dirty() {
                                std::fs::write("cfg.toml", cfg.toml().to_string())?;
                            }
                            anyhow::Ok(())
                        });
                        Some(handle)
                    } else {
                        None
                    };
                    if let Some(Ok(handle)) = handle {
                        handle.await?;
                    }

                    load_texture(device, queue, res).await?;

                    anyhow::Ok(())
                };
                if let Err(e) = task.await {
                    error!("Load failed for {:?}", e);
                    WaitResult::Exit
                } else {
                    WaitResult::Function(Box::new(|s| {
                        s.app.egui_ctx.set_fonts(GLOBAL_DATA.font.clone());
                        Trans::Switch(state)
                    }))
                }
            }).expect("Spawn init task failed");


            (Trans::Push(WaitFutureState::from_wait_thing(handle)), LoopState::POLL_WITHOUT_RENDER)
        } else {
            (Trans::None, LoopState::WAIT_ALL)
        }
    }

    fn on_event(&mut self, s: &mut StateData, e: StateEvent) {
        if matches!(e, StateEvent::ReloadGPU) {
            let gpu = s.app.gpu.as_ref().expect("I FOUND GPU");
            println!("block on loading");
            futures::executor::block_on(load_texture(gpu.device.clone(), gpu.queue.clone(), s.app.res.clone()))
                .expect("Load texture failed");
            println!("block end");
        }
    }
}
