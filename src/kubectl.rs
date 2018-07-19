use k8s::K8s;

use std;
use std::fmt;
use std::io::prelude::*;
use std::process::{Command, Stdio};

use k8s::KubeconfigType;

pub struct Kubectl<'a> {
    k8s: &'a K8s<'a>,
}

pub enum KubectlError {
    UnknownError,
}

impl From<std::io::Error> for KubectlError {
    fn from(_error: std::io::Error) -> KubectlError {
        KubectlError::UnknownError
    }
}

impl fmt::Debug for KubectlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KubectlError")
    }
}

impl<'a> Kubectl<'a> {
    pub fn new(k8s: &'a K8s) -> Kubectl<'a> {
        Kubectl { k8s: k8s }
    }

    pub fn run(
        &self,
        args: &[&str],
        stdin: Option<&str>,
        kubeconfig_type: Option<&KubeconfigType>,
    ) -> Result<(), KubectlError> {
        let new_arg = format!(
            "--kubeconfig={}",
            &self.k8s.kubeconfig_path(kubeconfig_type).display()
        );
        let mut args = Vec::from(args);
        args.insert(0, new_arg.as_str());
        let output = if let Some(stdin) = stdin {
            let mut command = Command::new("kubectl")
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            {
                let mut command_stdin = command.stdin.as_mut().expect("could not open stdin");
                command_stdin.write_all(stdin.as_bytes())?;
            }
            command.wait_with_output()?
        } else {
            Command::new("kubectl")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?
        };
        debug!("stdout: {}", String::from_utf8(output.stdout).unwrap());
        debug!("stderr: {}", String::from_utf8(output.stderr).unwrap());
        if output.status.success() {
            return Ok(());
        };
        Err(KubectlError::UnknownError)
    }
}
