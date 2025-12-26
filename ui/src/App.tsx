import { useState, useRef, useEffect, useCallback } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const ENTERPRISE_STYLES = `
  * { box-sizing: border-box; }
  body { margin: 0; font-family: 'Inter', system-ui, sans-serif; font-size: 13px; color: #1F2937; background-color: #F6F7F8; -webkit-font-smoothing: antialiased; font-weight: 450; line-height: 1.45; }
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
  .shadow-sm { box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05); }
  .bg-white { background-color: #FFFFFF; }
  .bg-blue-50 { background-color: #EFF6FF; }
  .bg-gray-50 { background-color: #F9FAFB; }
  .bg-gray-200 { background-color: #E5E7EB; }
  .border { border-style: solid; border-width: 1px; }
  .border-blue-200 { border-color: #BFDBFE; }
  .border-gray-200 { border-color: #E5E7EB; }
  .text-blue-600 { color: #2563EB; }
  .text-blue-700 { color: #1D4ED8; }
  .text-gray-400 { color: #9CA3AF; }
  .text-gray-500 { color: #6B7280; }
  .text-gray-700 { color: #374151; }
  .text-gray-900 { color: #111827; }
  .rounded { border-radius: 4px; }
  .rounded-sm { border-radius: 2px; }
  .rounded-full { border-radius: 9999px; }
  .tracking-wider { letter-spacing: 0.05em; }
  .tracking-widest { letter-spacing: 0.1em; }
  .z-10 { z-index: 10; }
  .h-4 { height: 16px; }
  .w-4 { width: 16px; }
  .w-2 { width: 8px; }
  .h-2 { height: 8px; }
  .w-\[1px\] { width: 1px; }
  .mx-2 { margin-left: 8px; margin-right: 8px; }
  .mb-2 { margin-bottom: 8px; }
  .mb-4 { margin-bottom: 16px; }
  .mt-1 { margin-top: 4px; }
  .px-2 { padding-left: 8px; padding-right: 8px; }
  .py-0.5 { padding-top: 2px; padding-bottom: 2px; }
  .px-3 { padding-left: 12px; padding-right: 12px; }
  .py-1 { padding-top: 4px; padding-bottom: 4px; }
  .px-4 { padding-left: 16px; padding-right: 16px; }
  .animate-pulse { animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: .5; } }
  .enterprise-scroll-container { overflow-y: auto; overflow-x: auto; contain: strict; scrollbar-width: auto; scrollbar-color: #555555 #F0F0F0; }
  .enterprise-scroll-container::-webkit-scrollbar { width: 14px; height: 14px; }
  .enterprise-scroll-container::-webkit-scrollbar-track { background: #F0F0F0; border-left: 1px solid #E5E7EB; }
  .enterprise-scroll-container::-webkit-scrollbar-thumb { background-color: #6B7280; border: 3px solid transparent; background-clip: content-box; border-radius: 0; }
  .enterprise-scroll-container::-webkit-scrollbar-thumb:hover { background-color: #4B5563; }
  .app-header { height: 50px; flex-shrink: 0; border-bottom: 1px solid #E5E7EB; background-color: #FFFFFF; padding: 0 16px; }
  .table-header { height: 32px; background-color: #F9FAFB; border-bottom: 1px solid #E5E7EB; color: #4b5563; font-weight: 600; text-transform: uppercase; font-size: 11px; display: flex; align-items: center; position: sticky; top: 0; z-index: 10; }
  .header-cell { padding: 0 12px; border-right: 1px solid #E5E7EB; display: flex; align-items: center; height: 100%; white-space: nowrap; }
  .table-row { height: 32px; border-bottom: 1px solid #E5E7EB; display: flex; align-items: center; background-color: #FFFFFF; transition: background 0.05s; }
  .table-row:hover { background-color: #F3F4F6; }
  .cell { padding: 0 12px; border-right: 1px solid #F3F4F6; font-size: 13px; color: #374151; height: 100%; display: flex; align-items: center; }
  .drop-overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background-color: rgba(255,255,255,0.6); backdrop-filter: blur(8px); z-index: 9999; display: flex; flex-direction: column; align-items: center; justify-content: center; border: 4px solid #3B82F6; }
  .landing-screen { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; background: radial-gradient(circle at 50% 50%, #FFFFFF 0%, #F3F4F6 100%); padding: 40px; }
  .hero-card { width: 480px; background: white; border-radius: 24px; padding: 48px; box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04); border: 1px solid #E5E7EB; text-align: center; transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); cursor: pointer; }
  .hero-card:hover { transform: translateY(-4px); box-shadow: 0 25px 30px -5px rgba(59, 130, 246, 0.1), 0 15px 15px -5px rgba(59, 130, 246, 0.05); border-color: #3B82F6; }
  .hero-icon { width: 80px; height: 80px; background: #EFF6FF; border-radius: 20px; display: flex; align-items: center; justify-content: center; margin: 0 auto 24px; color: #3B82F6; }
  .hero-title { font-size: 24px; font-weight: 700; color: #111827; margin-bottom: 12px; }
  .hero-subtitle { font-size: 14px; color: #6B7280; line-height: 1.6; margin-bottom: 32px; }
  .upload-btn { background: #2563EB; color: white; padding: 12px 24px; border-radius: 12px; font-weight: 600; display: inline-flex; align-items: center; gap: 8px; transition: background 0.2s; border: none; cursor: pointer; }
  .upload-btn:hover { background: #1D4ED8; }
  .file-types { margin-top: 32px; display: flex; gap: 12px; justify-content: center; }
  .file-badge { font-size: 11px; font-weight: 600; padding: 4px 8px; border-radius: 6px; background: #F3F4F6; color: #4B5563; border: 1px solid #E5E7EB; }
  .skeleton-bg { background: linear-gradient(90deg, #F3F4F6 25%, #E5E7EB 50%, #F3F4F6 75%); background-size: 200% 100%; animation: skeleton-loading 1.5s infinite; }
  @keyframes skeleton-loading { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
`;

