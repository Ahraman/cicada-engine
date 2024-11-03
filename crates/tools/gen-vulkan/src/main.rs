use gen_vulkan::{
    emit::EmitSettings,
    error::{CmdError, Error},
    parse::ParseSettings,
};

fn main() -> Result<(), Error> {
    let (parse, emit) = process_cmd_line()?;
    gen_vulkan::run(&parse, &emit)?;

    Ok(())
}

fn process_cmd_line() -> Result<(ParseSettings, EmitSettings), CmdError> {
    let mut args = std::env::args();
    let (mut parse, emit) = (ParseSettings::default(), EmitSettings::default());

    let _ = args.next(); // First is always the executable location.
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => {
                parse.path = Some(
                    args.next()
                        .ok_or(CmdError::ReqCmdArg("--path".to_owned()))?,
                )
            }
            // "-nV" | "--no-video" => settings.no_video = true,
            _ => return Err(CmdError::BadCmdArg(arg)),
        }
    }

    Ok((parse, emit))
}
