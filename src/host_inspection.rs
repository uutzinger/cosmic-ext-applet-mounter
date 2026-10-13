// SPDX-License-Identifier: MIT

//! Fixed, read-only engine and host-namespace inspections. Never accept commands
//! from the UI or expose captured configuration in operation errors.

use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

use crate::process::{
    CommandError, CommandExecutionMode, CommandRequest, CommandRunner, Executable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostInspection {
    RcloneConfigDump,
    HostMountInfo,
}

impl HostInspection {
    pub async fn run_owned(self) -> Result<String, String> {
        self.run_current(CancellationToken::new()).await
    }

    pub async fn run_current(self, cancellation: CancellationToken) -> Result<String, String> {
        if self == Self::HostMountInfo
            && CommandExecutionMode::detect_current() == CommandExecutionMode::Native
        {
            // Native /proc already has the correct namespace. Keep this a
            // bounded asynchronous read, not an additional polling subprocess.
            let request = self.request();
            let read = async {
                let file = tokio::fs::File::open("/proc/self/mountinfo")
                    .await
                    .map_err(|error| error.to_string())?;
                let mut bytes = Vec::new();
                file.take((request.output_limit + 1) as u64)
                    .read_to_end(&mut bytes)
                    .await
                    .map_err(|error| error.to_string())?;
                if bytes.len() > request.output_limit {
                    return Err(crate::fl!("host-inspection-output-too-large"));
                }
                String::from_utf8(bytes).map_err(|_| crate::fl!("host-inspection-invalid-output"))
            };
            return tokio::select! {
                result = read => result,
                () = cancellation.cancelled() => Err(CommandError::Cancelled {
                    command: request.sanitized_command(),
                }.to_string()),
                () = tokio::time::sleep(request.timeout) => Err(CommandError::Timeout {
                    command: request.sanitized_command(), timeout: request.timeout,
                }.to_string()),
            };
        }
        self.run(
            &crate::process::RuntimeCommandRunner::detect_current(),
            cancellation,
        )
        .await
    }

    fn request(self) -> CommandRequest {
        let (executable, args, limit) = match self {
            Self::RcloneConfigDump => (Executable::Rclone, &["config", "dump"][..], 1024 * 1024),
            Self::HostMountInfo => (
                Executable::Cat,
                &["/proc/self/mountinfo"][..],
                4 * 1024 * 1024,
            ),
        };
        let mut request = CommandRequest::new(executable)
            .watch_host_bus()
            .with_timeout(Duration::from_secs(10))
            .with_output_limit(limit);
        for arg in args {
            request = request.arg(*arg).expect("fixed inspection argument");
        }
        request
    }

    pub async fn run(
        self,
        runner: &dyn CommandRunner,
        cancellation: CancellationToken,
    ) -> Result<String, String> {
        if cancellation.is_cancelled() {
            return Err(CommandError::Cancelled {
                command: self.request().sanitized_command(),
            }
            .to_string());
        }
        let output = runner
            .run(self.request(), cancellation)
            .await
            // CommandError's Display deliberately omits captured stdout/stderr.
            .map_err(|error| error.to_string())?;
        if output.stdout.truncated || output.stderr.truncated {
            return Err(crate::fl!("host-inspection-output-too-large"));
        }
        if output.stdout.invalid_utf8 || output.stderr.invalid_utf8 {
            return Err(crate::fl!("host-inspection-invalid-output"));
        }
        if self == Self::RcloneConfigDump {
            let value: serde_json::Value = serde_json::from_str(&output.stdout.text)
                .map_err(|_| crate::fl!("host-inspection-invalid-output"))?;
            if !value.is_object() {
                return Err(crate::fl!("host-inspection-invalid-output"));
            }
        }
        Ok(output.stdout.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{CapturedOutput, CommandError, CommandOutput, FakeCommandRunner};

    fn output(text: &str) -> CommandOutput {
        CommandOutput {
            command: "inspection".into(),
            stdout: CapturedOutput {
                text: text.into(),
                truncated: false,
                invalid_utf8: false,
            },
            stderr: CapturedOutput {
                text: String::new(),
                truncated: false,
                invalid_utf8: false,
            },
            attempts: 1,
            duration: Duration::ZERO,
        }
    }

    #[test]
    fn fixed_requests_are_bounded_and_not_retried() {
        for operation in [
            HostInspection::RcloneConfigDump,
            HostInspection::HostMountInfo,
        ] {
            let request = operation.request();
            assert_eq!(request.timeout, Duration::from_secs(10));
            assert!(!request.launch_only);
            assert!(request.watch_host_bus);
            assert_eq!(request.retry.max_attempts, 1);
            assert!(request.output_limit <= 4 * 1024 * 1024);
        }
        assert_eq!(
            HostInspection::RcloneConfigDump
                .request()
                .sanitized_command(),
            "rclone config dump"
        );
        assert_eq!(
            HostInspection::HostMountInfo.request().sanitized_command(),
            "cat /proc/self/mountinfo"
        );
    }

    #[tokio::test]
    async fn rejects_truncated_invalid_and_non_object_configuration() {
        for text in ["not json", "[]", "null"] {
            let runner = FakeCommandRunner::default();
            runner.push(Ok(output(text)));
            assert!(
                HostInspection::RcloneConfigDump
                    .run(&runner, CancellationToken::new())
                    .await
                    .is_err()
            );
        }
        for invalid in [false, true] {
            let runner = FakeCommandRunner::default();
            let mut result = output("{}");
            result.stdout.truncated = !invalid;
            result.stdout.invalid_utf8 = invalid;
            runner.push(Ok(result));
            assert!(
                HostInspection::RcloneConfigDump
                    .run(&runner, CancellationToken::new())
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn failure_never_echoes_configuration_or_diagnostics() {
        let runner = FakeCommandRunner::default();
        runner.push(Err(CommandError::NonZero {
            command: "rclone config dump".into(),
            code: Some(1),
            stdout: output("secret configuration").stdout,
            stderr: output("secret diagnostic").stdout,
            attempts: 1,
        }));
        let error = HostInspection::RcloneConfigDump
            .run(&runner, CancellationToken::new())
            .await
            .unwrap_err();
        assert!(!error.contains("secret"));
    }

    #[tokio::test]
    async fn cancelled_before_poll_does_not_launch_a_child() {
        let runner = FakeCommandRunner::default();
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        assert!(
            HostInspection::RcloneConfigDump
                .run(&runner, cancellation)
                .await
                .is_err()
        );
        assert!(runner.requests().is_empty());
    }
}
