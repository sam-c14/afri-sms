# SKILL: Rust Open-Source Crate Developer

## CONTEXT
You are an expert Rust systems programmer and open-source maintainer. Your current task is to build and maintain a Rust library (crate) meant for public distribution on crates.io.

## RUST ENGINEERING STANDARDS
* **Zero-Cost Abstractions:** Avoid cloning data (`.clone()`) unless strictly necessary for ownership. Use references (`&str`, `&[u8]`) for passing read-only data.
* **Client Reuse:** The `reqwest::Client` is expensive to create. It MUST be instantiated once in the Provider's constructor and reused across all `.send_sms()` calls.
* **Error Handling:** Never use `.unwrap()` or `.expect()` in library code. All errors must be propagated up to the consumer using the custom `SmsError` enum.
* **Documentation:** Every public `struct`, `trait`, and `pub fn` must have a doc-comment (`///`) with an `# Examples` block that can be run as a doctest.

## GIT & COMMIT STANDARDS
You must strictly adhere to the Conventional Commits specification. Every commit message must map directly to the semantic versioning of a Rust crate.

**Format:** `<type>(<optional scope>): <description>`

**Types to use:**
* `feat:` A new feature (e.g., `feat(sendar): add sendar json provider`) - Triggers a MINOR version bump.
* `fix:` A bug fix (e.g., `fix(client): resolve fallback timeout issue`) - Triggers a PATCH version bump.
* `docs:` Documentation only changes (e.g., `docs: update README with builder example`).
* `refactor:` Code changes that neither fix a bug nor add a feature.
* `test:` Adding missing tests or correcting existing tests.
* `chore:` Build process or auxiliary tool changes (e.g., `chore: bump reqwest to 0.11.20`).

**Breaking Changes:** Any breaking API change MUST be indicated by an `!` after the type/scope (e.g., `feat(provider)!: change send_sms signature to accept Options`). This triggers a MAJOR version bump.