//! Explicit, fail-closed OS isolation capability contract.
//!
//! This module does not create namespaces, seccomp filters, cgroups, tokens, or
//! AppContainer profiles. It defines the policy and capability evidence that a
//! platform backend must establish before a worker may start. No platform
//! backend is enabled in this build, so `FailClosedSandbox` refuses every start.
//!
//! This is deliberate: an unsandboxed or partially configured process is never
//! an acceptable fallback for untrusted tool execution.
use std::path::PathBuf;

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPlatform {
    Linux,
    Windows,
    Other,
}

impl SandboxPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Other
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IsolationBackend {
    LinuxNamespaces,
    WindowsAppContainer,
}

impl IsolationBackend {
    pub fn platform(self) -> SandboxPlatform {
        match self {
            Self::LinuxNamespaces => SandboxPlatform::Linux,
            Self::WindowsAppContainer => SandboxPlatform::Windows,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LinuxNamespaces => "linux-namespaces",
            Self::WindowsAppContainer => "windows-appcontainer",
        }
    }
}

/// Capability evidence reported by a platform backend. A missing capability is
/// a hard refusal, not a warning.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IsolationCapabilities {
    pub user_namespace: bool,
    pub mount_namespace: bool,
    pub pid_namespace: bool,
    pub network_namespace: bool,
    pub seccomp: bool,
    pub cgroup_v2: bool,
    pub no_new_privs: bool,
    pub read_only_root: bool,
    pub restricted_token: bool,
    pub app_container: bool,
    pub job_object: bool,
}

impl IsolationCapabilities {
    pub fn linux_required() -> Self {
        Self {
            user_namespace: true,
            mount_namespace: true,
            pid_namespace: true,
            network_namespace: true,
            seccomp: true,
            cgroup_v2: true,
            no_new_privs: true,
            read_only_root: true,
            ..Self::default()
        }
    }

    pub fn windows_required() -> Self {
        Self {
            restricted_token: true,
            app_container: true,
            job_object: true,
            read_only_root: true,
            ..Self::default()
        }
    }

    pub fn required_for(backend: IsolationBackend) -> Self {
        match backend {
            IsolationBackend::LinuxNamespaces => Self::linux_required(),
            IsolationBackend::WindowsAppContainer => Self::windows_required(),
        }
    }

