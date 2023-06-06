use std::env;

use api::LiveData;

use api::Vehicle;
use flexi_logger::FileSpec;
use serenity::async_trait;

use serenity::builder::CreateEmbed;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::GuildChannel;
use serenity::prelude::*;
use serenity::utils::Colour;
use serenity::Client;

mod api;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);

        // ready.ctx.set_activity(Activity::playing("with fire")).await;
        let http = ctx.http.clone();
        for g in ready.guilds {
            let channels = g.id.channels(http.clone()).await.unwrap();
            let live_racers = channels
                .get(&ChannelId(
                    std::env::var("DISCORD_CHANNEL_ID")
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                ))
                .unwrap();

            tokio::spawn(update_live_racers(ctx.clone(), live_racers.clone()));
        }
    }
}

async fn update_live_racers(ctx: Context, chan: GuildChannel) -> ! {
    chan.delete_messages(
        ctx.http.clone(),
        chan.messages(ctx.http.clone(), |b| b).await.unwrap(),
    )
    .await
    .unwrap();

    let mut msg = chan
        .send_message(ctx.http.clone(), |m| {
            m.embed(|e| {
                e.title("Starting up...");
                e
            });
            m
        })
        .await
        .unwrap();

    let rf2la_url = std::env::var("RF2LA_URL").unwrap();

    let mut last_data = vec![];

    // let fastest_lap = None;

    loop {
        let live_data: LiveData = reqwest::get(format!("{}/live/get_data", rf2la_url,))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let mut servers = vec![];

        for server in live_data.server_list {
            let server_live_data: LiveData =
                reqwest::get(format!("{}/live/get_data?name={}", rf2la_url, server.name))
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
            servers.push(server_live_data);
        }

        if servers != last_data {
            log::info!("Updating live racers: {:?}", servers);

            msg.edit(ctx.http.clone(), |m| {
                m.content("");
                let mut embeds = vec![];
                for server in servers.iter() {
                    let (players, ai): (Vec<_>, Vec<_>) =
                        server.m_vehicles.iter().partition(|v| v.m_control == 2);

                    let server_color = u32::from_str_radix(
                        &std::env::var(format!("{}_COLOUR", server.m_scoring_info.m_server_name))
                            .unwrap_or("000000".to_string()),
                        16,
                    )
                    .unwrap();

                    let (red, green, blue) = (
                        (server_color >> 16) as u8,
                        (server_color >> 8) as u8,
                        server_color as u8,
                    );

                    let server_icon =
                        env::var(format!("{}_ICON", server.m_scoring_info.m_server_name))
                            .unwrap_or(server.m_scoring_info.m_server_name.clone());

                    embeds.push(
                        CreateEmbed::default()
                            .colour(Colour::from_rgb(red, green, blue))
                            .title(server_icon)
                            .field("Track", server.m_scoring_info.m_track_name.clone(), false)
                            .field("Drivers", format_drivers(&players), false)
                            // .field("AI", format_drivers(&ai), false)
                            .field(
                                "Track temperature",
                                format_temp(server.m_scoring_info.m_track_temp),
                                true,
                            )
                            .field(
                                "Ambient",
                                format_temp(server.m_scoring_info.m_ambient_temp),
                                true,
                            )
                            .field(
                                "Elapsed / End time",
                                format!(
                                    "{} / {}",
                                    format_minutes_time(server.m_scoring_info.m_current_et),
                                    format_minutes_time(server.m_scoring_info.m_end_et)
                                ),
                                false,
                            )
                            .to_owned(),
                    );
                }

                m.set_embeds(embeds);
                m
            })
            .await
            .unwrap();
        }

        last_data = servers.clone();

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn format_temp(temp: f32) -> String {
    format!("{:.2}°C / {:.0}°F", temp, temp * 1.8 + 32.0)
}

fn format_drivers(vec: &Vec<&Vehicle>) -> String {
    if vec.is_empty() {
        return "None".to_owned();
    }

    let mut vec = vec.clone();
    vec.sort_by(|a, b| a.m_place.cmp(&b.m_place));

    // let fastest_lap = vec
    //     .iter()
    //     .filter(|v| v.m_best_lap_time > 0.0)
    //     .min_by(|a, b| a.m_best_lap_time.partial_cmp(&b.m_best_lap_time).unwrap())
    //     .map(|v| v.m_best_lap_time)
    //     .unwrap_or(0.0);

    vec.iter()
        .map(|v| {
            format!(
                "* {}\n  * Laps: {}\n  * Last Lap: {}\n  * Fastest Lap: {}",
                v.m_driver_name.clone(),
                v.m_total_laps,
                format_laptime(v.m_last_lap_time),
                format_laptime(v.m_best_lap_time)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// time in minutes f32 (595.4 minutes) to mm:ss
fn format_minutes_time(time: f32) -> String {
    if time <= 0.0 {
        return "--:--".to_owned();
    }

    let minutes = (time / 60.0).floor();
    let seconds = (time - minutes * 60.0).floor();

    format!("{}:{:02}", minutes, seconds)
}

fn format_laptime(seconds: f32) -> String {
    if seconds <= 0.0 {
        return "--:--.---".to_owned();
    }

    let minutes = (seconds / 60.0) as u32;
    let remaining_seconds = seconds % 60.0;
    let formatted_seconds = format!("{:06.3}", remaining_seconds);

    format!(
        "{}:{:02}.{}",
        minutes,
        remaining_seconds as u32,
        formatted_seconds.split('.').next().unwrap()
    )
}

#[tokio::main]
async fn main() {
    let handle = flexi_logger::Logger::try_with_str("warn,discord_bot=info")
        .unwrap()
        .format(flexi_logger::colored_default_format)
        .log_to_stdout()
        // .log_to_file(FileSpec::default().basename("discord-bot"))
        // .write_mode(flexi_logger::WriteMode::Direct)
        .start()
        .unwrap();
    log_panics::init();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::empty() | GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why);
    }

    drop(handle);
}
