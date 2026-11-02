# Back up and restore local learning data

Stop Coach if it is running, finish any pending browser recording saves, and quit Phrase Gym before copying files.
Use the complete data directory, including `phrase-gym.sqlite3`, any `-wal` and `-shm` files,
`references/`, and `recordings/`. The default is the OS user data directory for `phrase-gym`;
on macOS it is `~/Library/Application Support/phrase-gym`. If you launched with `--data-dir`, use
that exact directory instead. Keep backups private because they contain learner recordings.

With the application and any preparation process stopped, run these commands after setting `DATA_DIR` to the correct path:

```bash
DATA_DIR="$HOME/Library/Application Support/phrase-gym"
BACKUP_DIR="$HOME/Phrase Gym backups/phrase-gym-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$(dirname "$BACKUP_DIR")"
cp -Rp "$DATA_DIR" "$BACKUP_DIR"
```

Check that the backup contains the database, `references/` and `recordings/`. To restore, stop
the application and any preparation process again and keep the current directory as a fallback. Copy the entire dated backup
back into place; do not overwrite a live database or copy only its main file.

```bash
mv "$DATA_DIR" "$DATA_DIR.before-restore-$(date +%Y%m%d-%H%M%S)"
cp -Rp "$BACKUP_DIR" "$DATA_DIR"
```

Start the same compatible Phrase Gym version. In the Library, check lesson and phrase counts and
open representative phrase details for due dates and saved recordings. Play a representative
reference and each saved take you need to confirm.
If a copy fails, keep the fallback and backup untouched and retry while the app is stopped.

## Automatic copy before an upgrade

When a new Phrase Gym version needs a newer database layout, it upgrades your database in place
on startup. Just before that, it saves a copy of the database beside it, named
`phrase-gym-schema-<old version>-<date and time>.sqlite3`. Your references and recordings are not
changed by an upgrade. To go back to the previous Phrase Gym version, stop the application, move
`phrase-gym.sqlite3` and any `-wal` and `-shm` files aside, copy the saved file to
`phrase-gym.sqlite3`, and start the previous version.
