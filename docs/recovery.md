# Rust Server backup and recovery

Use the Rust Server commands against Rust data directories only. The Rust release does not import Python databases, media or browser journals. These local commands require filesystem access to the data; they do not use Coach or providers.

Create the parent directory for a new snapshot, then back up the running or stopped Server:

```bash
mkdir -p .data/rust-snapshots
cargo run --locked -p phrase-gym-server -- backup --data-dir .data/rust-server --snapshot .data/rust-snapshots/pre-upgrade
cargo run --locked -p phrase-gym-server -- verify-backup --snapshot .data/rust-snapshots/pre-upgrade
```

Every destination must be new, with an existing parent, and outside its source directory. Existing destinations, including empty or incomplete directories, are refused. Use a new name for another attempt.

Backup uses SQLite's online backup API, which includes committed WAL pages and produces a consistent database image during concurrent writes. It then copies and verifies the immutable media referenced by that image. The snapshot contains `phrase-gym.sqlite3`, a `media/` directory and `snapshot.json` with format, Rust schema revision, application version and SHA-256/length inventories. Keep the entire directory together. The database includes all recording BLOBs, content revisions, schedules/history, sessions, device settings and staged-package metadata. Every reference-media catalog entry is retained, including historical/session references, staged packages and uploads not yet published. Existing provenance/preparation versions remain in their database records. There is no active guide registry in the current schema.

A missing or corrupt referenced asset fails the backup. A failed or interrupted operation leaves a reserved or marked incomplete destination; verification and Server startup refuse it. Do not remove recovery markers to force startup. Retain the original data and investigate the failed operation; create a fresh destination for retry. A `NAME.recovery-pending` sibling file reserves a destination before its internal `.recovery-pending` marker is durable. Complete snapshots are also refused as live data directories, preventing accidental migration of the backup itself.

Restore into isolation:

```bash
cargo run --locked -p phrase-gym-server -- restore --snapshot .data/rust-snapshots/pre-upgrade --data-dir .data/rust-restore
```

Restore verifies the manifest, database checksum, recognized Rust revision, SQLite integrity and foreign keys, recording checksums and every referenced media file. It copies to the fresh destination, checks the resulting image again and completes only after the files are synchronized. It never overwrites a running or stopped data directory and never migrates the snapshot. Verify content, settings, schedules/history, saved takes and reference playback using the intended Server release against this isolated directory before switching the stopped Server to it. Learner recordings remain opaque original bytes: checksum checks establish retention, while actual browser/device playback remains a separate acceptance check.

Before an upgrade, keep the previous Server binary and take a complete snapshot. For rollback, stop the upgraded Server, restore the matching pre-upgrade snapshot into a new directory, and run the previous release against that directory. Schema, content, recordings and media return together to the snapshot point; later writes remain in the upgraded directory. Never copy back only a database, run a schema-only downgrade or mix files from different snapshots. Running a newer release against a restored older revision performs its normal forward migration, so use the matching binary when rehearsing rollback.

Reconciliation reports missing/corrupt originals and orphan hashes and deletes no files. It retains old and active staging bytes, historical/session media and backup assets regardless of age. Unacknowledged staging files and unreferenced orphan bytes remain at the source; they are not claimed as acknowledged input in a snapshot. Disk planning must account for this retention and every learner take; there is no automatic expiry.

Profiles, durable inbox generations/attachments, guide registries and worker leases are not present yet. Their implementation must extend snapshot asset validation and recovery tests, then invalidate recovered leases and reconcile transfers/receipts before workers resume. The broader operational and actual-device rehearsals remain in the [implementation plan](../specs/plan/implementation-plan.md#9-private-operation-and-recovery).
