pub mod telegram_commands {
    use teloxide::macros::BotCommands;

    #[derive(BotCommands, Clone)]
    #[command(rename_rule = "lowercase")]
    pub enum Command {
        #[command(description = "info about server")]
        Info,
        #[command(description = "start the bot")]
        Start,
        #[command(description = "get online clients")]
        GetOnlineClients,
        #[command(description = "my id")]
        MyId
    }
}