pub mod api;
pub mod db;
pub mod model;
pub mod ucloud;

use api::Api;
use tracing::info;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

#[event(start)]
fn start() {
    console_error_panic_hook::set_once();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

#[event(fetch)]
async fn fetch(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Response::ok("Hello World")
}

#[event(scheduled)]
async fn scheduled(
    _event: worker::ScheduledEvent,
    env: worker::Env,
    _ctx: worker::ScheduleContext,
) {
    let ucloud = ucloud::UCloud::new(
        env.secret("USERNAME").unwrap().to_string(),
        env.secret("PASSWORD").unwrap().to_string(),
        env.secret("API_URL").unwrap().to_string(),
    );
    let db = env.d1("DB").unwrap();

    let undone_list = ucloud.get_undone_list().await.unwrap();
    info!("undone_list: {:?}", undone_list);

    let unpushed_list = db::filter_pushed_undone_list(&undone_list, &db)
        .await
        .unwrap();

    // push to telegram
    let bot = api::telegram::Telegram::new(
        env.secret("TELEGRAM_TOKEN").unwrap().to_string(),
        env.secret("TELEGRAM_CHAT_ID").unwrap().to_string(),
    );
    bot.push(&unpushed_list).await.unwrap();

    // save to database
    db::save_activities_batch(&unpushed_list.undone_list, &db)
        .await
        .unwrap();
}