function App() {
  const [dataMap, setDataMap] = useState<Record<number, any>>({});
  const [totalRows, setTotalRows] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [loadingMsg, setLoadingMsg] = useState("");
  const [isDragging, setIsDragging] = useState(false);
  const parentRef = useRef<HTMLDivElement>(null);
  const fetchingPages = useRef(new Set<number>());

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

  const processFile = async (path: string) => {
    setIsLoading(true);
    setLoadingMsg("Engine 2025: Đang xử lý...");
    try {
      const count = await invoke<number>("load_excel_file", { filePath: path });
      setTotalRows(count);
      setDataMap({});
      fetchingPages.current.clear();
      setLoadingMsg("");
      fetchPage(0);
    } catch (err) { alert("Lỗi: " + err); }
    finally { setIsLoading(false); }
  };

  const fetchPage = useCallback((page: number) => {
    const PAGE_SIZE = 100;
    invoke<string>("get_window", { offset: page * PAGE_SIZE, limit: PAGE_SIZE })
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

      {/* --- HEADER ĐÃ SỬA: TÁCH BIỆT TÊN VÀ PHIÊN BẢN --- */}
      <header className="app-header flex items-center justify-between bg-white shadow-sm z-10">
        <div className="flex items-center gap-3">
          {/* 1. Tên dự án chuẩn Engineering */}
          <h1 className="font-bold text-xl text-[#111827] tracking-wider font-mono">
            TACHFILE_TO
          </h1>

          {/* 2. Badge Phiên bản tách biệt */}
          <div className="flex items-center">
            <span className="h-4 w-[1px] bg-gray-300 mx-2"></span> {/* Vách ngăn */}
            <span className="text-[9px] font-bold text-blue-700 bg-blue-50 border border-blue-200 rounded px-2 py-0.5 shadow-sm uppercase tracking-tighter">
              BẢN CHUYÊN NGHIỆP | 12.2025
            </span>
          </div>
        </div>

        {/* Thông tin trạng thái bên phải */}
        <div className="text-xs font-medium text-gray-500 bg-gray-50 px-3 py-1 rounded border border-gray-200">
          {isLoading ? (
            <span className="text-blue-600 animate-pulse">{loadingMsg}</span>
          ) : (
            <span>RECORDS: <span className="text-gray-900 font-bold tabular-nums">{totalRows.toLocaleString()}</span></span>
          )}
        </div>
      </header>

      {/* TABLE HEADER (Visible when data loaded) */}
      {totalRows > 0 && (
        <div className="table-header">
          <div className="header-cell" style={{ width: 60 }}>#</div>
          {columns.map(col => (
            <div key={col} className="header-cell" style={{ width: 150 }}>{col}</div>
          ))}
        </div>
      )}

      {/* MAIN VIEW AREA */}
      <div ref={parentRef} className="enterprise-scroll-container w-full bg-white relative" style={{ flex: 1 }}>
        {totalRows === 0 ? (
          // EMPTY STATE (Màn hình chờ chuyên nghiệp)
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <div className="w-16 h-16 mb-4 text-gray-200">
              <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 13h6m-3-3v6m5 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
            </div>
            <p className="text-lg font-medium text-gray-500">Sẵn sàng tiếp nhận dữ liệu</p>
            <p className="text-xs text-gray-400 mt-1 uppercase tracking-widest">Kéo thả file Excel (.xlsx) vào đây để phân tích</p>
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
                  <div className="cell tabular-nums text-gray-400 bg-gray-50" style={{ width: 60 }}>{virtualRow.index + 1}</div>
                  {row ? (
                    columns.map(col => (
                      <div key={col} className="cell truncate" style={{ width: 150 }}>{String(row[col])}</div>
                    ))
                  ) : (
                    <div className="cell w-full">
                      <div className="skeleton-bg h-4 w-2/3 rounded-sm opacity-50"></div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* --- FOOTER ĐÃ SỬA: THÊM KHOẢNG CÁCH (GAP) --- */}
      <div className="h-[28px] bg-[#F9FAFB] border-t border-[#E5E7EB] flex items-center justify-between px-4 text-[10px] text-gray-500 font-mono select-none z-10">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-green-500"></span>
            <span className="font-bold text-gray-700">CORE: RUST 2024</span>
          </div>
          <div className="flex items-center gap-1">
            <span>ENGINE:</span>
            <span className="text-blue-600 font-bold">POLARS 0.44</span>
          </div>
          <div className="hidden sm:block">MEM: OPTIMIZED</div>
        </div>
        <div>
          © 2025 TACHFILE_TO SYSTEM <span className="text-gray-300 mx-2">|</span> BUILD: 2025.12.26
        </div>
      </div>
    </div>
  );
}

export default App;
