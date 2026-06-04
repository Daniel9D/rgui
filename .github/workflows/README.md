# rsgui CI

Three jobs, defined in [ci.yml](ci.yml):

- **`test`** — runs the full test suite on `ubuntu-latest` and
  `windows-latest` under three configurations: default features,
  `validation-layers` enabled (REND-03, surfaces wgpu validation
  issues the default release path silently allows), and
  `vulkan-goldens` enabled (REND-01, cross-backend visual goldens
  under Vulkan). Ubuntu runners install the Vulkan validation layers
  via `apt` (`vulkan-validationlayers` + `mesa-vulkan-drivers`); the
  Windows runner uses the Mesa software Vulkan ICD that ships with
  the runner image (the Vulkan SDK is not pre-installed).
- **`clippy`** — `cargo clippy --all-targets --all-features -- -D warnings`
  + `cargo fmt --all -- --check`. Denies warnings on the
  validation-layers-gated code paths.
- **`doc`** — `cargo doc --no-deps --document-private-items`.
  Enforces the public API is documented; the project treats
  undocumented public items as bugs.

The Vulkan golden tests from Phase 5 plan 05-01 run in the `test`
job. The stress-scene test from Phase 5 plan 05-02 runs in the
default-feature step of the same job. Future plans (05-04 frame
budget) will add their CI hooks here.
