use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Capability Denied: {capability}")]
#[diagnostic(
    code(env_architect::security::denied),
    help("Add '{capability}' to your manifest's 'capabilities' section to allow this action.")
)]
pub struct CapabilityError {
    pub capability: String,

    #[source_code]
    pub src: NamedSource<String>,

    #[label("this capability is required but not granted")]
    pub span: SourceSpan,
}

pub fn report_denied_capability(file_path: &str, file_content: &str, capability: &str) {
    let span = match file_content.find(capability) {
        Some(offset) => SourceSpan::new(offset.into(), capability.len().into()),
        None => SourceSpan::new(SourceOffset::from(0), 0_usize.into()),
    };

    let err = CapabilityError {
        capability: capability.to_string(),
        src: NamedSource::new(file_path, file_content.to_string()),
        span,
    };

    println!("{:?}", miette::Report::new(err));
}
