# Feature coverage review

Follow `CODE_STANDARDS.md` §11. Tests and test-only helpers belong in this
crate's `tests/` directory and exercise the public library API. Keep `src/`
free of test declarations, test source files, and test-only helpers.

Strongly target no more than five `#[test]` cases for each major feature,
counted across suite names and files. Use simple fixtures and rule-like tests,
consolidating related inputs without deleting useful regression coverage.
Before committing, review affected features and explain specific independent
contracts that warrant any additional cases.

The state integration entry point spans several responsibilities: movement
and pickup, placement, repair, upgrades, shift timing, records, save migration,
and complete-shift replay. Review those responsibilities, not the aggregate
entry point or each child filename, against the feature target. A repair rule
still belongs to repair when it is exercised from another suite.

Content validation, audio behavior, and persistence adapters have different
failure boundaries; preserve their meaningful contracts while consolidating
related inputs. Existing render-math or presentation checks do not establish a
requirement for general UI/rendering unit coverage. Use captures and manual
browser checks for presentation and touch behavior.

The ignored replay reports are explicit balance diagnostics. Run them when
retuning the shop size or full-shift route; normal regression cases should
assert gameplay outcomes. Report current execution results with the change
rather than treating an earlier machine's permissions or test counts as a
standing property of the project.
