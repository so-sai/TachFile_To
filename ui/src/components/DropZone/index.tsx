import React, { useState, useCallback, useRef } from 'react';
import { Upload, FileText, FileSpreadsheet, X } from 'lucide-react';
import { clsx } from 'clsx';

// --- TYPES (Chuẩn bị cho Schema v0.1) ---
type FileStatus = 'idle' | 'scanning' | 'ready' | 'error';

interface IngestionFile {
    id: string;
    file: File;
    status: FileStatus;
    progress: number;
    message?: string;
}

// --- CONFIG ---
const MAX_SIZE = 50 * 1024 * 1024; // 50MB
const ALLOWED_EXT = ['.pdf', '.xlsx', '.xls', '.docx'];

export const DropZone = () => {
    const [isDragging, setIsDragging] = useState(false);
    const [files, setFiles] = useState<IngestionFile[]>([]);
    const fileInputRef = useRef<HTMLInputElement>(null);

    // --- HANDLERS ---
    const handleDrag = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === 'dragenter' || e.type === 'dragover') {
            setIsDragging(true);
        } else if (e.type === 'dragleave') {
            setIsDragging(false);
        }
    }, []);

    const validateFile = (file: File): string | null => {
        // 1. Check Extension
        const ext = '.' + file.name.split('.').pop()?.toLowerCase();
        if (!ALLOWED_EXT.includes(ext)) return "Định dạng không hỗ trợ (Chỉ nhận PDF, Excel, Word)";
        // 2. Check Size
        if (file.size > MAX_SIZE) return "File quá lớn (>50MB)";
        return null;
    };

    // MOCK: Giả lập gọi Iron Core (Sẽ thay bằng Tauri Command thật)
    const mockScanning = async (fileId: string) => {
        // Stage 1: Uploading
        setFiles(prev => prev.map(f => f.id === fileId ? { ...f, status: 'scanning', progress: 10, message: 'Đang nạp...' } : f));
        await new Promise(r => setTimeout(r, 300));

        // Stage 2: Iron Core Processing (Giả lập)
        setFiles(prev => prev.map(f => f.id === fileId ? { ...f, progress: 50, message: 'Iron Core: Phân tích cấu trúc...' } : f));
        await new Promise(r => setTimeout(r, 500));

        // Stage 3: Done
        setFiles(prev => prev.map(f => f.id === fileId ? { ...f, status: 'ready', progress: 100, message: 'Sẵn sàng' } : f));
    };

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        setIsDragging(false);

        if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            processFiles(Array.from(e.dataTransfer.files));
        }
    }, []);

    const processFiles = (newFiles: File[]) => {
        const ingestionFiles: IngestionFile[] = newFiles.map(file => ({
            id: crypto.randomUUID(),
            file,
            status: 'idle',
            progress: 0
        }));

        setFiles(prev => [...prev, ...ingestionFiles]);

        // Tự động trigger scan từng file
        ingestionFiles.forEach(f => {
            const error = validateFile(f.file);
            if (error) {
                setFiles(prev => prev.map(item => item.id === f.id ? { ...item, status: 'error', message: error } : item));
            } else {
                mockScanning(f.id);
            }
        });
    };

    const removeFile = (id: string) => {
        setFiles(prev => prev.filter(f => f.id !== id));
    };

    // --- RENDER HELPERS ---
    const getIcon = (fileName: string) => {
        if (fileName.endsWith('.xlsx') || fileName.endsWith('.xls')) return <FileSpreadsheet className="text-green-600" />;
        return <FileText className="text-blue-600" />;
    };

    return (
        <div className="flex flex-col h-full w-full p-6 gap-6 bg-gray-50 dark:bg-zinc-900/50">
            {/* DROP AREA */}
            <div
                onDragEnter={handleDrag}
                onDragLeave={handleDrag}
                onDragOver={handleDrag}
                onDrop={handleDrop}
                onClick={() => fileInputRef.current?.click()}
                className={clsx(
                    "flex-shrink-0 h-48 border-2 border-dashed rounded-lg flex flex-col items-center justify-center cursor-pointer transition-all duration-200",
                    isDragging
                        ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20 scale-[1.01] shadow-lg"
                        : "border-gray-300 dark:border-zinc-700 bg-white dark:bg-zinc-900 hover:border-blue-400 hover:bg-gray-50 dark:hover:bg-zinc-800"
                )}
            >
                <input
                    type="file"
                    multiple
                    className="hidden"
                    ref={fileInputRef}
                    accept=".pdf,.xlsx,.xls,.docx"
                    onChange={(e) => e.target.files && processFiles(Array.from(e.target.files))}
                />
                <div className="p-4 bg-blue-100 dark:bg-blue-900/50 rounded-full mb-3">
                    <Upload className="w-8 h-8 text-blue-600 dark:text-blue-400" />
                </div>
                <p className="text-sm font-medium text-gray-900 dark:text-gray-200">
                    Kéo thả hồ sơ vào đây hoặc <span className="text-blue-600 dark:text-blue-400">Click để chọn</span>
                </p>
                <p className="text-xs text-gray-500 mt-1">Hỗ trợ: PDF, Excel, Word (Max 50MB)</p>
            </div>

            {/* FILE LIST */}
            <div className="flex-1 overflow-y-auto space-y-3">
                {files.length === 0 && (
                    <div className="h-full flex items-center justify-center text-gray-400 text-sm italic">
                        Chưa có tài liệu nào được nạp.
                    </div>
                )}

                {files.map(file => (
                    <div key={file.id} className="group bg-white dark:bg-zinc-900 border border-gray-200 dark:border-zinc-700 p-3 rounded-md shadow-sm flex items-center gap-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
                        <div className="w-10 h-10 flex items-center justify-center bg-gray-100 dark:bg-zinc-800 rounded">
                            {getIcon(file.file.name)}
                        </div>

                        <div className="flex-1 min-w-0">
                            <div className="flex justify-between mb-1">
                                <span className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">{file.file.name}</span>
                                <span className={clsx("text-xs font-bold uppercase", {
                                    'text-gray-500': file.status === 'idle',
                                    'text-blue-600': file.status === 'scanning',
                                    'text-green-600': file.status === 'ready',
                                    'text-red-600': file.status === 'error',
                                })}>{file.status === 'scanning' ? `${file.progress}%` : file.status}</span>
                            </div>

                            {/* Progress Bar */}
                            <div className="h-1.5 w-full bg-gray-100 dark:bg-zinc-800 rounded-full overflow-hidden">
                                <div
                                    className={clsx("h-full transition-all duration-300", {
                                        'bg-blue-500': file.status === 'scanning',
                                        'bg-green-500': file.status === 'ready',
                                        'bg-red-500': file.status === 'error',
                                        'bg-gray-300': file.status === 'idle'
                                    })}
                                    style={{ width: `${file.progress}%` }}
                                />
                            </div>

                            <p className="text-[10px] text-gray-500 mt-1 truncate">{file.message}</p>
                        </div>

                        <button
                            onClick={(e) => { e.stopPropagation(); removeFile(file.id); }}
                            className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                        >
                            <X size={16} />
                        </button>
                    </div>
                ))}
            </div>
        </div>
    );
};