    /// Names of capabilities that are required but not proven by the backend.
    pub fn missing(&self, required: &Self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        for (name, required, observed) in [
            (
                "user_namespace",
                required.user_namespace,
                self.user_namespace,
            ),
            (
                "mount_namespace",
                required.mount_namespace,
                self.mount_namespace,
            ),
            ("pid_namespace", required.pid_namespace, self.pid_namespace),
            (
                "network_namespace",
                required.network_namespace,
                self.network_namespace,
            ),
            ("seccomp", required.seccomp, self.seccomp),
            ("cgroup_v2", required.cgroup_v2, self.cgroup_v2),
            ("no_new_privs", required.no_new_privs, self.no_new_privs),
            (
                "read_only_root",
                required.read_only_root,
                self.read_only_root,
            ),
            (
                "restricted_token",
                required.restricted_token,
                self.restricted_token,
            ),
            ("app_container", required.app_container, self.app_container),
            ("job_object", required.job_object, self.job_object),
        ] {
            if required && !observed {
                missing.push(name);
            }
        }
        missing
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxPolicy {
    pub backend: IsolationBackend,
    pub required: IsolationCapabilities,
    pub scratch_root: PathBuf,
    pub allow_network: bool,
    pub read_only_root: bool,
}

impl SandboxPolicy {
    pub fn linux_default(scratch_root: PathBuf) -> Self {
        Self {
            backend: IsolationBackend::LinuxNamespaces,
            required: IsolationCapabilities::linux_required(),
            scratch_root,
            allow_network: false,
            read_only_root: true,
        }
    }

    pub fn windows_default(scratch_root: PathBuf) -> Self {
        Self {
            backend: IsolationBackend::WindowsAppContainer,
            required: IsolationCapabilities::windows_required(),
            scratch_root,
            allow_network: false,
            read_only_root: true,
        }
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.backend.platform() == SandboxPlatform::current(),
            "sandbox backend does not match the current platform"
        );
        ensure!(
            !self.allow_network,
            "network access is not permitted for an isolated worker"
        );
        ensure!(
            self.read_only_root,
            "an isolated worker requires a read-only minimal root"
        );
        ensure!(
            !self.scratch_root.as_os_str().is_empty() && self.scratch_root.is_absolute(),
            "sandbox scratch root must be an absolute path"
        );
        ensure!(
            self.required == IsolationCapabilities::required_for(self.backend),
            "sandbox policy must require the complete capability set for its backend"
        );
        Ok(())
    }

    /// Stable digest of the exact isolation policy, used to bind a prepared
    /// backend to the run that requested it.
    pub fn digest(&self) -> String {
        let required = &self.required;
        let canonical = format!(
            "backend={};scratch={};network={};read_only={};user_ns={};mount_ns={};pid_ns={};net_ns={};seccomp={};cgroup_v2={};no_new_privs={};read_only_root={};restricted_token={};app_container={};job_object={}",
            self.backend.as_str(),
            self.scratch_root.display(),
            self.allow_network,
            self.read_only_root,
            required.user_namespace,
            required.mount_namespace,
            required.pid_namespace,
            required.network_namespace,
            required.seccomp,
            required.cgroup_v2,
            required.no_new_privs,
            required.read_only_root,
            required.restricted_token,
            required.app_container,
            required.job_object,
        );
        Sha256::digest(canonical.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxStatus {
    pub supported: bool,
    pub backend: IsolationBackend,
    pub platform: SandboxPlatform,
    pub missing: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedSandbox {
    pub backend: IsolationBackend,
    pub policy_sha256: String,
    pub status: SandboxStatus,
}

pub trait SandboxBackend {
    fn status(&self, policy: &SandboxPolicy) -> SandboxStatus;
    fn prepare(&self, policy: &SandboxPolicy) -> Result<PreparedSandbox>;
}

/// No platform isolation backend is implemented in this build. It reports every
/// required capability as missing and refuses to prepare a worker.
#[derive(Clone, Copy, Debug, Default)]
pub struct FailClosedSandbox;

impl SandboxBackend for FailClosedSandbox {
    fn status(&self, policy: &SandboxPolicy) -> SandboxStatus {
        SandboxStatus {
            supported: false,
            backend: policy.backend,
            platform: SandboxPlatform::current(),
            missing: policy
                .required
                .missing(&policy.required)
                .into_iter()
                .map(str::to_owned)
                .chain(std::iter::once(
                    "platform isolation backend is not implemented".to_owned(),
                ))
                .collect(),
        }
    }

    fn prepare(&self, policy: &SandboxPolicy) -> Result<PreparedSandbox> {
        policy.validate()?;
        let status = self.status(policy);
        if !status.supported {
            bail!(
                "OS isolation is unavailable ({}); refusing to start an unsandboxed worker",
                status.missing.join(", ")
            );
        }
        Ok(PreparedSandbox {
            backend: policy.backend,
            policy_sha256: policy.digest(),
            status,
        })
    }
}

/// Parse an optional, review-gated `[sandbox]` policy section. This is a
/// configuration contract only: no backend is enabled by parsing it, and a
/// missing or malformed policy always fails closed.
pub fn parse_sandbox_policy(text: &str) -> Result<SandboxPolicy> {
    let mut in_section = false;
    let mut backend = None;
    let mut scratch_root = None;
    let mut allow_network = None;
    let mut read_only_root = None;

    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_section = line == "[sandbox]";
            continue;
        }
        if !in_section {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .with_context(|| format!("invalid sandbox policy line {}", index + 1))?;
        let key = key.trim();
        let value = value.trim();
        let slot = match key {
            "backend" => &mut backend,
            "scratch_root" => &mut scratch_root,
            "allow_network" => &mut allow_network,
            "read_only_root" => &mut read_only_root,
            _ => bail!("unknown sandbox policy key: {key}"),
        };
        ensure!(slot.is_none(), "duplicate sandbox policy key: {key}");
        *slot = Some(value.to_owned());
    }

    let backend = match backend.as_deref() {
        Some("linux-namespaces") => IsolationBackend::LinuxNamespaces,
        Some("windows-appcontainer") => IsolationBackend::WindowsAppContainer,
        Some(other) => bail!("unsupported sandbox backend: {other}"),
        None => bail!("sandbox policy requires an explicit backend"),
    };
    let scratch_root =
        PathBuf::from(scratch_root.context("sandbox policy requires an explicit scratch_root")?);
    let allow_network = parse_bool(allow_network.as_deref(), "allow_network")?;
    let read_only_root = parse_bool(read_only_root.as_deref(), "read_only_root")?;
    let policy = match backend {
        IsolationBackend::LinuxNamespaces => SandboxPolicy::linux_default(scratch_root),
        IsolationBackend::WindowsAppContainer => SandboxPolicy::windows_default(scratch_root),
    };
    let policy = SandboxPolicy {
        allow_network,
        read_only_root,
        ..policy
    };
    policy.validate()?;
    Ok(policy)
}

fn parse_bool(value: Option<&str>, label: &str) -> Result<bool> {
    match value {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => bail!("{label} must be true or false, got {other}"),
        None => bail!("sandbox policy requires an explicit {label}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_policy() -> SandboxPolicy {
        let scratch = std::env::temp_dir().join("rocksoul-sandbox-test");
        match SandboxPlatform::current() {
            SandboxPlatform::Linux => SandboxPolicy::linux_default(scratch),
            SandboxPlatform::Windows => SandboxPolicy::windows_default(scratch),
            SandboxPlatform::Other => SandboxPolicy::linux_default(scratch),
        }
    }

    #[test]
    fn policy_requires_no_network_and_read_only_root() {
        let mut policy = current_policy();
        assert!(policy.validate().is_ok());
        policy.allow_network = true;
        assert!(policy.validate().is_err());
        policy.allow_network = false;
        policy.read_only_root = false;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn policy_rejects_backend_for_another_platform() {
        let scratch = std::env::temp_dir().join("rocksoul-sandbox-test");
        let policy = if SandboxPlatform::current() == SandboxPlatform::Linux {
            SandboxPolicy::windows_default(scratch)
        } else {
            SandboxPolicy::linux_default(scratch)
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn fail_closed_backend_refuses_every_prepare() {
        let policy = current_policy();
        let backend = FailClosedSandbox;
        let status = backend.status(&policy);
        assert!(!status.supported);
        assert!(
            status
                .missing
                .iter()
                .any(|item| item.contains("not implemented"))
        );
        assert!(backend.prepare(&policy).is_err());
    }

    #[test]
    fn capability_reporting_lists_each_missing_control() {
        let required = IsolationCapabilities::linux_required();
        let observed = IsolationCapabilities::default();
        let missing = observed.missing(&required);
        assert!(missing.contains(&"user_namespace"));
        assert!(missing.contains(&"seccomp"));
        assert!(missing.contains(&"cgroup_v2"));
        assert!(missing.contains(&"no_new_privs"));
    }

    #[test]
    fn sandbox_policy_config_is_strict_and_fails_closed() {
        let scratch = std::env::temp_dir()
            .join("rocksoul-sandbox-config")
            .display()
            .to_string();
        let backend = if SandboxPlatform::current() == SandboxPlatform::Windows {
            "windows-appcontainer"
        } else {
            "linux-namespaces"
        };
        let valid = format!(
            "[sandbox]\nbackend = {backend}\nscratch_root = \"{scratch}\"\nallow_network = false\nread_only_root = true\n"
        );
        // Quoted values are not part of the minimal grammar; use raw paths.
        let valid = valid.replace('"', "");
        assert!(parse_sandbox_policy(&valid).is_ok());
        let network = valid.replace("allow_network = false", "allow_network = true");
        assert!(parse_sandbox_policy(&network).is_err());
        let unknown = valid.replace("read_only_root", "unexpected_key");
        assert!(parse_sandbox_policy(&unknown).is_err());
        let duplicate = format!("{valid}allow_network = false\n");
        assert!(parse_sandbox_policy(&duplicate).is_err());
        let missing_backend = valid.replace(&format!("backend = {backend}\n"), "");
        assert!(parse_sandbox_policy(&missing_backend).is_err());
    }

    #[test]
    fn policy_digest_is_stable_and_changes_with_policy() {
        let mut policy = current_policy();
        let first = policy.digest();
        assert_eq!(first, policy.digest());
        policy.allow_network = true;
        assert_ne!(first, policy.digest());
    }
}
