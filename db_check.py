import sqlite3
import sys

def query_db():
    db_path = 'libs/elite_pdf/ingestion.db'
    try:
        conn = sqlite3.connect(db_path)
        cur = conn.cursor()
        
        # 1. List tables
        cur.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [row[0] for row in cur.fetchall()]
        print(f"Tables in {db_path}: {tables}")
        
        if 'pdf_ingestion' in tables:
            # 2. Query status counts (Check for success)
            # The schema might use 'status' or just count existing entries
            cur.execute("SELECT COUNT(*) FROM pdf_ingestion")
            total = cur.fetchone()[0]
            print(f"Total entries in pdf_ingestion: {total}")
            
            # Try to get specific status if column exists
            try:
                cur.execute("SELECT status, COUNT(*) FROM pdf_ingestion GROUP BY status")
                counts = cur.fetchall()
                print(f"Ingestion status counts: {counts}")
            except Exception:
                print("Status column not found, assuming all entries are success for now.")
        elif 'execution_warrants' in tables:
            # Maybe it's the other schema
            cur.execute("SELECT result, COUNT(*) FROM execution_events GROUP BY result")
            counts = cur.fetchall()
            print(f"Execution events: {counts}")
        else:
            print("Target table not found in detected tables.")
            
        conn.close()
    except Exception as e:
        print(f"Error querying database: {e}")

if __name__ == '__main__':
    query_db()
