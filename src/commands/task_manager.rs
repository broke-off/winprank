pub mod task_manager {
    use std::error::Error;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    pub fn disable_task_manager() -> Result<(), Box<dyn Error>> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System";
        let (key, _disp) = hkcu.create_subkey(path)?;

        key.set_value("DisableTaskMgr", &1u32)?;
        Ok(())
    }
}