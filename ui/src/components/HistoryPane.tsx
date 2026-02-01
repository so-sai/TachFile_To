import React from 'react';
import { useTruthStore } from '../lib/useTruthStore';
import { History, User, ShieldCheck, Fingerprint } from 'lucide-react';

const HistoryPane: React.FC = () => {
    const { auditTrail, isAuditLoading, activeFile } = useTruthStore();

    if (!activeFile) return null;

    return (
        <div className="flex-1 flex flex-col bg-white overflow-hidden border-l border-[#E5E7EB]">
            {/* HEADER */}
            <div className="h-12 flex items-center justify-between px-4 border-b border-[#F3F4F6] bg-slate-50/80">
                <div className="flex items-center gap-2">
                    <History size={14} className="text-slate-400" />
                    <h3 className="text-[11px] font-black text-slate-500 uppercase tracking-widest font-['Space_Grotesk']">
                        Lịch sử hiệu chỉnh (Audit Trail)
                    </h3>
                </div>
                {isAuditLoading && (
                    <div className="text-[9px] font-bold text-blue-500 animate-pulse uppercase">Syncing...</div>
                )}
            </div>

            {/* CONTENT */}
            <div className="flex-1 overflow-auto custom-scrollbar p-0 bg-slate-50/30">
                {auditTrail.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full opacity-30 gap-3 grayscale">
                        <History size={32} />
                        <span className="text-[10px] font-bold uppercase tracking-widest text-center px-8">
                            Chưa có dữ liệu hiệu chỉnh cho hồ sơ này
                        </span>
                    </div>
                ) : (
                    <div className="flex flex-col">
                        {auditTrail.map((snapshot, sIdx) => (
                            <div key={sIdx} className="border-b border-[#F3F4F6] hover:bg-white transition-colors">
                                <div className="p-4 flex flex-col gap-3">
                                    {/* Snapshot Meta */}
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-2">
                                            <div className="p-1.5 bg-blue-50 rounded">
                                                <ShieldCheck size={12} className="text-blue-600" />
                                            </div>
                                            <div className="flex flex-col">
                                                <span className="text-[10px] font-black text-slate-700 uppercase leading-none">
                                                    Snapshot Sealed
                                                </span>
                                                <span className="text-[9px] font-bold text-slate-400 tabular-nums">
                                                    {new Date(snapshot.timestamp).toLocaleString()}
                                                </span>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-1.5 px-2 py-0.5 bg-emerald-50 border border-emerald-100 rounded-sm">
                                            <div className="w-1 h-1 rounded-full bg-emerald-500"></div>
                                            <span className="text-[9px] font-black text-emerald-700 uppercase">
                                                {snapshot.verdict}
                                            </span>
                                        </div>
                                    </div>

                                    {/* Repair Batch */}
                                    <div className="flex flex-col gap-2">
                                        {snapshot.repairs.map((repair, rIdx) => (
                                            <div key={rIdx} className="pl-8 border-l-2 border-slate-100 flex flex-col gap-1.5">
                                                <div className="flex items-center justify-between">
                                                    <div className="flex items-center gap-1.5">
                                                        <User size={10} className="text-slate-400" />
                                                        <span className="text-[9px] font-bold text-slate-500 uppercase">
                                                            {repair.actor}
                                                        </span>
                                                    </div>
                                                    <span className="px-1.5 py-0.5 bg-amber-50 text-amber-700 text-[8px] font-black rounded uppercase border border-amber-100">
                                                        {repair.reason_code}
                                                    </span>
                                                </div>

                                                <div className="bg-white border border-slate-100 rounded p-2 flex flex-col gap-1 shadow-sm">
                                                    <div className="flex items-center gap-2 opacity-50">
                                                        <span className="text-[9px] font-bold text-slate-400 uppercase w-8">Old:</span>
                                                        <span className="text-[10px] font-mono text-slate-600 line-through truncate">
                                                            {JSON.stringify(repair.old_value)}
                                                        </span>
                                                    </div>
                                                    <div className="flex items-center gap-2">
                                                        <span className="text-[9px] font-bold text-amber-600 uppercase w-8">New:</span>
                                                        <span className="text-[11px] font-bold text-[#D97706] truncate">
                                                            {JSON.stringify(repair.new_value)}
                                                        </span>
                                                    </div>
                                                </div>
                                            </div>
                                        ))}
                                    </div>

                                    {/* Integrity Hash */}
                                    <div className="flex items-center gap-2 mt-1 opacity-40 hover:opacity-100 transition-opacity">
                                        <Fingerprint size={10} className="text-slate-400" />
                                        <span className="text-[8px] font-mono text-slate-500 truncate font-bold uppercase tracking-tighter">
                                            HASH: {snapshot.hashes.virtual_truth}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

export default HistoryPane;
