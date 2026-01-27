import { useState, useRef, useEffect, useCallback } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import DashboardFounder from './components/DashboardFounder';
import './styles/Dashboard.css';

type TabType = 'dashboard' | 'data';

const ENTERPRISE_STYLES = `
  * { box-sizing: border-box; }
  body { margin: 0; font-family: 'Space Grotesk', 'Inter', system-ui, sans-serif; font-size: 13px; color: #000; background-color: #EEE; -webkit-font-smoothing: antialiased; font-weight: 500; }
  .tabular-nums { font-variant-numeric: tabular-nums; }
  .truncate { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .flex { display: flex; }
  .flex-col { flex-direction: column; }
  .h-full { height: 100%; }
  .w-full { width: 100%; }
  .h-screen { height: 100vh; }
  .w-screen { width: 100vw; }
  .items-center { align-items: center; }
  .justify-between { justify-content: space-between; }
  .justify-center { justify-content: center; }
  .gap-1 { gap: 4px; }
  .gap-2 { gap: 8px; }
  .gap-3 { gap: 12px; }
  .gap-4 { gap: 16px; }
  .gap-6 { gap: 24px; }
  .font-mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; }
  .shadow-hard { box-shadow: 4px 4px 0px 0px rgba(0,0,0,1); }
  .bg-white { background-color: #FFFFFF; }
  .bg-blue-50 { background-color: #EFF6FF; }
  .bg-gray-50 { background-color: #F9FAFB; }
  .bg-gray-200 { background-color: #E5E7EB; }
  .border { border-style: solid; border-width: 1px; }
  .border-2 { border-width: 2px; }
  .border-4 { border-width: 4px; }
  .border-black { border-color: #000; }
  .text-blue-600 { color: #2563EB; }
  .text-red-600 { color: #EE0000; }
  .rounded-none { border-radius: 0px; }
  .tracking-tighter { letter-spacing: -0.05em; }
  .z-10 { z-index: 10; }
  .h-4 { height: 16px; }
  .w-4 { width: 16px; }
  .w-2 { width: 8px; }
  .h-2 { height: 8px; }
  .app-header { height: 60px; flex-shrink: 0; border-bottom: 4px solid #000; background-color: #FFF; padding: 0 24px; }
  .table-header { height: 32px; background-color: #000; color: #FFF; font-weight: 800; text-transform: uppercase; font-size: 11px; display: flex; align-items: center; position: sticky; top: 0; z-index: 10; }
  .header-cell { padding: 0 12px; border-right: 1px solid #333; display: flex; align-items: center; height: 100%; white-space: nowrap; }
  .table-row { height: 32px; border-bottom: 1px solid #DDD; display: flex; align-items: center; background-color: #FFFFFF; }
  .table-row:hover { background-color: #FFFF00; color: #000; }
  .cell { padding: 0 12px; border-right: 1px solid #EEE; font-size: 13px; color: #000; height: 100%; display: flex; align-items: center; }
  .drop-overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(255, 255, 0, 0.9); z-index: 9999; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 10px solid #000; }
  .skeleton-bg { background: #EEE; position: relative; overflow: hidden; }
  .skeleton-bg::after { content: ""; position: absolute; top: 0; right: 0; bottom: 0; left: 0; transform: translateX(-100%); background: linear-gradient(90deg, transparent, rgba(255,255,255,0.5), transparent); animation: shimmer 1.5s infinite; }
  @keyframes shimmer { 100% { transform: translateX(100%); } }
  .tab-btn { padding: 12px 24px; font-weight: 900; font-size: 14px; border: none; cursor: pointer; transition: none; text-transform: uppercase; border-right: 4px solid #000; }
  .tab-btn.active { background: #FFFF00; color: #000; }
  .tab-btn:not(.active) { background: #FFF; color: #000; }
  .tab-btn:not(.active):hover { background: #000; color: #FFF; }
`;

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('dashboard');
  const [dataMap, setDataMap] = useState<Record<number, any>>({});
  const [totalRows, setTotalRows] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMsg, setLoadingMsg] = useState("");
  const [isDragging, setIsDragging] = useState(false);
  const [hasData, setHasData] = useState(false);
  const parentRef = useRef<HTMLDivElement>(null);
  const fetchingPages = useRef(new Set<number>());

  // V3.1: Sheet Selector State
  const [filePath, setFilePath] = useState("");
  const [sheets, setSheets] = useState<string[]>([]);
  const [currentSheet, setCurrentSheet] = useState("");

  // Matrix Logs Sequence
  const matrixLogs = [
    "> KÍCH HOẠT LÕI THÉP (IRON CORE)...",
    "> ĐANG ĐỌC DỮ LIỆU EXCEL BIẾN THỂ TỰ ĐỘNG...",
    "> POLARS 0.52: ĐANG XỬ LÝ DỮ LIỆU...",
    "> CHUẨN HÓA THUẬT NGỮ QS VIỆT NAM...",
    "> ĐANG TRÍCH XUẤT PHÁN QUYẾT TỪNG DÒNG...",
  ];

  useEffect(() => {
    const setupDragDrop = async () => {
      const unlistenHover = await listen('tauri://drag-over', () => setIsDragging(true));
      const unlistenCancel = await listen('tauri://drag-leave', () => setIsDragging(false));
      const unlistenDrop = await listen('tauri://drag-drop', async (e: any) => {
        setIsDragging(false);
        const files = e.payload.paths as string[];
        if (files?.[0]) processFile(files[0]);
      });
      return () => {
        unlistenHover();
        unlistenCancel();
        unlistenDrop();
      };
    };
    setupDragDrop();
  }, []);

  // V3.2: Intelligent Router (Hotfix for Alpha RC1)
  const [docContent, setDocContent] = useState<string>("");
  const [isDocMode, setIsDocMode] = useState(false);

  const processFile = async (path: string) => {
    setIsLoading(true);
    setFilePath(path);
    setLoadingMsg("ĐANG PHÂN TÍCH FILE...");

    const extension = path.split('.').pop()?.toLowerCase();

    // ROUTING LOGIC
    if (extension === 'xlsx' || extension === 'xls') {
      await processExcel(path);
    } else if (extension === 'pdf' || extension === 'docx' || extension === 'md') {
      await processDocument(path);
    } else {
      alert("Định dạng chưa được hỗ trợ trong Alpha RC1 (chỉ PDF, DOCX, XLSX, MD)");
      setIsLoading(false);
    }
  };

  const processExcel = async (path: string) => {
    setIsDocMode(false);
    // Legacy Excel Logic
    let logIdx = 0;
    const logInterval = setInterval(() => {
      if (logIdx < matrixLogs.length) {
        setLoadingMsg(matrixLogs[logIdx]);
        logIdx++;
      }
    }, 300);

    try {
      const response = await invoke<any>("excel_load_file", { path: path });
      setTotalRows(response.total_rows);
      setSheets(response.sheets || []);
      setCurrentSheet(response.current_sheet || "");
      setDataMap({});
      fetchingPages.current.clear();
      setHasData(true);
      setActiveTab('dashboard');
    } catch (err) {
      alert("Lỗi đọc Excel: " + err);
    } finally {
      clearInterval(logInterval);
      setLoadingMsg("");
      setIsLoading(false);
    }
  };

  const processDocument = async (path: string) => {
    setIsDocMode(true);
    setLoadingMsg("KÍCH HOẠT FAST LANE (NO-GIL)...");
    try {
      // Call Unified Extractor (Rust -> Iron Core -> Python 3.14t)
      const result = await invoke<any>("extract_file", { path: path });

      // Result is generic JSON. For Alpha RC1, we prioritize showing the "content" or "raw_text".
      let content = "";
      if (typeof result === 'string') {
        content = result;
      } else if (result.content) {
        content = result.content;
      } else {
        content = JSON.stringify(result, null, 2);
      }

      setDocContent(content);
      setHasData(true);
      setActiveTab('data'); // Switch to Data tab to show preview
    } catch (err) {
      console.error(err);
      alert("Lỗi Fast Lane: " + err);
    } finally {
      setIsLoading(false);
    }
  };

  // V3.1: Hàm đổi Sheet
  const changeSheet = async (sheetName: string) => {
    if (!filePath) return;
    setIsLoading(true);
    setCurrentSheet(sheetName);
    try {
      const response = await invoke<any>("excel_select_sheet", { path: filePath, sheetName });
      setTotalRows(response.total_rows);
      setDataMap({});
      fetchingPages.current.clear();
    } catch (err) {
      alert("Lỗi đọc sheet: " + err);
    } finally {
      setIsLoading(false);
    }
  };

  const fetchPage = useCallback((page: number) => {
    const PAGE_SIZE = 100;
    invoke<string>("excel_get_window", { start: page * PAGE_SIZE, end: (page + 1) * PAGE_SIZE })
      .then((jsonStr) => {
        try {
          const rows = JSON.parse(jsonStr);
          setDataMap(prev => {
            const next = { ...prev };
            rows.forEach((row: any, i: number) => {
              next[(page * PAGE_SIZE) + i] = row;
            });
            return next;
          });
        } catch (e) {
          console.error("JSON parse error:", e, jsonStr);
        }
      });
  }, []);

  const rowVirtualizer = useVirtualizer({
    count: totalRows,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 20,
  });

  useEffect(() => {
    if (totalRows === 0) return;
    const virtualItems = rowVirtualizer.getVirtualItems();
    if (!virtualItems.length) return;

    const PAGE_SIZE = 100;
    const startPage = Math.floor(virtualItems[0].index / PAGE_SIZE);
    const endPage = Math.floor(virtualItems[virtualItems.length - 1].index / PAGE_SIZE);

    for (let page = startPage; page <= endPage; page++) {
      if (!fetchingPages.current.has(page)) {
        fetchingPages.current.add(page);
        fetchPage(page);
      }
    }
  }, [rowVirtualizer.getVirtualItems(), totalRows, fetchPage]);

  const firstRowKey = Object.keys(dataMap)[0];
  const firstRowData = firstRowKey !== undefined ? dataMap[Number(firstRowKey)] : null;
  const columns = firstRowData ? Object.keys(firstRowData) : [];

  return (
    <div className="flex flex-col h-screen overflow-hidden relative font-sans text-[#1F2937]">
      <style>{ENTERPRISE_STYLES}</style>

      {/* GLASS OVERLAY DRAG DROP */}
      {isDragging && (
        <div className="drop-overlay">
          <div className="text-2xl font-bold text-[#2563EB] mb-2">TACHFILE_TO</div>
          <div className="text-gray-600">Thả file dữ liệu vào vùng trọng lực</div>
        </div>
      )}

      {/* --- HEADER V2.5: FOUNDER'S COMMAND CENTER --- */}
      <header className="app-header flex items-center justify-between z-10">
        <div className="flex items-center h-full">
          {/* 1. Tên dự án - Brutalist Style */}
          <h1 className="font-black text-2xl text-black tracking-tighter mr-6 uppercase">
            TACHFILE_TO
          </h1>

          {/* 2. TAB BUTTONS - Hard Edges */}
          <div className="flex h-full border-l-4 border-black">
            <button
              onClick={() => setActiveTab('dashboard')}
              className={`tab-btn ${activeTab === 'dashboard' ? 'active' : ''}`}
            >
              🚦 DASHBOARD
            </button>
            <button
              onClick={() => setActiveTab('data')}
              className={`tab-btn ${activeTab === 'data' ? 'active' : ''}`}
            >
              📋 DATA GRID
            </button>
          </div>
        </div>

        {/* 3. TRẠNG THÁI HỆ THỐNG & SHEET SELECTOR */}
        <div className="flex items-center gap-4">
          {/* V3.1: Sheet Selector Dropdown */}
          {hasData && sheets.length > 1 && (
            <div className="flex items-center gap-2 bg-gray-200 px-3 py-1">
              <span className="text-xs font-black uppercase">SHEET:</span>
              <select
                value={currentSheet}
                onChange={(e) => changeSheet(e.target.value)}
                className="border-2 border-black bg-white text-black font-bold text-sm px-2 py-1 cursor-pointer"
              >
                {sheets.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </div>
          )}

          <div className="bg-black text-[#00FF00] font-black px-4 py-2 text-xs uppercase tracking-widest border-2 border-black">
            IRON CORE: VALIDATED
          </div>
          <div className="text-sm font-black text-black">
            {isLoading ? (
              <span className="animate-pulse">{loadingMsg || "ĐANG XỬ LÝ..."}</span>
            ) : (
              <span>RECORDS: <span className="text-blue-600">{totalRows.toLocaleString()}</span></span>
            )}
          </div>
        </div>
      </header>

      {/* MAIN CONTENT AREA */}
      {activeTab === 'dashboard' ? (
        // DASHBOARD TAB
        <div className="flex-1 overflow-auto bg-[#EEE]">
          {hasData && !isDocMode ? (
            <DashboardFounder />
          ) : hasData && isDocMode ? (
            <div className="flex flex-col items-center justify-center h-full p-8">
              <div className="brutal-box w-full max-w-4xl bg-white p-6 border-4 border-black shadow-hard">
                <h2 className="text-2xl font-black mb-4 uppercase bg-yellow-400 inline-block px-2">DOCUMENT DIGITIZED (FAST LANE)</h2>
                <p className="mb-4 font-bold">Dữ liệu đã được trích xuất qua Engine No-GIL. Hiện tại Dashboard chỉ hỗ trợ Excel.</p>
                <button onClick={() => setActiveTab('data')} className="bg-black text-white px-6 py-3 font-bold hover:bg-gray-800 transition-colors">
                  XEM NỘI DUNG CHI TIẾT &rarr;
                </button>
              </div>
            </div>
          ) : (
            // EMPTY STATE - Brutalist Style
            <div className="flex flex-col items-center justify-center h-full p-8">
              <div className="border-[8px] border-black p-12 bg-white shadow-[16px_16px_0px_0px_rgba(0,0,0,1)] max-w-2xl text-center">
                <h2 className="text-6xl font-black mb-6 tracking-tighter uppercase underline decoration-8 decoration-yellow-400">
                  READY FOR JUDGMENT
                </h2>
                <p className="text-2xl font-bold mb-10 text-gray-700">Kéo thả file Excel/PDF để bắt đầu phân tích</p>
                <div className="grid grid-cols-3 gap-6 text-xs font-black uppercase tracking-widest">
                  <div className="p-4 border-4 border-black bg-green-500">100% TIẾNG VIỆT</div>
                  <div className="p-4 border-4 border-black bg-blue-500 text-white">IRON CORE V2.5</div>
                  <div className="p-4 border-4 border-black bg-yellow-500">NO-GIL READY</div>
                </div>
              </div>
            </div>
          )}
        </div>
      ) : (
        // DATA VIEW TAB
        <>
          {isDocMode ? (
            // DOCUMENT OVERVIEW MODE
            <div className="w-full h-full p-6 overflow-auto bg-gray-50 font-mono text-sm whitespace-pre-wrap">
              {docContent || "Không có nội dung text."}
            </div>
          ) : (
            // EXCEL GRID MODE
            <>
              {/* TABLE HEADER */}
              {totalRows > 0 && (
                <div className="table-header">
                  <div className="header-cell" style={{ width: 60 }}>ID</div>
                  {columns.map(col => (
                    <div key={col} className="header-cell" style={{ width: 180 }}>{col}</div>
                  ))}
                </div>
              )}

              <div ref={parentRef} className="enterprise-scroll-container w-full bg-white relative" style={{ flex: 1 }}>
                {totalRows === 0 ? (
                  <div className="flex flex-col items-center justify-center h-full text-black font-black uppercase">
                    <p className="text-4xl italic">Chưa có dữ liệu chi tiết</p>
                  </div>
                ) : (
                  <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
                    {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                      const row = dataMap[virtualRow.index];
                      return (
                        <div
                          key={virtualRow.index}
                          className="table-row"
                          style={{
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            width: '100%',
                            transform: `translateY(${virtualRow.start}px)`
                          }}
                        >
                          <div className="cell tabular-nums font-black bg-gray-100 border-r-2 border-black" style={{ width: 60 }}>{virtualRow.index + 1}</div>
                          {row ? (
                            columns.map(col => (
                              <div key={col} className="cell truncate font-medium" style={{ width: 180 }}>{String(row[col])}</div>
                            ))
                          ) : (
                            <div className="cell w-full bg-gray-50 h-full"></div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </>
          )}
        </>
      )}

      {/* --- FOOTER V2.5 --- */}
      <footer className="h-[32px] bg-black text-white flex items-center justify-between px-6 text-[10px] font-black uppercase tracking-widest select-none z-10 border-t-2 border-white/20">
        <div className="flex items-center gap-8">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 bg-[#00FF00]"></span>
            <span>SYSTEM STANDBY</span>
          </div>
          <div className="text-yellow-400">
            ENGINE: POLARS 0.52 | CALAMINE 0.32
          </div>
        </div>
        <div>
          TACHFILE_TO VER 2.50 [IRON CORE]
        </div>
      </footer>
    </div>
  );
}

export default App;
