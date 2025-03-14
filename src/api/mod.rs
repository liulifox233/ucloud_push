pub mod telegram;
pub mod ticktick;

use anyhow::Result;

use crate::model::UndoneList;

pub trait Api {
    #[allow(async_fn_in_trait)]
    async fn push(&self, message: &UndoneList) -> Result<()>;
}
