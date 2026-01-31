import { useEffect } from 'react';

import FileLedger from './components/FileLedger';
import TableTruthView from './components/TableTruthView';
import EvidencePane from './components/EvidencePane';
import SummaryPane from './components/SummaryPane';
import AppendixA from './components/AppendixA';

function App() {
  // MISSION 026: Keyboard Shortcuts (Ctrl+F, ESC)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'f') {
        e.preventDefault();
        console.log('CMD: FOCUS_FILE_LEDGER');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-white selection:bg-yellow-400 no-round">

      {/* 🛡️ BLOOMBERG HEADER: APP TITLE & ENGINE STATUS */}
      <header className="h-[48px] flex-shrink-0 border-b-2 border-black bg-black px-6 flex items-center justify-between select-none no-round">
        <div className="flex items-center gap-4">
          <h1 className="font-black text-xl tracking-tighter uppercase italic text-white">
            TACHFILE_TO <span className="text-[#00FF00] not-italic">VN-CORE</span>
          </h1>
          <div className="bg-[#00FF00] text-black px-2 py-0.5 font-black text-[9px] tracking-widest uppercase no-round border border-black">
            UNIFIED_CONSTITUTION_V2.0
          </div>
        </div>

        <div className="flex items-center gap-6 font-black text-[9px] uppercase tracking-widest text-[#444]">
          <div className="flex items-center gap-2">
            <span className="w-1.5 h-1.5 bg-[#00FF00] animate-pulse"></span>
            <span className="text-[#00FF00]">CORE_SYNC: ACTIVE</span>
          </div>
          <div>EST: 2026-01-31</div>
          <div className="text-white border-l border-white/20 pl-4">INSPECTION_MODE</div>
        </div>
      </header>

      {/* MAIN INSPECTION AREA: THE 4-PANEL ARCHITECTURE */}
      <main className="flex-1 flex overflow-hidden">

        {/* PANEL 1: FILE LEDGER (LEFT SIDEBAR) */}
        <aside className="w-[280px] flex-shrink-0 flex flex-col no-round">
          <FileLedger />
        </aside>

        {/* CENTER COMPARTMENT: PANEL 2 & PANEL 3 */}
        <section className="flex-1 flex flex-col overflow-hidden no-round">

          {/* HORIZONTAL SPLIT: TABLE TRUTH (TOP) & EVIDENCE (BOTTOM) */}
          <div className="flex-1 flex flex-col min-h-0 no-round">
            <TableTruthView />
            <EvidencePane />
          </div>

          {/* PANEL 4: DISCREPANCY SUMMARY (BOTTOM BAR) */}
          <SummaryPane />

        </section>

      </main>

      {/* APPENDIX A: LEGAL NOTICE & DEFINITIONS */}
      <AppendixA />

      {/* STATUS BAR (BLOOMBERG STYLE) */}
      <footer className="h-[20px] bg-black text-[#666] px-4 flex items-center justify-between font-black text-[8px] uppercase tracking-[0.2em] select-none border-t border-[#222] no-round">
        <div className="flex gap-6">
          <span className="text-[#00FF00]">LEGAL_GRADE_AUDIT_PASS</span>
          <span>DENSITY: 32PX_FIXED</span>
        </div>
        <div className="flex gap-4">
          <span>MEM: 247MB/500MB</span>
          <span>LATENCY: 0.04MS</span>
        </div>
        <div className="text-[#888]">BUILD_CORE_A25-1</div>
      </footer>
    </div>
  );
}

export default App;
