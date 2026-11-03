"""Dependency-free SQLite migration smoke test using SQL literals from the Rust source.

This is not a Rust compile test. It checks the SQL emitted for fresh and legacy stores.
"""
import pathlib
import re
import sqlite3
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
MAIN = (ROOT / "agent/src/main.rs").read_text(encoding="utf-8")
FEATURES = (ROOT / "agent/src/features.rs").read_text(encoding="utf-8")
DEVICES = (ROOT / "agent/src/devices.rs").read_text(encoding="utf-8")


def statements(source):
    return re.findall(r'db\.execute_batch\("(.*?)"\)\?;', source, re.S)


class MigrationTests(unittest.TestCase):
    def new_connection(self):
        db = sqlite3.connect(":memory:")
        db.executescript(statements(MAIN)[0])
        return db

    def test_bootstrap_current_schema(self):
        db = self.new_connection()
        batches = statements(FEATURES)
        self.assertGreaterEqual(len(batches), 6)
        for sql in batches[:6]:
            db.executescript(sql)
        self.assertEqual(db.execute("PRAGMA user_version").fetchone()[0], 7)
        self.assertEqual(db.execute("SELECT value FROM settings WHERE key='distraction_categories'").fetchone()[0], "[]")
        self.assertEqual(db.execute("SELECT value FROM settings WHERE key='prediction_opt_in'").fetchone()[0], "false")
        self.assertTrue(db.execute("SELECT name FROM sqlite_master WHERE name='rule_states'").fetchone())
        self.assertTrue(any(col[1] == 'profile_id' for col in db.execute("PRAGMA table_info(sessions)")))
        db.execute("INSERT INTO rules VALUES(?,?,?,?,?,?,?,?,?)", ("uuid", "Rule", "youtube.com", None, None, 30, 1, 1000, "{}"))
        db.execute("INSERT INTO classifications VALUES(?,?,?,?,?,?)", ("app:foo.exe", "awaiting_consent", "Unclassified", "reason", 1, 1))
        self.assertEqual(db.execute("SELECT COUNT(*) FROM rules").fetchone()[0], 1)
        db.executescript(statements(DEVICES)[0])
        db.execute("INSERT INTO device_sessions VALUES(?,?,?,?,?,?,?)", ("external-1", "session-1", 100, 200, "code.exe", 0, None))
        db.execute("INSERT OR IGNORE INTO device_sessions VALUES(?,?,?,?,?,?,?)", ("external-1", "session-1", 100, 200, "code.exe", 0, None))
        self.assertEqual(db.execute("SELECT COUNT(*) FROM device_sessions").fetchone()[0], 1)

    def test_legacy_v1_preserves_data(self):
        db = self.new_connection()
        db.execute("PRAGMA user_version=1")
        db.execute("INSERT INTO sessions VALUES(?,?,?,?,?,?)", ("old", 100, 200, "code.exe", 0, None))
        batches = statements(FEATURES)
        for sql in batches[:6]:
            db.executescript(sql)
        self.assertEqual(db.execute("PRAGMA user_version").fetchone()[0], 7)
        self.assertEqual(db.execute("SELECT app FROM sessions WHERE id='old'").fetchone()[0], "code.exe")
        self.assertTrue(any(col[1] == 'config_json' for col in db.execute("PRAGMA table_info(rules)").fetchall()))

    def test_v3_adds_distraction_without_repeating_alter(self):
        db = self.new_connection()
        for sql in statements(FEATURES)[:2]:
            db.executescript(sql)
        db.execute("PRAGMA user_version=3")
        for sql in statements(FEATURES)[2:6]:
            db.executescript(sql)
        self.assertEqual(db.execute("PRAGMA user_version").fetchone()[0], 7)

if __name__ == '__main__':
    unittest.main()
