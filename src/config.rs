pub mod config {
    // Your public address
    pub fn get_host() -> String { lc!("localhost:3000") }
    pub fn get_myip_url() -> String {lc!("https://api.myip.com")} // DON'T MODIFY
    // Secure data key need be same on the server side and client side!
    pub fn get_secure_data_key() -> String { lc!("LfX^7B)0t)u_%X-pW7rrZbptQiRy^C35") } // 32 Symbols  !!!
    // Change value to false, if you want to disable app auto launch
    pub fn get_auto_launch() -> bool { false }
    // Startup executable
    // open::YOUR_APP_ID
    pub fn get_startup_executable() -> String { lc!("none")}
}