import React, { useRef } from 'react';
import { useTruthStore } from '../lib/useTruthStore';
import { useVirtualizer } from '@tanstack/react-virtual';

const TableTruthView: React.FC = () => {
    const { cells, activeFile, selectedCellId, selectCell, isTableLoading } = useTruthStore();
    const parentRef = useRef<HTMLDivElement>(null);

    // 🛡️ VIRTUALIZATION ENGINE (Mission 026)
    const rowVirtualizer = useVirtualizer({
        count: cells.length / 4, // Assuming 4 columns as per TableTruth grid mapping logic in ui_bridge.rs
        getScrollElement: () => parentRef.current,
        estimateSize: () => 32, // Strict 32px row height
        overscan: 10,
    });

    if (!activeFile) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center bg-gray-50 border-r-4 border-black select-none">
                <div className="text-gray-400 font-black text-4xl italic opacity-30 uppercase tracking-tighter">
                    Chờ tiếp nhận hồ sơ
                </div>
            </div>
        );
    }

    if (isTableLoading) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center bg-white border-r-4 border-black font-black uppercase text-xs">
                <span className="animate-pulse">ĐANG TRUY XUẤT HIẾN PHÁP DỮ LIỆU...</span>
            </div>
        );
    }

    // Logic to group cells into rows (Assuming 4 columns: ID, GIÁ TRỊ, TRẠNG THÁI, LÝ DO)
    const columnCount = 4;

    return (
        <div className="flex-1 flex flex-col bg-white border-r-4 border-black overflow-hidden no-round">
            <div className="bg-black text-white p-3 font-black text-xs uppercase tracking-widest flex justify-between items-center select-none no-round">
                <span>2. TABLE TRUTH VIEW [BẢN GỐC SỰ THẬT]</span>
                <span className="text-[10px] text-yellow-400 font-mono">{activeFile}</span>
            </div>

            {/* HEADER (Sticky) */}
            <div className="grid grid-cols-[60px_1fr_120px_200px] bg-gray-200 border-b-2 border-black font-black text-[10px] uppercase tracking-wider select-none no-round">
                <div className="p-2 border-r border-black flex items-center justify-center">ID</div>
                <div className="p-2 border-r border-black flex items-center">GIÁ TRỊ (TRUTH)</div>
                <div className="p-2 border-r border-black flex items-center">TRẠNG THÁI</div>
                <div className="p-2 flex items-center">LÝ DO (BẰNG CHỨNG)</div>
            </div>

            {/* VIRTUALIZED BODY */}
            <div
                ref={parentRef}
                className="flex-1 overflow-auto custom-scrollbar no-round"
                style={{ height: '100%' }}
            >
                <div
                    style={{
                        height: `${rowVirtualizer.getTotalSize()}px`,
                        width: '100%',
                        position: 'relative',
                    }}
                >
                    {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                        const rowIdx = virtualRow.index;
                        const rowCells = cells.slice(rowIdx * columnCount, (rowIdx + 1) * columnCount);

                        // Heuristic: Use the row's first cell (STT/ID) or verdict to color the row
                        // In ui_bridge, we mapped grid as Flat list, but virtualization needs rows
                        // Let's assume the Row mapping logic in components handles this

                        const rowSelected = rowCells.some(c => c.cell_id === selectedCellId);
                        const hasLoi = rowCells.some(c => c.verdict === 'Inadmissible');

                        return (
                            <div
                                key={virtualRow.key}
                                onClick={() => {
                                    if (rowCells[1]) selectCell(rowCells[1].cell_id); // Select the value cell
                                }}
                                className={`
                                    absolute top-0 left-0 w-full grid grid-cols-[60px_1fr_120px_200px] border-b border-gray-300 cursor-pointer tabular-nums
                                    ${rowSelected ? 'bg-yellow-100 ring-2 ring-inset ring-yellow-400 z-10' : 'hover:bg-gray-50'}
                                    ${hasLoi ? 'bg-red-50 text-red-700' : ''}
                                `}
                                style={{
                                    height: `32px`,
                                    transform: `translateY(${virtualRow.start}px)`,
                                }}
                            >
                                <div className="p-2 border-r border-gray-300 flex items-center justify-center font-black bg-gray-50/50 text-[11px]">
                                    {rowIdx + 1}
                                </div>
                                <div className={`p-2 border-r border-gray-300 flex items-center font-bold font-mono text-xs overflow-hidden truncate`}>
                                    {rowCells[1]?.value || <span className="text-gray-300 italic opacity-30">NULL</span>}
                                </div>
                                <div className="p-2 border-r border-gray-300 flex items-center">
                                    <span className={`
                                        px-2 py-0.5 font-black text-[9px] uppercase tracking-wide
                                        ${rowCells[1]?.verdict === 'Admissible' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700 border border-red-700'}
                                    `}>
                                        {rowCells[1]?.verdict === 'Admissible' ? 'HỢP LỆ' : 'KHÔNG HỢP LỆ'}
                                    </span>
                                </div>
                                <div className="p-2 italic text-[10px] text-gray-500 flex items-center overflow-hidden truncate">
                                    {rowCells[1]?.reason || 'SẠCH'}
                                </div>
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
};

export default TableTruthView;
