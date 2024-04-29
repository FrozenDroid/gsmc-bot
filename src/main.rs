use std::env;

use anyhow::Context as _;

use api::Vehicle;
use serenity::async_trait;

use serenity::builder::CreateEmbed;
use serenity::model::gateway::Gateway;
use serenity::model::guild::Member;
use serenity::model::prelude::ChannelId;
use serenity::model::prelude::GuildChannel;
use serenity::prelude::*;
use serenity::utils::Colour;
use serenity::Client;
use tokio::time::sleep;

use crate::utils::format_channel;

mod api;
mod utils;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: serenity::model::gateway::Ready) {
        log::info!("{} is connected!", ready.user.name);

        let http = ctx.http.clone();
        for g in ready.guilds {
            log::info!(
                "Connected to guild \"{}\" ({})",
                g.id.name(&ctx).unwrap_or("unknown".to_string()),
                g.id
            );
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

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        // I tried using an environment variable for this, but Batch is giving me an anheurism
        if let Err(e) = new_member.user.direct_message(&ctx, |m| {
            m.content(r#"
Welcome!
**We currently have 2 leagues running:**

**The GSMC - Downforce Cup:** this is a series running over 8 races for the course of a few months using the **Formula Pro** cars. Each race is 40 minutes. The next race is on 21st April, the signup and info for the season is available here: https://discord.com/channels/692059034064519268/1053702897734520832

**The GSME:** this is an endurance series that runs 1 race a month, over the whole year. We are using **GTE** cars. Each race is 1hr long, with special events being longer with driver swaps. The next race is on the 28th April, the signup and info for the season is available here: https://discord.com/channels/692059034064519268/1053702923118448691

Also, scheduling for all the events can be found here: https://discord.com/channels/692059034064519268/1200104236231434411, and in the discord "Events" list!

Let an admin know if you have any questions at all!
            "#)
        }).await {
            log::warn!("Could not send DM to user {}: {}", new_member.user.name, e);
        }
    }
}

async fn update_live_racers(ctx: Context, chan: GuildChannel) -> ! {
    clean_channel(&chan, &ctx).await;

    let mut msg = chan
        .send_message(ctx.http.clone(), |m| {
            m.embed(|e| {
                e.title("Starting up...");
                e
            });
            m
        })
        .await
        .with_context(|| {
            format!(
                "Error sending message in channel {}",
                utils::format_channel(&chan)
            )
        })
        .unwrap();

    let mut last_servers_data = vec![];

    loop {
        let general_data = match api::get_live_data(None).await {
            Ok(live_data) => live_data,
            Err(e) => {
                log::error!("Error getting live data: {}", e);
                continue;
            }
        };

        let mut servers_data = vec![];

        for server in general_data.server_list {
            match api::get_live_data(Some(server.name)).await {
                Ok(live_data) => {
                    servers_data.push(live_data);
                }
                Err(e) => {
                    log::error!("Error getting live data: {}", e);
                    continue;
                }
            }
        }

        if servers_data != last_servers_data {
            log::debug!(
                "Updating live racers in channel \"{}\" ({})...",
                chan.name,
                chan.id
            );
            log::trace!("with data: {:#?}", servers_data);

            if let Err(e) = msg
                .edit(ctx.http.clone(), |m| {
                    m.content("");

                    let mut embeds = vec![];
                    for server in servers_data.iter() {
                        let players: Vec<_> = server
                            .m_vehicles
                            .iter()
                            .filter(|v| v.m_control == 2)
                            .collect();

                        let server_colour = match u32::from_str_radix(
                            &std::env::var(format!(
                                "{}_COLOUR",
                                server.m_scoring_info.m_server_name
                            ))
                            .unwrap_or("000000".to_string()),
                            16,
                        ) {
                            Ok(c) => {
                                let (red, green, blue) = ((c >> 16) as u8, (c >> 8) as u8, c as u8);
                                Colour::from_rgb(red, green, blue)
                            }
                            Err(e) => {
                                log::warn!(
                                    "Invalid colour set for server {}: {}",
                                    server.m_scoring_info.m_server_name,
                                    e
                                );
                                Colour::default()
                            }
                        };

                        let server_thumbnail = env::var(format!(
                            "{}_THUMBNAIL",
                            &server.m_scoring_info.m_server_name
                        ))
                        .ok();

                        let mut embed = CreateEmbed::default();

                        if let Some(thumbnail) = server_thumbnail {
                            embed.thumbnail(thumbnail);
                        }

                        embed
                            .colour(server_colour)
                            .title(&server.m_scoring_info.m_server_name)
                            .field("Track", server.m_scoring_info.m_track_name.clone(), false)
                            .field("Drivers", format_drivers(&players), false)
                            .field(
                                "Track temperature",
                                utils::format_temp(server.m_scoring_info.m_track_temp),
                                true,
                            )
                            .field(
                                "Ambient",
                                utils::format_temp(server.m_scoring_info.m_ambient_temp),
                                true,
                            )
                            .field(
                                "Elapsed / End time",
                                format!(
                                    "{} / {}",
                                    utils::format_minutes_time(server.m_scoring_info.m_current_et),
                                    utils::format_minutes_time(server.m_scoring_info.m_end_et)
                                ),
                                false,
                            );

                        embeds.push(embed.to_owned());
                    }

                    m.set_embeds(embeds);
                    m
                })
                .await
            {
                log::warn!(
                    "Error updating live racers in channel \"{}\" ({}): {}",
                    chan.name,
                    chan.id,
                    e
                );
            }
        }

        last_servers_data = servers_data.clone();

        sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn clean_channel(chan: &GuildChannel, ctx: &Context) {
    match chan
        .delete_messages(
            ctx.http.clone(),
            chan.messages(ctx.http.clone(), |b| b).await.unwrap(),
        )
        .await
    {
        Ok(_) => {}
        Err(serenity::Error::Model(ModelError::BulkDeleteAmount)) => {
            log::warn!(
                "Could not delete messages in channel {}, probably because there were none. Continuing...",
                format_channel(chan)
            );
        }
        Err(e) => {
            panic!(
                "Error deleting messages in channel {}: {}",
                format_channel(chan),
                e
            );
        }
    }
}

fn format_drivers(vec: &Vec<&Vehicle>) -> String {
    if vec.is_empty() {
        return "None".to_owned();
    }

    let mut vec = vec.clone();
    vec.sort_by(|a, b| a.m_place.cmp(&b.m_place));

    vec.iter()
        .map(|v| {
            format!(
                "* {}\n  * Laps: {}\n  * Last Lap: {}\n  * Fastest Lap: {}",
                v.m_driver_name.clone(),
                v.m_total_laps,
                utils::format_laptime(v.m_last_lap_time),
                utils::format_laptime(v.m_best_lap_time)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::main]
async fn main() {
    let handle = flexi_logger::Logger::try_with_str("warn,discord_bot=info")
        .unwrap()
        .format(flexi_logger::colored_default_format)
        .log_to_stdout()
        .start()
        .unwrap();
    log_panics::init();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::empty() | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        log::error!("Client error: {:?}", why);
    }

    drop(handle);
}
