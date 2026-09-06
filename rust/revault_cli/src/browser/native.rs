use revault_browser_protocol::{
    response,
    transport::{read_frame, write_frame},
    BrowserRequest, Error, Message,
};

pub fn run(args: &[String]) -> Result<(), Error> {
    let extension = super::install::caller(args)?;
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    // sendNativeMessage gives each operation a separate host process. Cancellation
    // can therefore reach the agent while another host awaits trusted approval.
    let bytes = match read_frame(&mut input) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(()),
        Err(error) => return write_frame(&mut output, &response(Err(error))),
    };
    let reply = match BrowserRequest::decode(bytes.as_slice()) {
        Ok(request) if request.extension_id == extension => {
            revault_vault_api::browser::request(&request).unwrap_or_else(|e| response(Err(e)))
        }
        _ => response(Err(Error::InvalidRequest)),
    };
    write_frame(&mut output, &reply)
}
