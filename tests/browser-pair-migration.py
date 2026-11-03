"""SQLite contract test for persisted approvals and pending discoveries."""
import pathlib,re,sqlite3,unittest
SRC=pathlib.Path(__file__).resolve().parents[1]/'agent/src/bridge.rs'
class BrowserPairMigration(unittest.TestCase):
    def setUp(self):
        self.db=sqlite3.connect(':memory:')
        contents=SRC.read_text(encoding='utf-8')
        sql=re.search(r'db\.execute_batch\("(CREATE TABLE IF NOT EXISTS browser_pairs.*?)"\)', contents, re.S)
        self.assertIsNotNone(sql, 'Rust migration SQL must be present')
        self.migration=sql.group(1).replace(r'\n','\n')
        self.db.executescript(self.migration)
    def test_idempotence(self):self.db.executescript(self.migration)
    def test_pending_requests_and_pairing(self):
        id='diakokddlmicacfakghilkajnojloick'
        self.db.execute('INSERT INTO browser_pair_requests VALUES(?,?)',(id,4000))
        self.db.execute('INSERT INTO browser_pairs(extension_id,browser,created_ms) VALUES(?,?,?)',(id,'Brave',4200))
        self.assertEqual(self.db.execute('SELECT browser FROM browser_pairs').fetchone()[0],'Brave')
        self.assertEqual(self.db.execute('SELECT last_seen_ms FROM browser_pairs').fetchone()[0],0)
    def test_revoke_and_reenroll(self):
        id='diakokddlmicacfakghilkajnojloick'
        self.db.execute('INSERT INTO browser_pairs(extension_id,browser,created_ms) VALUES(?,?,?)',(id,'Brave',4200))
        self.db.execute('DELETE FROM browser_pairs WHERE extension_id=?',(id,))
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM browser_pairs').fetchone()[0],0)
        self.db.execute('INSERT INTO browser_pairs(extension_id,browser,created_ms) VALUES(?,?,?)',(id,'Brave',4300))
        self.assertEqual(self.db.execute('SELECT COUNT(*) FROM browser_pairs').fetchone()[0],1)
if __name__=='__main__':unittest.main()
