use gen_vulkan::{
    error::{CmdError, Error},
    Settings,
};

fn main() -> Result<(), Error> {
    let settings = process_cmd_line()?;
    gen_vulkan::run(&settings)?;

    Ok(())
}

fn process_cmd_line() -> Result<Settings, CmdError> {
    let mut args = std::env::args();
    let mut settings = Settings::default();

    let _ = args.next(); // First is always the executable location.
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => {
                settings.path = Some(
                    args.next()
                        .ok_or(CmdError::ReqCmdArg("--path".to_owned()))?,
                )
            }
            "-nV" | "--no-video" => settings.no_video = true,
            _ => return Err(CmdError::BadCmdArg(arg)),
        }
    }

    Ok(settings)
}
