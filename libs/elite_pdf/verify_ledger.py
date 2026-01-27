import os
import sqlite3
import shutil

db_path = "ingestion.db"
if os.path.exists(db_path):
    os.remove(db_path)
    print(f"Removed old {db_path} (PRE-IMPORT)")

import elite_pdf

def verify():
    test_pdf = "real_test.pdf"
    
    print("\n--- FIRST RUN ---")
    try:
        doc1 = elite_pdf.EliteDocument(test_pdf)
        print(f"Pages: {doc1.count_pages()}")
    except Exception as e:
        print(f"First run error: {e}")

    print("\n--- SECOND RUN (Same File) ---")
    try:
        # On second run, it should print [Elite Ledger] logs if successful
        doc2 = elite_pdf.EliteDocument(test_pdf)
        print(f"Pages: {doc2.count_pages()}")
    except Exception as e:
        print(f"Second run error: {e}")

    print("\n--- CHECKING DATABASE ---")
    if os.path.exists(db_path):
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM pdf_ingestion")
        rows = cursor.fetchall()
        for row in rows:
            print(f"DB Record: {row}")
        conn.close()
    else:
        print("Database file NOT found!")

if __name__ == "__main__":
    verify()
