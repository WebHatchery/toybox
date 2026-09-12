# Shared runtime audit — 2026-09-12

The publisher's shared runtime sources and the deployed Preview runtime were
checked before closing the cleanup item. The project-specific bridge files
(`storage.js`, `clipboard.js`, `quad-net.js`, and
`macroquad-gamepads-0.1.js`) have identical SHA-256 hashes between
`rust_management/web/` and `Release/shared-assets/runtime/`. The upstream
`mq_js_bundle.js` and `sapp_jsutils.js` files are maintained only in the
deployed runtime directory, as the publisher downloads/refreshes them there.

No divergent stale runtime file was found, so no destructive deletion was
needed. The audit is recorded here because the managed session cannot write
the external `rust_management` tree.
