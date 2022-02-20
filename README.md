# BevyJamHNSorter

A small tool that fetched the [bevy #jam-theme-voting] channel and sort them with an HN/Reddit inspired rule.

## Instruction

To use this tool you **must** define the `DISCORD_AUTHORIZATION` env variable,
this value is used by this tool to fetch the messages in the channel.

Fetching will be done with the rights this `authorization` header has.
The header value can be retrieved by [using your browser while being connected to Discord].

```bash
export DISCORD_AUTHORIZATION='YOUR AUTHORIZATION TOKEN HERE'
cargo run
```

[bevy #jam-theme-voting]: https://discord.com/channels/691052431525675048/937158195007348786
[using your browser while being connected to Discord]: https://discordhelp.net/discord-token
