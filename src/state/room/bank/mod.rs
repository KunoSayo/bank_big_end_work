use egui::Context;
use tokio::runtime::Runtime;

use crate::engine::network::peer::Peer;
use crate::engine::StateData;

pub(crate) mod menu;
pub(crate) mod index;
mod transfer;
mod withdraw;
mod deposit;
pub(super) mod info;

pub struct BankUiRenderArg<'a> {
    pub(crate) rt: &'a Runtime,
    pub(crate) target: &'a Peer,
}

pub trait BankUi: Send {
    fn render(&mut self, _: &mut StateData, ctx: &Context, args: BankUiRenderArg<'_>) -> Option<Box<dyn BankUi>>;
}

