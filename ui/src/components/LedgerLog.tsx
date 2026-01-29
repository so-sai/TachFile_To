import React, { useEffect } from 'react';
import { useLedgerStore } from '../lib/ledgerStore';

const LedgerLog: React.FC = () => {
    const { entries, isLoading, fetchEntries } = useLedgerStore();

    useEffect(() => {
        fetchEntries();
    }, []);

    return (
        <div className="h-full w-full bg-[#050505] text-green-500 font-mono p-4 overflow-hidden flex flex-col">
            <div className="mb-4 border-b border-green-900 pb-2 flex justify-between items-center">
                <h2 className="text-xl font-black italic tracking-tighter">NHẬT KÝ TRÍCH XUẤT (EXTRACTION LEDGER)</h2>
                <button
                    onClick={() => fetchEntries()}
                    className="text-xs bg-green-900/30 hover:bg-green-500 hover:text-black px-2 py-1 border border-green-500 transition-colors"
                >
                    REFRESH_AUDIT_TRAIL
                </button>
            </div>

            <div className="flex-1 overflow-auto custom-scrollbar">
                {isLoading && <div className="animate-pulse">QUERYING_SQLITE_DATABASE...</div>}

                {!isLoading && entries.length === 0 && (
                    <div className="text-gray-600">NO_RECORDS_FOUND_IN_LEDGER_SQLITE</div>
                )}

                <table className="w-full text-left border-collapse">
                    <thead>
                        <tr className="text-xs text-green-700 bg-green-900/10">
                            <th className="p-2 border border-green-900">TIMESTAMP</th>
                            <th className="p-2 border border-green-900">FILE_PATH</th>
                            <th className="p-2 border border-green-900">ENGINE</th>
                            <th className="p-2 border border-green-900">PAGES</th>
                            <th className="p-2 border border-green-900">LATENCY</th>
                            <th className="p-2 border border-green-900">STATUS</th>
                        </tr>
                    </thead>
                    <tbody className="text-[11px]">
                        {entries.map((entry) => (
                            <tr key={entry.id} className="hover:bg-green-500/10 border-b border-green-900/30">
                                <td className="p-2 whitespace-nowrap">{new Date(entry.created_at).toLocaleString()}</td>
                                <td className="p-2 truncate max-w-[300px]" title={entry.source_path}>{entry.source_path}</td>
                                <td className="p-2 text-blue-400">{entry.extraction_engine} v{entry.extraction_version}</td>
                                <td className="p-2">{entry.pages_processed}</td>
                                <td className="p-2 text-yellow-500">{entry.processing_time_ms}ms</td>
                                <td className={`p-2 font-bold ${entry.status === 'success' ? 'text-green-400' : 'text-red-500'}`}>
                                    {entry.status.toUpperCase()}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>

            <div className="mt-4 text-[9px] text-green-900 border-t border-green-900 pt-2 flex justify-between uppercase tracking-widest">
                <span>Ledger Integrity: Verified</span>
                <span>Storage: SQLite (WAL_MODE)</span>
            </div>
        </div>
    );
};

export default LedgerLog;
