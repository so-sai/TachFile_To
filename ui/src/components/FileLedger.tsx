import React, { useEffect } from 'react';
import { useTruthStore, FileStatusLabel } from '../lib/useTruthStore';

const StatusBadge: React.FC<{ status: FileStatusLabel }> = ({ status }) => {
    let colorClass = 'bg-gray-500';
    let label: string = status;

    switch (status) {
        case 'Clean':
            colorClass = 'bg-green-100 text-green-700 border border-green-700';
            label = 'SẠCH';
            break;
        case 'Tainted':
            colorClass = 'bg-yellow-100 text-yellow-700 border border-yellow-700';
            label = 'VẤN ĐỀ';
            break;
        case 'Rejected':
            colorClass = 'bg-red-100 text-red-700 border border-red-700';
            label = 'TỪ CHỐI';
            break;
    }

    return (
        <span className={`px-2 py-0.5 font-black text-[9px] uppercase tracking-tighter no-round ${colorClass}`}>
            {label}
        </span>
    );
};

const FileLedger: React.FC = () => {
    const { files, activeFile, selectFile, fetchFiles, isFilesLoading } = useTruthStore();

    useEffect(() => {
        fetchFiles();
    }, [fetchFiles]);

    if (isFilesLoading) {
        return (
            <div className="flex-1 bg-gray-100 p-4 border-r-4 border-black font-black uppercase text-[10px]">
                <span className="animate-pulse">Loading Ledger...</span>
            </div>
        );
    }

    return (
        <div className="flex-1 flex flex-col bg-gray-100 border-r-4 border-black overflow-hidden no-round">
            <div className="bg-black text-white p-3 font-black text-xs uppercase tracking-widest select-none no-round">
                1. FILE LEDGER [SỔ CÁI HỒ SƠ]
            </div>

            <div className="flex-1 overflow-auto custom-scrollbar no-round">
                {files.length === 0 ? (
                    <div className="p-4 italic text-gray-400 text-xs">Không có hồ sơ.</div>
                ) : (
                    files.map((file) => (
                        <div
                            key={file.name}
                            onClick={() => selectFile(file.name)}
                            className={`
                                flex flex-col p-3 border-b border-gray-300 cursor-pointer transition-none no-round
                                ${activeFile === file.name ? 'bg-white border-l-8 border-black' : 'hover:bg-gray-200'}
                            `}
                        >
                            <div className="flex justify-between items-start mb-1">
                                <span className="font-bold text-[11px] truncate max-w-[180px] uppercase tracking-tight">
                                    {file.name}
                                </span>
                                <StatusBadge status={file.status} />
                            </div>
                            <div className="text-[9px] text-gray-500 font-mono">
                                RECEIVED: {file.timestamp}
                            </div>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};

export default FileLedger;
