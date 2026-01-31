import React from 'react';

const AppendixA: React.FC = () => {
    return (
        <div className="bg-gray-200 p-4 border-t-2 border-black select-none no-round">
            <div className="max-w-5xl mx-auto border-4 border-black p-4 bg-white no-round">
                <h3 className="font-black text-[11px] mb-3 underline uppercase tracking-tighter italic">PHỤ LỤC A: BẢN GIẢI NGHĨA PHÁP LÝ (LEGAL DEFINITIONS)</h3>
                <div className="grid grid-cols-2 gap-8 text-[10px] leading-tight tabular-nums">
                    <div className="space-y-3">
                        <p><span className="font-black border-b-2 border-green-600">SẠCH (CLEAN):</span> File khớp hoàn toàn với cấu trúc hiến pháp dữ liệu. Không phát hiện sai lỗi vật lý hoặc luận lý.</p>
                        <p><span className="font-black border-b-2 border-yellow-600">VẤN ĐỀ (TAINTED):</span> Phát hiện mâu thuẫn giữa các nguồn (Cross-source) hoặc độ tin cậy thấp. Cần giám định bằng mắt.</p>
                    </div>
                    <div className="space-y-3">
                        <p><span className="font-black border-b-2 border-red-600">TỪ CHỐI (REJECTED):</span> Vi phạm rào cản an toàn (Mojibake, Dòng ma, Cấu trúc rỗng). Dữ liệu không có giá trị pháp lý.</p>
                        <p><span className="font-black border-b-2 border-black/30">HỢP LỆ (ADMISSIBLE):</span> Giá trị ô khớp chính xác với vệt quét thị giác từ tài liệu gốc.</p>
                    </div>
                </div>
                <div className="mt-4 pt-4 border-t border-gray-100 flex justify-between items-center text-[9px] text-gray-400 font-black italic uppercase tracking-widest">
                    <span>TACHFILETO_LEGAL_NOTICE: THIẾT BỊ GIÁM ĐỊNH DỮ LIỆU ĐỘC LẬP</span>
                    <span>VER: 2.0 (BLOOMBERG_SPEC)</span>
                </div>
            </div>
        </div>
    );
};

export default AppendixA;
