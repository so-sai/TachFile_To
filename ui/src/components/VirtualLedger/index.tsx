// ui/src/components/VirtualLedger/index.tsx
import React, { useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { tableFromIPC } from 'apache-arrow';
import {
    useReactTable, getCoreRowModel, flexRender, ColumnDef, SortingState, getSortedRowModel, Header
} from '@tanstack/react-table';
import { useVirtualizer } from '@tanstack/react-virtual';
import { clsx } from 'clsx';
import { ArrowUpDown, RefreshCw, Database } from 'lucide-react';
import { LineItem } from '../../data/mockLedgerData';

// Rust Interface Mapping
interface LedgerEntry {
    stt: number;
    code: string;
    name: string;
    unit: string;
    quantity: number;
    price: number;
    vat_rate: number;
}

// Map Rust data to Internal UI Model
const mapToLineItem = (entry: any): LineItem => {
    // Column Names match Rust excel_engine.rs V2.1
    const stt = entry["STT"];
    const total = entry["THÀNH TIỀN"];
    const statusRaw = entry["TRẠNG THÁI"] || 'valid';

    // Convert status to internal format
    let status: 'valid' | 'warning' | 'error' = 'valid';
    if (statusRaw === 'Từ chối') status = 'error';
    if (statusRaw === 'Chờ xử lý') status = 'warning';

    return {
        id: `row_${stt}`,
        stt: stt,
        code: entry["MÃ HIỆU"],
        name: entry["DIỄN GIẢI CÔNG VIỆC"],
        nameStd: entry["DIỄN GIẢI CHUẨN"] || entry["DIỄN GIẢI CÔNG VIỆC"],
        category: entry["PHÂN LOẠI"] || 'khác',
        unit: entry["DVT"],
        quantity: entry["KHỐI LƯỢNG"],
        price: entry["ĐƠN GIÁ"],
        total: total,
        status: status,
        notes: ''
    };
};

// Resizer Component
const Resizer = ({ header }: { header: Header<LineItem, unknown> }) => {
    if (header.column.getCanResize() === false) return null;
    const handleMouseDown = (e: React.MouseEvent | React.TouchEvent) => {
        header.getResizeHandler()(e);
    };
    return (
        <div
            onMouseDown={handleMouseDown}
            onTouchStart={handleMouseDown}
            className={clsx(
                "absolute top-0 right-0 h-full w-4 cursor-col-resize touch-none select-none z-20 flex justify-center group",
                header.column.getIsResizing() ? "bg-blue-100/50 dark:bg-blue-900/30" : ""
            )}
            style={{ transform: 'translateX(50%)' }}
        >
            <div className={clsx(
                "h-full w-[2px] transition-colors",
                header.column.getIsResizing() ? "bg-blue-700 dark:bg-blue-500" : "bg-transparent group-hover:bg-blue-300 dark:group-hover:bg-blue-700"
            )} />
        </div>
    );
};

// Status Badge V1.1: Matrix / High Contrast
const StatusBadge = ({ status }: { status: string }) => {
    const styles = {
        valid: "bg-green-100 text-green-900 border-green-800 dark:bg-green-900/40 dark:text-green-400 dark:border-green-700",
        warning: "bg-yellow-100 text-yellow-900 border-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-400 dark:border-yellow-700",
        error: "bg-red-100 text-red-900 border-red-800 dark:bg-red-900/40 dark:text-red-400 dark:border-red-700",
    };
    const labels: Record<string, string> = { valid: "HỢP LỆ", warning: "⚠ CẦN KIỂM", error: "✗ SAI SỐ" };
    const s = status as keyof typeof styles;
    return (
        <span className={clsx("inline-flex items-center px-1.5 py-0.5 font-black text-[9px] border leading-none uppercase", styles[s])}>
            {labels[s] || status.toUpperCase()}
        </span>
    );
};

export const VirtualLedger = () => {
    const [data, setData] = useState<LineItem[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [loadTime, setLoadTime] = useState<number | null>(null);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);
    const [sorting, setSorting] = useState<SortingState>([]);
    const parentRef = useRef<HTMLDivElement>(null);

    const loadRealData = async () => {
        if (isLoading) return;
        setIsLoading(true);
        setErrorMessage(null);
        const start = performance.now();
        try {
            console.log("🚀 [Frontend] Requesting fetch_ledger_data (Arrow IPC)...");
            const buffer = await invoke<number[]>('fetch_ledger_data');

            // 1. Parse Arrow IPC Buffer
            const table = tableFromIPC(new Uint8Array(buffer));
            const entries = table.toArray();

            // SAFE LOGGING (Rule: Never log raw 100k rows)
            console.log(`📥 [Frontend] Received Arrow data. Row Count: ${entries?.length || 0}`);

            if (!entries || entries.length === 0) {
                console.warn("⚠️ [Frontend] Received empty dataset from Rust");
                setData([]);
                return;
            }

            const mapped = entries.map(mapToLineItem);
            setData(mapped);
            const end = performance.now();
            setLoadTime(end - start);
        } catch (e) {
            const errTyped = e as any;
            console.error("❌ [Frontend] IPC Fetch Failed:", errTyped);
            setErrorMessage(errTyped.message || String(errTyped));
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        loadRealData();
    }, []);

    const columns = useMemo<ColumnDef<LineItem>[]>(() => [
        { accessorKey: 'stt', header: 'STT', size: 50, cell: info => <span className="text-gray-500 font-bold font-mono text-[11px]">{info.getValue<number>()}</span> },
        { accessorKey: 'code', header: 'MÃ HIỆU', size: 100, cell: info => <span className="font-bold text-gray-900 font-mono tracking-tighter">{info.getValue<string>()}</span> },
        { accessorKey: 'name', header: 'DIỄN GIẢI GỐC', size: 220, cell: info => <span className="text-gray-600 text-[11px] leading-tight line-clamp-1 italic">{info.getValue<string>()}</span> },
        {
            accessorKey: 'nameStd',
            header: 'DIỄN GIẢI CHUẨN',
            size: 220,
            cell: info => <span className="font-bold text-blue-700 uppercase text-[11px] leading-tight line-clamp-1">{info.getValue<string>()}</span>
        },
        {
            accessorKey: 'category',
            header: 'PHÂN LOẠI',
            size: 100,
            cell: info => (
                <span className={clsx(
                    "px-1.5 py-0.5 rounded text-[10px] font-extrabold uppercase",
                    info.getValue<string>() === 'vật tư' ? "bg-orange-100 text-orange-700" :
                        info.getValue<string>() === 'nhân công' ? "bg-blue-100 text-blue-700" :
                            info.getValue<string>() === 'logistics' ? "bg-green-100 text-green-700" :
                                "bg-gray-100 text-gray-600"
                )}>
                    {info.getValue<string>()}
                </span>
            )
        },
        { accessorKey: 'unit', header: 'ĐVT', size: 50, cell: info => <span className="text-gray-600 font-bold uppercase text-[10px]">{info.getValue<string>()}</span> },
        { accessorKey: 'quantity', header: 'KHỐI LƯỢNG', size: 90, cell: info => <div className="text-right font-bold font-mono text-gray-900">{info.getValue<number>().toLocaleString('vi-VN', { minimumFractionDigits: 2 })}</div> },
        { accessorKey: 'price', header: 'ĐƠN GIÁ', size: 110, cell: info => <div className="text-right font-mono text-gray-800">{info.getValue<number>().toLocaleString('vi-VN')}</div> },
        { accessorKey: 'total', header: 'THÀNH TIỀN', size: 130, cell: info => <div className="text-right font-mono font-black text-blue-900">{info.getValue<number>().toLocaleString('vi-VN')}</div> },
        { accessorKey: 'status', header: 'TT', size: 80, cell: info => <StatusBadge status={info.getValue<string>()} /> }
    ], []);

    const table = useReactTable({
        data, columns, state: { sorting }, onSortingChange: setSorting, columnResizeMode: 'onChange',
        getCoreRowModel: getCoreRowModel(), getSortedRowModel: getSortedRowModel(),
    });

    const { rows } = table.getRowModel();
    const rowVirtualizer = useVirtualizer({ count: rows.length, getScrollElement: () => parentRef.current, estimateSize: () => 44, overscan: 20 });

    return (
        <div className="flex flex-col h-full w-full bg-white dark:bg-zinc-950">
            {/* TOOLBAR V1.1: High Contrast, Matrix Buttons */}
            <div className="px-8 py-3 border-b-2 border-gray-300 dark:border-zinc-800 bg-gray-100 dark:bg-zinc-900/80 flex justify-between items-center shrink-0 transition-colors">
                <div className="text-[10px] font-black text-gray-900 dark:text-gray-200 flex items-center gap-6">
                    <span className="uppercase tracking-[0.2em]">DỮ LIỆU DỰ ÁN: <b>{rows.length.toLocaleString()}</b> DÒNG</span>
                    {loadTime && <span className="text-[10px] text-green-800 dark:text-green-500 bg-green-200 dark:bg-green-900/50 px-3 py-1 border border-green-800 font-black tracking-tighter">LATENCY: {loadTime.toFixed(1)}ms</span>}
                </div>
                <div className="flex gap-4">
                    <button
                        onClick={loadRealData}
                        disabled={isLoading}
                        className={clsx(
                            "px-6 py-2 text-white font-black text-[10px] tracking-widest rounded-sm shadow-sm transition-colors flex items-center gap-2 border-2",
                            isLoading ? "bg-gray-400 border-gray-500 cursor-not-allowed" : "bg-blue-700 border-blue-900 hover:bg-blue-800"
                        )}
                    >
                        {isLoading ? <RefreshCw className="animate-spin w-4 h-4" /> : <Database className="w-4 h-4" />}
                        {isLoading ? "ĐANG ĐỒNG BỘ..." : "QUÉT LẠI DỮ LIỆU (F5)"}
                    </button>
                    <button className="px-6 py-2 bg-white dark:bg-zinc-800 border-2 border-gray-300 dark:border-zinc-700 text-gray-900 dark:text-white font-black text-[10px] tracking-widest rounded-sm hover:bg-gray-50 uppercase">+ THÊM CÔNG VIỆC</button>
                </div>
            </div>

            {/* TABLE CONTAINER V1.2: Grok Scrollbar (Global CSS in App.css) */}
            <div
                ref={parentRef}
                className="flex-1 overflow-auto relative w-full bg-white dark:bg-zinc-950 enterprise-scroll-container"
            >

                <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>

                    {/* LOADING OVERLAY V1.1 */}
                    {isLoading && (
                        <div className="absolute inset-0 z-50 bg-white/60 dark:bg-zinc-950/60 flex items-center justify-center backdrop-blur-[1px]">
                            <div className="flex items-center gap-4 p-8 bg-white dark:bg-zinc-900 border-4 border-gray-900 dark:border-white shadow-[12px_12px_0_0_rgba(0,0,0,0.1)]">
                                <RefreshCw className="w-12 h-12 text-blue-800 dark:text-blue-500 animate-spin" strokeWidth={3} />
                                <div className="space-y-1">
                                    <h3 className="text-2xl font-black text-gray-900 dark:text-white uppercase tracking-tighter">SYNCHRONIZING...</h3>
                                    <p className="text-[10px] font-black text-gray-500 font-mono tracking-[0.3em]">ZERO-COPY ARROW v2.0</p>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* ERROR / EMPTY STATES */}
                    {!isLoading && errorMessage && (
                        <div className="absolute inset-x-0 top-20 flex flex-col items-center justify-center p-12 text-center z-40">
                            <div className="bg-red-50 dark:bg-red-900/20 border-4 border-red-700 p-8 max-w-md shadow-[8px_8px_0_0_rgba(185,28,28,0.2)]">
                                <h3 className="text-xl font-black text-red-700 uppercase mb-2">CRITICAL IPC ERROR</h3>
                                <p className="text-xs font-bold text-gray-700 dark:text-red-200 font-mono mb-6 bg-red-100 dark:bg-red-900/40 p-3 border border-red-200 overflow-hidden break-words">{errorMessage}</p>
                                <button onClick={loadRealData} className="w-full py-3 bg-red-700 text-white font-black uppercase text-xs tracking-widest hover:bg-red-800 transition-colors">RESET IPC HANDSHAKE</button>
                            </div>
                        </div>
                    )}

                    {!isLoading && !errorMessage && rows.length === 0 && (
                        <div className="absolute inset-0 flex flex-col items-center justify-center text-gray-400 bg-gray-50/50 dark:bg-zinc-900/20">
                            <div className="p-10 border-4 border-dashed border-gray-300 dark:border-zinc-800 flex flex-col items-center">
                                <Database size={48} className="opacity-20 mb-4" />
                                <h3 className="text-xl font-black uppercase tracking-[0.2em] opacity-30">NO DATA DETECTED</h3>
                                <p className="text-[10px] font-bold mt-2 font-mono uppercase">Waiting for Python Worker Signal...</p>
                            </div>
                        </div>
                    )}

                    <table className="w-full text-left border-collapse table-fixed bg-white dark:bg-zinc-950" style={{ width: table.getTotalSize() }}>
                        <thead className="sticky top-0 z-10 bg-gray-100 dark:bg-zinc-900">
                            {table.getHeaderGroups().map(hg => (
                                <tr key={hg.id} className="h-12 border-b-2 border-gray-300 dark:border-zinc-800">
                                    {hg.headers.map(header => (
                                        <th
                                            key={header.id}
                                            style={{ width: header.getSize() }}
                                            className="px-4 py-3 text-[10px] font-black text-gray-600 dark:text-gray-400 uppercase tracking-widest border-r border-gray-300 dark:border-zinc-800 bg-gray-100 dark:bg-zinc-900 relative transition-colors"
                                        >
                                            <div className="flex items-center gap-2 cursor-pointer" onClick={header.column.getToggleSortingHandler()}>
                                                {flexRender(header.column.columnDef.header, header.getContext())}
                                                {{ asc: <ArrowUpDown size={12} className="text-blue-700" />, desc: <ArrowUpDown size={12} className="text-blue-700 rotate-180" /> }[header.column.getIsSorted() as string] ?? null}
                                            </div>
                                            <Resizer header={header} />
                                        </th>
                                    ))}
                                </tr>
                            ))}
                        </thead>
                        <tbody className="divide-y-0">
                            {rowVirtualizer.getVirtualItems().map(virtualRow => {
                                const row = rows[virtualRow.index];
                                return (
                                    <tr
                                        key={row.id}
                                        className="h-[44px] absolute w-full top-0 left-0 hover:bg-gray-100 dark:hover:bg-zinc-800 border-b border-gray-200 dark:border-zinc-800 transition-none"
                                        style={{ transform: `translateY(${virtualRow.start}px)` }}
                                    >
                                        {row.getVisibleCells().map(cell => (
                                            <td
                                                key={cell.id}
                                                style={{ width: cell.column.getSize() }}
                                                className="px-4 py-2 border-r border-gray-300 dark:border-zinc-800/50 last:border-r-0 truncate text-sm"
                                            >
                                                <div className="truncate text-gray-900 dark:text-gray-100 leading-tight">
                                                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                                </div>
                                            </td>
                                        ))}
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
};
