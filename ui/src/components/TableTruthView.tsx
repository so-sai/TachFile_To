import React, { useRef } from 'react';
import { useTruthStore } from '../lib/useTruthStore';
import { useVirtualizer } from '@tanstack/react-virtual';

const TableTruthView: React.FC = () => {
    const { cells, activeFile, selectedCellId, selectCell, isTableLoading } = useTruthStore();
    const parentRef = useRef<HTMLDivElement>(null);

    // 🛡️ VIRTUALIZATION ENGINE
    const rowVirtualizer = useVirtualizer({
        count: cells.length / 4, // Assuming 4 columns as per logic
        getScrollElement: () => parentRef.current,
        estimateSize: () => 36, // Relaxed row height
        overscan: 10,
    });

    if (!activeFile) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 select-none">
                <div className="text-slate-300 font-bold text-lg uppercase tracking-wider">
                    Chưa chọn hồ sơ
                </div>
            </div>
        );
    }

    if (isTableLoading) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center bg-white font-bold uppercase text-xs text-slate-400">
                <span className="animate-pulse">ĐANG TRUY XUẤT DỮ LIỆU...</span>
            </div>
        );
    }

    const columnCount = 4;

    return (
        <div className="flex-1 flex flex-col bg-white overflow-hidden">
            {/* HEADER (Sticky) */}
            <div className="grid grid-cols-[60px_1fr_120px_200px] bg-slate-100 border-b border-slate-200 font-bold text-[10px] uppercase tracking-wider select-none text-slate-600">
                <div className="p-3 border-r border-slate-200 flex items-center justify-center">STT</div>
                <div className="p-3 border-r border-slate-200 flex items-center">GIÁ TRỊ</div>
                <div className="p-3 border-r border-slate-200 flex items-center">TRẠNG THÁI</div>
                <div className="p-3 flex items-center">GHI CHÚ</div>
            </div>

            {/* VIRTUALIZED BODY */}
            <div
                ref={parentRef}
                className="flex-1 overflow-auto custom-scrollbar"
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

                        const rowSelected = rowCells.some(c => c.cell_id === selectedCellId);
                        const hasLoi = rowCells.some(c => c.verdict === 'Inadmissible');

                        return (
                            <div
                                key={virtualRow.key}
                                onClick={() => {
                                    if (rowCells[1]) selectCell(rowCells[1].cell_id);
                                }}
                                className={`
                                    absolute top-0 left-0 w-full grid grid-cols-[60px_1fr_120px_200px] border-b border-slate-50 cursor-pointer text-sm
                                    ${rowSelected ? 'bg-blue-50/50 ring-1 ring-inset ring-blue-200 z-10' : 'hover:bg-slate-50'}
                                    ${hasLoi ? 'text-red-600 bg-red-50/30' : 'text-slate-700'}
                                `}
                                style={{
                                    height: `36px`,
                                    transform: `translateY(${virtualRow.start}px)`,
                                }}
                            >
                                <div className="px-3 py-2 border-r border-slate-100 flex items-center justify-center bg-slate-50/50 text-xs text-slate-500 font-mono">
                                    {rowIdx + 1}
                                </div>
                                <div className={`px-3 py-2 border-r border-slate-100 flex items-center font-mono overflow-hidden truncate`}>
                                    {rowCells[1]?.value || <span className="text-slate-300 italic">NULL</span>}
                                </div>
                                <div className="px-3 py-2 border-r border-slate-100 flex items-center">
                                    <span className={`
                                        px-2 py-0.5 text-[9px] font-bold rounded-full uppercase tracking-wide
                                        ${rowCells[1]?.verdict === 'Admissible' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}
                                    `}>
                                        {rowCells[1]?.verdict === 'Admissible' ? 'HỢP LỆ' : 'SAI LỆCH'}
                                    </span>
                                </div>
                                <div className="px-3 py-2 italic text-[11px] text-slate-400 flex items-center overflow-hidden truncate">
                                    {rowCells[1]?.reason || ''}
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
