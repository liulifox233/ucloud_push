pub mod api;
pub mod d1;
pub mod model;
pub mod ucloud;

use api::Api;
use tracing::{error, info};
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
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if req.method() != Method::Get {
        return Response::error("Method Not Allowed", 405);
    }

    let kv = env.kv("KV").unwrap();

    match req
        .url()?
        .path_segments()
        .unwrap()
        .collect::<Vec<_>>()
        .first()
    {
        Some(&"ping") => Response::ok("pong"),
        Some(&"push") => {
            push(env).await?;
            Response::ok("Success")
        }
        Some(&"auth") => {
            let ticktick = api::ticktick::TickTick::new(
                env.secret("TICKTICK_CLIENT_ID").unwrap().to_string(),
                env.secret("TICKTICK_CLIENT_SECRET").unwrap().to_string(),
                kv.clone(),
            )
            .await;
            let url = req.url().unwrap();

            ticktick
                .auth(url, &env.secret("REDIRECT_URI").unwrap().to_string(), kv)
                .await
                .unwrap();
            Response::ok("Success")
        }
        Some(&"refresh") => {
            let ticktick = api::ticktick::TickTick::new(
                env.secret("TICKTICK_CLIENT_ID").unwrap().to_string(),
                env.secret("TICKTICK_CLIENT_SECRET").unwrap().to_string(),
                kv.clone(),
            )
            .await;
            let bot = api::telegram::Telegram::new(
                env.secret("TELEGRAM_TOKEN").unwrap().to_string(),
                env.secret("TELEGRAM_CHAT_ID").unwrap().to_string(),
            );
            let redirect_uri = env.secret("REDIRECT_URI").unwrap().to_string();
            ticktick.login(&bot, &redirect_uri, kv).await.unwrap();

            Response::ok("Success")
        }
        _ => Response::error("Not Found", 404),
    }
}

#[event(scheduled)]
async fn scheduled(
    _event: worker::ScheduledEvent,
    env: worker::Env,
    _ctx: worker::ScheduleContext,
) {
    if let Err(e) = push(env).await {
        error!("push error: {:?}", e);
    }
}

async fn push(env: worker::Env) -> Result<()> {
    let ucloud = ucloud::UCloud::new(
        env.secret("USERNAME").unwrap().to_string(),
        env.secret("PASSWORD").unwrap().to_string(),
        env.secret("API_URL").unwrap().to_string(),
    );
    let db = env.d1("DB").unwrap();
    let kv = env.kv("KV").unwrap();

    let undone_list = ucloud.get_undone_list().await.unwrap();
    info!("undone_list: {:?}", undone_list);

    let unpushed_list = d1::filter_pushed_undone_list(&undone_list, &db)
        .await
        .unwrap();

    // push to telegram
    let bot = api::telegram::Telegram::new(
        env.secret("TELEGRAM_TOKEN").unwrap().to_string(),
        env.secret("TELEGRAM_CHAT_ID").unwrap().to_string(),
    );
    bot.push(&unpushed_list).await.unwrap();

    // push to ticktick
    let ticktick = api::ticktick::TickTick::new(
        env.secret("TICKTICK_CLIENT_ID").unwrap().to_string(),
        env.secret("TICKTICK_CLIENT_SECRET").unwrap().to_string(),
        kv.clone(),
    )
    .await;
    if ticktick.access_token.is_none() {
        ticktick
            .login(&bot, &env.secret("REDIRECT_URI").unwrap().to_string(), kv)
            .await
            .unwrap();
        info!("Sent login link to telegram");
    } else {
        ticktick.push(&unpushed_list).await.unwrap();
    }

    // save to database
    d1::save_activities_batch(&unpushed_list.undone_list, &db)
        .await
        .unwrap();

    Ok(())
}
