# TACHFILETO GUIDE: THE FORENSIC HANDBOOK 🛡️⚖️

Welcome to **TachFileTo V1.0.0**. This is not just a tool; it is your **Forensic Workstation** for verifying the engineering truth of Vietnamese AEC projects.

---

## 🏛️ 1. PHILOSOPHY: WHY THE "VERDICT"?
When you export data to Excel or Markdown, you will see a `TACHFILETO_VERDICT` column.
- **CLEAN**: Every digit in this row has been traced back and validated.
- **SUSPICIOUS**: There is a potential encoding or rounding issue. Check the Evidence Pane.
- **REJECTED**: This data has no origin in the source documents or violates the Iron Truth Contract.

**Remember**: As the Lead Engineer/QS, when you use this data, you are signing off on its forensic integrity.

---

## 🛠️ 2. TACTICAL REPAIR: ENCODING MOJIBAKE
Vietnamese projects often mix Unicode, TCVN3 (ABC), and VNI. TachFileTo detects these, but you are the judge.
- **The Evidence Pane**: When a cell is flagged, look at the Evidence Pane. It shows you the original crop.
- **Dual Interpretation**: The system will suggest both TCVN3 and VNI versions. Select the one that matches the visual crop.
- **Freeze Mode**: Once you select a repair, the system seals that cell. It becomes **Admissible Truth**.

---

## ⚡ 3. PERFORMANCE & USAGE TIPS
- **Mass Ingestion**: You can drag and drop an entire folder of PDF/Excel files. The system will process them in parallel.
- **1-Click Trace**: Click any "Inconsistent" number on the Dashboard to jump directly to its visual source in the PDF.
- **Windows Safety**: If you get a "Permission Denied" error during export, simply close the file in Excel. TachFileTo protects your data from being corrupted by concurrent writes.

---

## 📉 4. AUDIT EXPORT
- **Markdown (.md)**: Your "Legal Dossier". Use this for storage, Git commits, or attaching to official audit reports.
- **Excel (.xlsx)**: Your "Work Tool". It is formatted with frozen headers and numeric types, ready for your next set of calculations.

---
*TachFileTo: Extraction of Truth from Data Chaos.*
© 2026 DT Building Ecosystem.
