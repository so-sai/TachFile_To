import React, { useCallback, useState } from 'react';
import { CloudUpload } from 'lucide-react';

export const DropZone: React.FC = () => {
    const [isDragging, setIsDragging] = useState(false);

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(true);
    }, []);

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
    }, []);

    const handleDrop = useCallback((e: React.DragEvent) => {
        e.preventDefault();
        setIsDragging(false);
        // Handle files here in the future
        console.log('Files dropped:', e.dataTransfer.files);
    }, []);

    return (
        <div
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            className={`
                relative h-32 rounded-2xl border-2 border-dashed transition-all duration-300 ease-out cursor-pointer group
                flex flex-col items-center justify-center gap-3
                ${isDragging
                    ? 'border-blue-500 bg-blue-50/50 dark:bg-blue-900/20 scale-[1.02]'
                    : 'border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 hover:border-blue-400 hover:bg-gray-50'
                }
            `}
        >
            <div className={`p-3 rounded-full transition-colors duration-300 ${isDragging ? 'bg-blue-100 text-blue-600' : 'bg-white dark:bg-gray-700 text-gray-400 group-hover:text-blue-500 shadow-sm'}`}>
                <CloudUpload className="w-6 h-6" />
            </div>
            <div className="text-center">
                <p className={`text-sm font-medium transition-colors ${isDragging ? 'text-blue-600' : 'text-gray-600 dark:text-gray-300'}`}>
                    Kéo thả hồ sơ vào đây
                </p>
                <p className="text-xs text-gray-400 mt-1">
                    Hỗ trợ PDF, Excel (Max 50MB) (iOS 14 Style)
                </p>
            </div>
        </div>
    );
};
