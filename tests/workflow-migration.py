"""Additive task profile migration regression coverage (schema v7 to v8)."""
import pathlib
import re
import sqlite3
import unittest

SOURCE=(pathlib.Path(__file__).resolve().parents[1]/"agent/src/workflow.rs").read_text(encoding="utf-8")
MIGRATION=re.search(r'db\.execute_batch\("(BEGIN;.*?PRAGMA user_version=8;COMMIT;)"\)\?;',SOURCE,re.S)
if MIGRATION is None:
    raise RuntimeError("Task workflow migration SQL was not found")

class TaskMigrationTests(unittest.TestCase):
    def new_db(self):
        db=sqlite3.connect(":memory:")
        db.executescript("CREATE TABLE focus_sessions(id TEXT PRIMARY KEY,goal_id TEXT); PRAGMA user_version=7;")
        db.execute("INSERT INTO focus_sessions(id,goal_id) VALUES(?,?)",("original-focus","goal-1"))
        return db
    def test_task_profiles_migrate_without_replacing_focus(self):
        db=self.new_db()
        db.executescript(MIGRATION.group(1))
        self.assertEqual(db.execute("PRAGMA user_version").fetchone()[0],8)
        self.assertEqual(db.execute("SELECT goal_id FROM focus_sessions WHERE id='original-focus'").fetchone()[0],"goal-1")
        self.assertIn("task_id",[col[1] for col in db.execute("PRAGMA table_info(focus_sessions)")])
        db.execute("INSERT INTO workflow_profiles(id,name,apps_json,domains_json,created_ms,updated_ms) VALUES('one','Coding','[]','[]',1,1)")
        db.execute("INSERT INTO workflow_tasks(id,profile_id,title,created_ms,updated_ms) VALUES('task','one','Finish exercise',1,1)")
        self.assertEqual(db.execute("SELECT state FROM workflow_tasks WHERE id='task'").fetchone()[0],"draft")
    def test_task_state_is_not_observed_time(self):
        db=self.new_db();db.executescript(MIGRATION.group(1))
        self.assertNotIn("measured_ms",[col[1] for col in db.execute("PRAGMA table_info(workflow_tasks)")])
if __name__=="__main__":
    unittest.main()
