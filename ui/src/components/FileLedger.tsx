import React, { useEffect } from 'react';
import { useTruthStore, FileStatusLabel } from '../lib/useTruthStore';

const StatusBadge: React.FC<{ status: FileStatusLabel }> = ({ status }) => {
    let colorClass = 'bg-gray-100 text-gray-600';
    let label: string = status;

    switch (status) {
        case 'Clean':
            colorClass = 'bg-green-100 text-green-700 font-bold';
            label = 'SẠCH';
            break;
        case 'Tainted':
            colorClass = 'bg-amber-100 text-amber-700 font-bold';
            label = 'CẢNH BÁO';
            break;
        case 'Rejected':
            colorClass = 'bg-red-100 text-red-700 font-bold';
            label = 'TỪ CHỐI';
            break;
    }

    return (
        <span className={`px-2 py-0.5 text-[9px] rounded-full uppercase tracking-wide ${colorClass}`}>
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
            <div className="flex-1 p-4 text-[10px] text-slate-400 uppercase">
                <span className="animate-pulse">Đang tải danh sách...</span>
            </div>
        );
    }

    return (
        <div className="flex-1 flex flex-col bg-white">
            <div className="flex-1 overflow-auto custom-scrollbar">
                {files.length === 0 ? (
                    <div className="p-4 italic text-slate-400 text-xs text-center">
                        Chưa có hồ sơ nào được nạp.
                    </div>
                ) : (
                    files.map((file) => (
                        <div
                            key={file.name}
                            onClick={() => selectFile(file.name)}
                            className={`
                                flex flex-col p-3 border-b border-slate-100 cursor-pointer transition-colors
                                ${activeFile === file.name ? 'bg-blue-50 border-l-4 border-blue-600' : 'hover:bg-slate-50 border-l-4 border-transparent'}
                            `}
                        >
                            <div className="flex justify-between items-start mb-1">
                                <span className={`font-semibold text-xs truncate max-w-[160px] ${activeFile === file.name ? 'text-blue-900' : 'text-slate-700'}`}>
                                    {file.name}
                                </span>
                                <StatusBadge status={file.status} />
                            </div>
                            <div className="text-[9px] text-slate-400 font-mono mt-1">
                                {file.timestamp}
                            </div>
                        </div>
                    ))
                )}
            </div>
        </div>
    );
};

export default FileLedger;
