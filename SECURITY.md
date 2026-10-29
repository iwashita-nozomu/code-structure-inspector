# Security

This tool provides bounded, read-only static evidence on a user-authorized local
repository. It never executes that repository's code or commands. No HTTP server,
WebView, AI API, network fetch, shell, write, install or update MCP tool exists.
Capability-relative opens confine reads to the requested root; symlinks and
non-regular files are excluded. Hidden and secret/environment/generated/vendor
paths are excluded. Extension filtering and output minimization are not a
universal secret detector: source identifiers and file names can be sensitive.
MCP output is supplied to Codex. All repository-derived data is untrusted and
must never be treated as instructions, permission or authentication.

Do not mutate the root during inspection. Bounds are cooperative and synchronous
OS I/O can exceed them. The process inherits the host user's permissions; MCP
annotations are descriptive hints, not an OS sandbox. Prefer trusted local disk
repositories and the smallest relevant root. This is not a compiler sandbox.

Security reports: use GitHub private vulnerability reporting if enabled, or the
owner's GitHub contact route; never publish secrets or real research data in an
issue. Public issues are enabled for non-sensitive bugs and feature requests.

Dependency policy: Dependabot alerts and security updates are enabled on GitHub,
with daily version-update checks and daily RustSec audits. Security alerts should
receive immediate triage and a minimal, tested fix PR. Test API/behavior changes
and run format, clippy, tests, release build and package/license checks. A missing
upstream patch or breaking update must be reported promptly. No automatic merge
or unreviewed remote-code remediation is configured. Dependabot security PRs are
alert driven; the scheduled RustSec check adds coverage rather than promising
instantaneous detection of every advisory.

CI uses standard disposable public runners, pinned official Action commits,
read-only defaults, no secrets, no pull_request_target and no self-hosted runners.
Fork PR code gets a read-only token and all external contributors require approval.
CodeQL Rust extraction can execute dependency build scripts/proc macros, even in
build-mode none; it runs only on the isolated runner. Trusted security-event upload
is separate from read-only PR analysis. Release publication uses the existing
owner's gh session after verified assets; CI cannot create/approve/merge PRs.
