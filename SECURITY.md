# Security

Please report vulnerabilities privately through
[GitHub security advisories](https://github.com/mooofna/turing-machines/security/advisories/new)
rather than in a public issue.

The website is static and computes everything in the visitor's browser. It sets no cookies, loads
nothing from other origins and sends no data anywhere; a content security policy restricts
scripts, workers and connections to the site itself. The Rust crates contain no `unsafe` code
(`unsafe_code = "forbid"` for the whole workspace). Secrets are kept out of the history by
gitleaks in pre-commit and CI.
