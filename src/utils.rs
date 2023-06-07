use serenity::model::prelude::GuildChannel;

// time in minutes f32 (595.4 minutes) to mm:ss
pub fn format_minutes_time(time: f32) -> String {
    if time <= 0.0 {
        return "--:--".to_owned();
    }

    let minutes = (time / 60.0).floor();
    let seconds = (time - minutes * 60.0).floor();

    format!("{}:{:02}", minutes, seconds)
}

pub fn format_laptime(seconds: f32) -> String {
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

pub fn format_temp(temp: f32) -> String {
    format!("{:.2}°C / {:.0}°F", temp, temp * 1.8 + 32.0)
}

pub fn format_channel(chan: &GuildChannel) -> String {
    format!("\"{}\" (id: {})", chan.guild_id, chan.id)
}
