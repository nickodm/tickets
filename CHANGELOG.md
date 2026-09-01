
# CHANGELOG

## v0.2.1 - Windows Problems

- Added rusqlite3 bundled feature, so the SQLite engine is always included with the executable and doesn't need dynamic libraries. (2cfabfd)
- Fixed bug when running `summary` command in an empty database. (1ae1016)
- Fixed bug with config dir creation. (71700ae)
- Fixed bug with confirmation answer parsing due to CRLF. (40a2e7b)
- Fixed bug with database drop in Windows. (3315145)

## v0.2.0

- Added gzip compression to backup files.
- Added `restore` command.

## v0.1.1

- Fixed integed overflow in summary's difference field when exceeded the goal.
- Changed companies and tickets summaries to SQLite views.

## v0.1.0

First version
