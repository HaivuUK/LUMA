# Security Policy

## Supported Versions

| Version               | Supported          |
| --------------------- | ------------------ |
| Latest stable release | :white_check_mark: |

LUMA is in active development. Security fixes are applied to the latest released version.

## Reporting a Vulnerability

If you discover a security vulnerability in LUMA, please report it responsibly. **Do not open a public issue.**

### How to Report

Use [GitHub's private vulnerability reporting](https://github.com/HaivuUK/LUMA/security/advisories/new) to submit a security advisory. This is the only channel for security reports and keeps the details private until a fix is available.

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Affected component(s) and version(s)
- Potential impact assessment
- Suggested fix (if available)

### Report Quality

- Keep reports short and concise. Include only the information needed to understand the threat and reproduce it, and do not overstate the impact.
- Do **not** include Personally Identifiable Information (PII) in your report. Redact or obfuscate any PII in your proof of concept (screenshots, JSON files, etc.) as much as possible. The same applies to secrets, keys, and credentials.
- If you used a large language model (LLM) to prepare the report, please disclose how. Review and edit any generated output before sending it, verify that your reproduction steps actually work, and confirm that everything in the report is valid and correct.
- All reports are validated manually. Submissions from automated tools (static analysis, AI, etc.) will not be considered unless you have manually reviewed and validated them first.

### What to Expect

- **Acknowledgment**: We aim to acknowledge your report within 5 business days.
- **Assessment**: The team will evaluate severity and impact and keep you informed of progress.
- **Fix Timeline**: Critical vulnerabilities are prioritized for patching. Other issues are addressed based on severity.
- **Disclosure**: We follow coordinated disclosure. Once a fix is released, we will publish an advisory and credit the reporter, unless anonymity is requested.

## Scope

This policy covers the [LUMA](https://github.com/HaivuUK/LUMA) repository.

## Security Considerations

LUMA is a rust application that uses the [Tauri](https://tauri.app/) framework and a number of rust crate dependencies. Known CVEs affecting these dependencies may also affect LUMA. If you are aware of an dependency vulnerability that has not been addressed here, please report it to us using the process above.

## Acknowledgments

We appreciate the security research community's efforts in helping keep LUMA and its users safe.