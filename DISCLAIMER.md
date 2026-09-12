# Disclaimer and limitation of liability

**READ THIS BEFORE INSTALLING, USING, FORKING, CONTRIBUTING TO, OR RELYING ON
ANYTHING IN THIS REPOSITORY.**

By cloning, downloading, building, installing, running, distributing, or
contributing to **cmux-moshi**, you acknowledge and agree to this document, the
[MIT License](LICENSE), [TERMS.md](TERMS.md), and (if you contribute)
[CONTRIBUTING.md](CONTRIBUTING.md). If you do not agree, do not use or contribute
to this software.

## Unofficial project

**cmux-moshi** is an independent, community-maintained host integration. It is
**not** an official product of, and is **not** affiliated with, endorsed by, or
sponsored by:

- [cmux](https://cmux.com) / [manaflow-ai/cmux](https://github.com/manaflow-ai/cmux)
- [Moshi](https://getmoshi.app) or its publishers
- Apple, GitHub, crates.io, Cursor, Railway, cloud providers, or any other
  third party whose names appear in docs or tooling

“cmux”, “Moshi”, and other product names are trademarks of their respective
owners. They are used only to describe compatibility.

## No warranty (absolute)

THE SOFTWARE, DOCUMENTATION, RELEASE ASSETS, CI ARTIFACTS, DEPENDENCIES, AND
ANY ADVICE IN THIS REPOSITORY ARE PROVIDED **“AS IS” AND “AS AVAILABLE”**,
WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, TITLE,
NON-INFRINGEMENT, SECURITY, ACCURACY, OR QUIET ENJOYMENT.

**There is no warranty that the software is free of defects, malware,
backdoors, supply-chain compromise, accidental harmful logic, or contributor
error.** Maintainers and copyright holders do **not** promise to review,
audit, sanitize, or continuously monitor every commit, pull request,
dependency, action, or release binary.

## Limitation of liability (maximum extent)

TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, THE AUTHORS, COPYRIGHT
HOLDERS, MAINTAINERS, AND CONTRIBUTORS SHALL **NOT BE LIABLE** FOR ANY CLAIM,
DAMAGES, OR OTHER LIABILITY — WHETHER IN CONTRACT, TORT (INCLUDING NEGLIGENCE),
STRICT LIABILITY, OR OTHERWISE — ARISING FROM, OUT OF, OR IN CONNECTION WITH
THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE, **INCLUDING WITHOUT
LIMITATION**:

1. **Supply-chain and dependency risk** — compromised crates, GitHub Actions,
   runners, mirrors, tags, release binaries, checksums, package registries,
   transitive dependencies, or build tooling.
2. **Malicious or defective contributions** — code, docs, or CI changes
   submitted by third parties that a maintainer merges, accepts, or overlooks,
   whether or not review was attempted.
3. **Credits, quotas, and cloud spend** — overuse or unexpected consumption of
   API credits, CI minutes, cloud compute, bandwidth, Cursor/agent usage,
   LLM tokens, hosting, or any third-party billable resource triggered by
   installing, building, testing, running, or automating this project.
4. **Host damage** — data loss, session kill/rename, shell rc edits,
   LaunchAgent installs, credential exposure, downtime, or system compromise.
5. **Indirect / consequential loss** — lost profits, lost data, business
   interruption, reputational harm, or punitive damages, even if advised of
   the possibility.

IF ANY LIABILITY CANNOT BE FULLY EXCLUDED IN YOUR JURISDICTION, IT IS LIMITED
TO THE GREATER OF (A) ZERO (US$0) OR (B) THE MINIMUM AMOUNT REQUIRED BY LAW.
YOU ASSUME **ALL** RISK.

## Your responsibilities

You alone are responsible for:

- Verifying source, commits, signatures, and checksums before use
- Pinning and auditing dependencies and CI for your threat model
- Backups, access control, and safe use of `sync` / `cleanup` / installers
- Any cloud, CI, or AI-agent spend incurred in your accounts
- Deciding whether to trust a release, fork, or contributor patch

Prefer `--dry-run` where available. Treat every binary and PR as untrusted
until **you** have validated it.

## Security reports

Suspected vulnerabilities: [SECURITY.md](SECURITY.md) (private). Filing a
report does **not** create any duty, SLA, or liability for maintainers.

## Scope reminder

Phones connecting over Mosh/SSH never see the cmux left sidebar. This repo’s
product is the **host CLI** (and optional packaging). See [AGENTS.md](AGENTS.md).

## Conflict

If anything in READMEs or issues conflicts with [LICENSE](LICENSE), this file,
or [TERMS.md](TERMS.md), the license and these liability terms control.
