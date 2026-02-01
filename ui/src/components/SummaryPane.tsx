import React, { useEffect } from 'react';
import { useTruthStore } from '../lib/useTruthStore';
import { RefreshCw, FileDown, FileJson } from 'lucide-react';

const SummaryPane: React.FC = () => {
    const { summary, refreshSummary, exportAudit } = useTruthStore();

    useEffect(() => {
        refreshSummary();
    }, [refreshSummary]);

    return (
        <div className="bg-white border-t border-slate-200 p-4 flex flex-col gap-4">
            <div className="flex justify-between items-center">
                <h2 className="font-bold text-xs text-slate-500 uppercase tracking-widest flex items-center gap-2">
                    PHÁN QUYẾT CUỐI CÙNG
                </h2>
                <button
                    onClick={() => refreshSummary()}
                    className="text-[10px] text-blue-600 hover:text-blue-800 font-bold flex items-center gap-1 transition-colors"
                >
                    <RefreshCw className="w-3 h-3" />
                    CẬP NHẬT
                </button>
            </div>

            <div className="grid grid-cols-3 gap-4">
                {/* CONSISTENT */}
                <div className="bg-green-50 rounded-lg p-3 flex flex-col items-center border border-green-100 shadow-sm">
                    <span className="text-[10px] text-green-600 font-bold uppercase mb-1">KHỚP (HỢP LỆ)</span>
                    <span className="text-2xl font-black text-green-700 tabular-nums">{summary?.consistent || 0}</span>
                </div>

                {/* INCONSISTENT */}
                <div className="bg-red-50 rounded-lg p-3 flex flex-col items-center border border-red-100 shadow-sm">
                    <span className="text-[10px] text-red-600 font-bold uppercase mb-1">SAI LỆCH (CẦN XỬ LÝ)</span>
                    <span className="text-2xl font-black text-red-700 tabular-nums">{summary?.inconsistent || 0}</span>
                </div>

                {/* REVIEW */}
                <div className="bg-amber-50 rounded-lg p-3 flex flex-col items-center border border-amber-100 shadow-sm">
                    <span className="text-[10px] text-amber-600 font-bold uppercase mb-1">CẦN KIỂM TRA LẠI</span>
                    <span className="text-2xl font-black text-amber-700 tabular-nums">{summary?.requires_review || 0}</span>
                </div>
            </div>

            <div className="flex justify-between items-center pt-2">
                <span className="text-[10px] text-slate-400 font-mono italic">
                    * Dữ liệu được đối chiếu đa nguồn
                </span>
                <div className="flex gap-2">
                    <button
                        onClick={() => exportAudit('md')}
                        className="bg-slate-100 text-slate-700 border border-slate-300 px-3 py-1.5 rounded text-xs font-bold hover:bg-slate-200 transition-colors flex items-center gap-2"
                        title="Xuất biên bản Markdown"
                    >
                        <FileJson className="w-3 h-3" />
                        BIÊN BẢN (MD)
                    </button>
                    <button
                        onClick={() => exportAudit('xlsx')}
                        className="bg-green-600 text-white border border-green-700 px-3 py-1.5 rounded text-xs font-bold hover:bg-green-700 transition-colors flex items-center gap-2 shadow-sm"
                        title="Xuất kết quả Excel"
                    >
                        <FileDown className="w-3 h-3" />
                        KẾT QUẢ (EXCEL)
                    </button>
                </div>
            </div>
        </div>
    );
};

export default SummaryPane;
