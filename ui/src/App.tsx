import React, { useEffect, useState } from 'react';
import {
  FileText,
  Search,
  Plus,
  CheckCircle2,
  AlertCircle,
  Clock,
  FileSpreadsheet,
  Activity,
  Layers,
  Database,
  FileUp
} from 'lucide-react';
import { useTruthStore } from './lib/useTruthStore';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const App: React.FC = () => {
  const {
    files,
    activeFile,
    cells,
    isFilesLoading,
    isTableLoading,
    fetchFiles,
    selectFile
  } = useTruthStore();

  const [metadataVisible, setMetadataVisible] = useState(false);

  useEffect(() => {
    fetchFiles();

    const unlisten = listen('tauri://drag-drop', async (event: any) => {
      const paths = event.payload.paths;
      if (paths && paths.length > 0) {
        handleLoadFile(paths[0]);
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleOpenFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Construction Data', extensions: ['xlsx', 'csv', 'pdf'] }]
    });
    if (selected && typeof selected === 'string') {
      handleLoadFile(selected);
    }
  };

  const handleLoadFile = async (path: string) => {
    try {
      await invoke('excel_load_file', { path });
      await fetchFiles();
      const fileName = path.split(/[\\/]/).pop() || path;
      await selectFile(fileName);
    } catch (err) {
      console.error("Critical Load Failure:", err);
    }
  };

  return (
    <div className="h-screen w-screen bg-[#F9FAFB] text-[#111827] font-sans overflow-hidden grid grid-cols-[280px_1fr] grid-rows-[56px_1fr_40px]">

      {/* 🚀 HEADER (COCKPIT - 56px) */}
      <header className="col-span-2 bg-white border-b border-[#E5E7EB] flex items-center justify-between px-6 z-20 select-none shadow-[0_1px_2px_rgba(0,0,0,0.03)]">
        <div className="flex items-center gap-5">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 bg-[#111827] flex items-center justify-center text-white rounded-[2px] shadow-sm">
              <Database size={18} strokeWidth={2.5} />
            </div>
            <div className="flex flex-col">
              <span className="text-[15px] font-black tracking-tighter leading-none text-[#111827]">SO-SAI</span>
              <span className="text-[10px] font-bold text-blue-600 tracking-[0.2em] uppercase mt-1">TACHFILETO</span>
            </div>
          </div>
          <div className="h-8 w-px bg-[#E5E7EB] mx-2" />
          <div className="flex items-center gap-2.5 bg-slate-50 px-3 py-1.5 rounded-sm border border-[#E5E7EB]">
            <Activity size={14} className="text-emerald-500" />
            <span className="text-[11px] font-bold uppercase tracking-wider text-slate-500">System Ready</span>
          </div>
        </div>

        <div className="flex items-center gap-6">
          <div className="flex flex-col items-end">
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest leading-none">Status</span>
            <span className="text-[12px] font-bold text-emerald-600 mt-1 uppercase">1.0.1.Stable</span>
          </div>
          <button
            onClick={() => setMetadataVisible(!metadataVisible)}
            className="w-10 h-10 flex items-center justify-center rounded-sm hover:bg-slate-50 border border-transparent hover:border-slate-200 transition-all text-slate-400 hover:text-blue-600"
          >
            <Layers size={20} />
          </button>
        </div>
      </header>

      {/* 📁 SIDEBAR (REGISTRY - 280px) */}
      <aside className="bg-white border-r border-[#E5E7EB] flex flex-col z-10">
        <div className="h-14 flex items-center justify-between px-5 border-b border-[#F3F4F6] bg-slate-50/50">
          <h2 className="text-[11px] font-black text-slate-400 uppercase tracking-[0.15em]">Sổ Cái Hồ Sơ</h2>
          <div className="px-2 py-0.5 bg-white border border-[#E5E7EB] rounded-sm text-[10px] font-black text-slate-600 tabular-nums">
            {files.length}
          </div>
        </div>

        <div className="flex-1 overflow-y-auto custom-scrollbar p-3 space-y-1.5 bg-[#FCFDFE]">
          {isFilesLoading ? (
            Array(6).fill(0).map((_, i) => (
              <div key={i} className="h-14 bg-slate-50 border border-slate-100 rounded-sm animate-pulse" />
            ))
          ) : files.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center p-8 text-center opacity-30">
              <Database size={40} className="text-slate-200 mb-4" />
              <p className="text-[11px] font-bold leading-relaxed uppercase tracking-widest text-slate-400">Trống Rỗng</p>
            </div>
          ) : (
            files.map((file) => (
              <button
                key={file.name}
                onClick={() => selectFile(file.name)}
                className={`w-full text-left p-3.5 rounded-sm transition-all border outline-none ${activeFile === file.name
                  ? 'bg-blue-50 border-blue-200 shadow-sm'
                  : 'bg-white border-transparent hover:bg-slate-50 hover:border-slate-100'
                  }`}
              >
                <div className="flex items-start gap-3.5">
                  <div className={`mt-1 h-5 w-5 flex items-center justify-center shrink-0 rounded-[2px] ${activeFile === file.name ? 'bg-blue-600 text-white' : 'bg-slate-100 text-slate-400'
                    }`}>
                    {file.name.toLowerCase().endsWith('.pdf') ? <FileText size={12} strokeWidth={3} /> : <FileSpreadsheet size={12} strokeWidth={3} />}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className={`text-[13px] font-bold truncate leading-tight ${activeFile === file.name ? 'text-blue-900' : 'text-slate-700'
                      }`}>
                      {file.name}
                    </p>
                    <div className="flex items-center gap-2 mt-1.5 opacity-60">
                      <Clock size={10} strokeWidth={2.5} className="text-slate-400" />
                      <span className="text-[10px] font-bold text-slate-500 tabular-nums uppercase">{file.timestamp}</span>
                    </div>
                  </div>
                </div>
              </button>
            ))
          )}
        </div>
      </aside>

      {/* 🖥️ MAIN WORKSPACE (CANVAS) */}
      <main className="relative overflow-hidden flex flex-col bg-[#F9FAFB]">
        {!activeFile ? (
          <div className="flex-1 flex flex-col items-center justify-center p-12">
            <div
              onClick={handleOpenFile}
              className="w-full max-w-2xl aspect-[16/9] border-2 border-dashed border-[#D1D5DB] bg-white rounded-sm flex flex-col items-center justify-center group cursor-pointer hover:border-blue-400 hover:bg-blue-50/10 transition-all active:scale-[0.995]"
            >
              <div className="w-24 h-24 bg-slate-50 rounded-full flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-500 border border-slate-100 shadow-inner">
                <FileUp size={40} className="text-slate-400 group-hover:text-blue-500 transition-colors" />
              </div>
              <h2 className="text-[20px] font-black text-[#111827] tracking-tighter mb-2">NẠP HỒ SƠ ĐỐI SOÁT</h2>
              <p className="text-[14px] text-slate-500 mb-8 max-w-md text-center font-medium">
                Kết nối dữ liệu pháp lý của bạn. Hỗ trợ Excel (.xlsx) và Hồ sơ PDF.
              </p>
              <div className="flex gap-4">
                <div className="px-5 py-2.5 bg-white border border-[#E5E7EB] rounded-sm text-[11px] font-black text-slate-600 shadow-sm flex items-center gap-2 uppercase tracking-widest">
                  <Database size={14} /> Local Drive
                </div>
                <div className="px-5 py-2.5 bg-slate-900 text-white rounded-sm text-[11px] font-black shadow-lg flex items-center gap-2 uppercase tracking-widest hover:bg-slate-800 transition-colors">
                  <Plus size={16} /> Chọn File
                </div>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* AUDIT STATUS BAR */}
            <div className="h-14 bg-white border-b border-[#E5E7EB] px-8 flex items-center justify-between shrink-0">
              <div className="flex items-center gap-4">
                <div className="h-1.5 w-1.5 rounded-full bg-blue-600 animate-ping" />
                <h3 className="text-[13px] font-black text-slate-800 uppercase tracking-tight flex items-center gap-2">
                  <Search size={16} className="text-blue-600" />
                  Kết Quả Giám Định: <span className="text-blue-600 ml-1">{activeFile}</span>
                </h3>
              </div>
              <div className="flex items-center gap-3">
                <div className="px-3 py-1.5 bg-emerald-50 text-emerald-700 border border-emerald-100 rounded-sm text-[11px] font-black flex items-center gap-2 uppercase">
                  <CheckCircle2 size={14} /> 157 Hợp Lệ
                </div>
                <div className="px-3 py-1.5 bg-amber-50 text-amber-700 border border-amber-100 rounded-sm text-[11px] font-black flex items-center gap-2 uppercase">
                  <AlertCircle size={14} /> 2 Sai Lệch
                </div>
              </div>
            </div>

            {/* HIGH-PRECISION DATA GRID */}
            <div className="flex-1 overflow-auto bg-[#F9FAFB] p-6">
              {isTableLoading ? (
                <div className="space-y-4 max-w-7xl mx-auto">
                  {Array(12).fill(0).map((_, i) => (
                    <div key={i} className="h-10 bg-white border border-slate-100 rounded-sm animate-pulse shadow-sm" />
                  ))}
                </div>
              ) : (
                <div className="max-w-7xl mx-auto bg-white border border-[#E5E7EB] shadow-[0_4px_12px_rgba(0,0,0,0.02)] rounded-sm overflow-hidden flex flex-col min-h-full">
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-[13px] border-collapse bg-white">
                      <thead className="sticky top-0 bg-[#F9FAFB] border-b border-[#E5E7EB] z-10">
                        <tr>
                          <th className="px-6 py-4 font-black text-slate-400 w-16 text-center border-r border-[#F3F4F6] uppercase tabular-nums">ID</th>
                          <th className="px-6 py-4 font-black text-slate-400 uppercase tracking-wider">Trình Diện Dữ Liệu Gốc</th>
                          <th className="px-6 py-4 font-black text-slate-400 text-right w-40 uppercase tracking-wider">Trạng Thái</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-[#F3F4F6]">
                        {cells.length === 0 ? (
                          <tr>
                            <td colSpan={3} className="px-6 py-24 text-center">
                              <div className="flex flex-col items-center gap-4 opacity-40">
                                <Activity size={32} className="text-slate-300" />
                                <p className="text-[12px] font-bold text-slate-500 uppercase tracking-widest">Đang tải hạ tầng dữ liệu...</p>
                              </div>
                            </td>
                          </tr>
                        ) : (
                          cells.map((cell) => (
                            <tr key={cell.cell_id} className="hover:bg-slate-50 transition-all group">
                              <td className="px-6 py-3.5 text-slate-400 text-center font-bold border-r border-[#F3F4F6] tabular-nums whitespace-nowrap">
                                {cell.cell_id.split('_').pop()}
                              </td>
                              <td className="px-6 py-3.5 text-slate-900 font-medium font-mono leading-relaxed bg-white/50 group-hover:bg-transparent transition-colors">
                                {cell.value || cell.source_text}
                              </td>
                              <td className="px-6 py-3.5 text-right whitespace-nowrap">
                                <span className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-[2px] text-[11px] font-black border uppercase shadow-sm ${cell.verdict === 'Admissible'
                                  ? 'bg-emerald-50 text-emerald-700 border-emerald-100'
                                  : 'bg-amber-50 text-amber-700 border-amber-100'
                                  }`}>
                                  {cell.verdict === 'Admissible' ? 'Hợp Lệ' : 'Nghi Vấn'}
                                </span>
                              </td>
                            </tr>
                          ))
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </main>

      {/* 🚀 ACTION BAR (MISSION CONTROL - 80px) */}
      <footer className="h-10 border-t border-gray-100 bg-[#F9FAFB]/50 backdrop-blur-md flex items-center justify-between px-6">
        <div className="flex items-center gap-3">
          <span className="text-[10px] font-bold tracking-tight text-gray-900 uppercase">
            © 2026 SO-SAI
          </span>
          <span className="text-gray-300">|</span>
          <a
            href="https://github.com/so-sai"
            className="text-[9px] font-mono text-gray-500 hover:text-blue-600 transition-colors uppercase tracking-widest"
          >
            github.com/so-sai
          </a>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1.5">
            <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></div>
            <span className="text-[10px] font-medium text-gray-500 uppercase">System Ready</span>
          </div>
        </div>
      </footer>

      {/* 🛡️ METADATA OVERLAY (DEBUG/SYSTEM INFOS) */}
      {metadataVisible && (
        <div
          className="fixed inset-0 bg-slate-900/60 backdrop-blur-md z-[100] flex items-center justify-center p-8 transition-all duration-300"
          onClick={() => setMetadataVisible(false)}
        >
          <div
            className="bg-white rounded-sm shadow-2xl w-full max-w-lg overflow-hidden border border-slate-200 flex flex-col scale-100"
            onClick={e => e.stopPropagation()}
          >
            <div className="p-8 bg-slate-950 text-white flex items-center justify-between">
              <div>
                <h2 className="text-xl font-black tracking-tight uppercase">Thông Số Hạ Tầng</h2>
                <div className="flex items-center gap-3 mt-2 text-slate-400">
                  <div className="h-px w-8 bg-blue-600" />
                  <p className="text-[10px] uppercase tracking-[0.3em] font-black">Forensic Logic Core</p>
                </div>
              </div>
              <Activity className="text-blue-500 animate-pulse" size={32} />
            </div>

            <div className="p-10 space-y-8 bg-white">
              <div className="grid grid-cols-2 gap-y-10 gap-x-12">
                <div>
                  <label className="text-[10px] font-black text-slate-300 uppercase tracking-[0.2em] block mb-2 px-1 border-l-2 border-slate-200">Legal Entity</label>
                  <p className="text-[14px] font-black text-slate-800 tracking-tight">SO-SAI : VN</p>
                </div>
                <div>
                  <label className="text-[10px] font-black text-slate-300 uppercase tracking-[0.2em] block mb-2 px-1 border-l-2 border-slate-200">Protocol</label>
                  <p className="text-[14px] font-black text-slate-800 tabular-nums">V1.0.1.Stable</p>
                </div>
                <div>
                  <label className="text-[10px] font-black text-slate-300 uppercase tracking-[0.2em] block mb-2 px-1 border-l-2 border-slate-200">Stability</label>
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                    <p className="text-[13px] font-black text-slate-800 uppercase italic tracking-tighter">Verified Release</p>
                  </div>
                </div>
                <div>
                  <label className="text-[10px] font-black text-slate-300 uppercase tracking-[0.2em] block mb-2 px-1 border-l-2 border-slate-200">Binary ID</label>
                  <p className="text-[11px] font-mono font-bold text-slate-400 tracking-tighter">COM.SO-SAI.TS</p>
                </div>
              </div>

              <div className="pt-10 mt-10 border-t border-slate-100">
                <button
                  onClick={() => setMetadataVisible(false)}
                  className="w-full py-4 bg-slate-900 text-white text-[11px] font-black uppercase tracking-[0.3em] rounded-sm hover:bg-blue-700 transition-all shadow-lg active:scale-[0.98]"
                >
                  Xác Nhận & Thoát
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default App;
