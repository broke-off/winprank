pub mod command {
    use serde::Serialize;
    use crate::core::enums::command_types::commands::RPTCommandType;

    #[derive(Clone, Serialize, Debug)]
    pub struct RPTCommand {
        pub rpt_type: RPTCommandType,
        pub data: Vec<u8>,
        pub flags: Vec<String>,
    }
}