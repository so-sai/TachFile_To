import React, { useEffect } from 'react';
import { useTruthStore } from '../lib/useTruthStore';

const SummaryPane: React.FC = () => {
    const { summary, refreshSummary } = useTruthStore();

    useEffect(() => {
        refreshSummary();
    }, [refreshSummary]);

    return (
        <div className="bg-black text-[#00FF00] p-4 flex flex-col gap-4 border-t-4 border-white select-none no-round">
            <div className="flex justify-between items-center">
                <h2 className="font-black text-xs tracking-widest uppercase">4. DISCREPANCY SUMMARY [TỔNG HỢP SAI LỆCH]</h2>
                <button
                    onClick={() => refreshSummary()}
                    className="text-[9px] bg-green-900/30 border border-[#00FF00] px-2 py-1 hover:bg-[#00FF00] hover:text-black transition-none no-round font-black"
                >
                    REFRESH_STATS [F5]
                </button>
            </div>

            <div className="grid grid-cols-3 gap-0 border border-[#00FF00]/30 no-round">
                <div className="border-r border-[#00FF00]/30 p-3 flex flex-col items-center">
                    <span className="text-[9px] text-gray-500 mb-1 font-black">KHỚP (CONSISTENT)</span>
                    <span className="text-3xl font-black tabular-nums">{summary?.consistent || 0}</span>
                </div>
                <div className="border-r border-[#00FF00]/30 p-3 flex flex-col items-center bg-red-900/10">
                    <span className="text-[9px] text-red-500 mb-1 font-black">LỆCH (INCONSISTENT)</span>
                    <span className="text-3xl font-black tabular-nums text-red-500">{summary?.inconsistent || 0}</span>
                </div>
                <div className="p-3 flex flex-col items-center bg-yellow-900/10">
                    <span className="text-[9px] text-yellow-500 mb-1 font-black">XÉT DUYỆT (REVIEW)</span>
                    <span className="text-3xl font-black tabular-nums text-yellow-500">{summary?.requires_review || 0}</span>
                </div>
            </div>
            <div className="text-[9px] text-gray-600 font-mono italic text-right uppercase tracking-widest">
                VERDICT_ENGINE: CROSS_SOURCE_CONTRADICTION_ENABLED
            </div>
        </div>
    );
};

export default SummaryPane;
