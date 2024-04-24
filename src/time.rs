use std::time::SystemTime;

pub fn sys_nanos() -> Result<u128, std::time::SystemTimeError> {
    let duration_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(duration_since_epoch.as_nanos())
}
