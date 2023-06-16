# G-Bot

Discord bot developed for the GSMC community racing league, but can be used for any rFactor 2 and Discord server.  
Depends on a [rF2 Log Analyzer](https://forum.studio-397.com/index.php?threads/rfactor2-log-analyzer-ver-2-with-offline-and-league-championship-manager.48117/) install to get live timings and results.

## Docker image

This bot is available as a Docker image on [Docker Hub](https://hub.docker.com/r/frozendroid/gsmc-bot).

## Environment variables

### RF2LA_URL

Point this to the root of the rF2 Log analyzer website without trailing slash.

### {SERVER_NAME}\_COLOUR (Optional)

Color hex code, this determines the color of the embed, for example:  
`GSMC_COLOUR=ff7e29`

### {SERVER_NAME}\_THUMBNAIL (Optional)

A URL to an image to be used as the thumbnail for the embed, for example:

```
GSMC_THUMBNAIL="https://i.imgur.com/7tODhAL.png"
GSME_THUMBNAIL="https://i.imgur.com/x5zoGk2.png"
```

### DISCORD_TOKEN

Discord bot token that you get from https://discord.com/developers/applications.

### DISCORD_CHANNEL_ID

Channel id for the channel to be used. You can get this by enabling developer mode in Discord settings and right clicking a channel -> "Copy Channel ID"
WARNING: will wipe the entire channel on boot, so use a dedicated one for this
