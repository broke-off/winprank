pub mod commands {
    use std::str::FromStr;
    use serde::Serialize;

    #[derive(Debug, Clone, Serialize)]
    pub enum RPTCommandType {
        PlayAudio,
        RunScript,
        KillScript,
        ChangeEnabledTskManager,
        Shutdown,
        ChangeWallpaper,
        ChangeCursor,
        WinMessage
    }

    impl FromStr<> for RPTCommandType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "play-audio" => Ok(RPTCommandType::PlayAudio),
                "run-script" => Ok(RPTCommandType::RunScript),
                "shutdown" => Ok(RPTCommandType::Shutdown),
                "kill-script" => Ok(RPTCommandType::KillScript),
                &_ => Ok(RPTCommandType::RunScript),
            }
        }
    }
}