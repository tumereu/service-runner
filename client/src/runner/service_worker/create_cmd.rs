use std::fmt::Display;
use std::process::{Command, Stdio};
use derive_more::Error;
use subst::Error;
use subst::error::{InvalidEscapeSequence, MissingClosingBrace, MissingVariableName, UnexpectedCharacter};
use crate::config::ExecutableEntry;
use crate::runner::service_worker::create_cmd::CmdCreationError::MalformattedExpression;

#[derive(Debug, Error)]
pub enum CmdCreationError {
    NoSuchVariable {
        var: String,
        target: String
    },
    MalformattedExpression {
        explanation: String,
        target: String,
    }
}
impl Display for CmdCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalformattedExpression { explanation, target } => {
                write!(f, "Error in expression '{target}': {explanation}")
            }
            CmdCreationError::NoSuchVariable { var, target } => {
                write!(f, "Error in expression '{target}': variable '{var}' is not defined")
            }
        }
    }
}

fn env_subst(value: &str) -> Result<String, CmdCreationError> {
    subst::substitute(value, &subst::Env)
            .map_err(|err| {
                match err {
                    Error::InvalidEscapeSequence(InvalidEscapeSequence { position, .. }) => MalformattedExpression {
                        explanation: format!("Expression contains an invalid escape sequence at position {position}"),
                        target: value.to_string()
                    },
                    Error::MissingVariableName(MissingVariableName { position, .. }) => MalformattedExpression {
                        explanation: format!("Expression is missing a variable name at position {position}"),
                        target: value.to_string()
                    },
                    Error::UnexpectedCharacter(UnexpectedCharacter { position, .. }) => MalformattedExpression {
                        explanation: format!("Expression contains an unexpected character at position {position}"),
                        target: value.to_string()
                    },
                    Error::MissingClosingBrace(MissingClosingBrace { position }) => MalformattedExpression {
                        explanation: format!("Expression is missing a closing brace for position {position}"),
                        target: value.to_string()
                    },
                    Error::NoSuchVariable(info) => CmdCreationError::NoSuchVariable {
                        var: info.name,
                        target: value.to_string()
                    }
                }
            })
}

pub fn create_cmd<S>(entry: &ExecutableEntry, dir: Option<S>) -> Result<Command, CmdCreationError>
where
    S: AsRef<str>,
{
    let mut cmd = Command::new(env_subst(&entry.executable)?);
    let args: Vec<String> = entry.args.iter()
        .map(|arg| env_subst(arg.as_ref()))
        .collect::<Result<Vec<String>, CmdCreationError>>()?;
    cmd.args(args);
    if let Some(dir) = dir {
        cmd.current_dir(env_subst(dir.as_ref())?);
    }
    for (key, value) in &entry.env {
        // Substitute environment variables if placeholders are used in the env entry
        cmd.env(key.clone(), env_subst(value)?);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Set process group
    if cfg!(target_os = "linux") {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }

    Ok(cmd)
}
