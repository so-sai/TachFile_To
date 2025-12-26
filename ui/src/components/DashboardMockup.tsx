import React from 'react';

// --- TRUTH CONTRACT: MOCK DATA ---
const MOCK_TRUTH = {
    project: "CHUNG CƯ TACHFILETO - ĐỢT 4",
    status: "CRITICAL", // RED ALERT
    last_update: "2025-12-26 12:00 PM",
    deviation: "+12.4%",
    financials: {
        contract: "19.0B",
        paid: "12.4B",
        profit: "-240M",
        unapproved: "450M"
    },
    risks: [
        { id: 1, item: "Thép D12", diff: "+5.2t", reason: "Vượt thiết kế", level: "ĐỎ" },
        { id: 2, item: "Cát vàng", diff: "+60k/m3", reason: "Sai đơn giá", level: "VÀNG" },
        { id: 3, item: "Bê tông móng", diff: "+120m3", reason: "Lệch hiện trường", level: "ĐỎ" }
    ],
    actions: [
        "Ký phụ lục thép D12 (QS đang chờ)",
        "Gửi biên bản nghiệm thu móng A1-A5",
        "Kiểm tra lại đơn giá cát vàng kỳ này"
    ]
};

const DashboardMockup: React.FC = () => {
    return (
        <div className="min-h-screen bg-gray-100 p-6 font-mono text-gray-900">
            {/* HEADER: ĐÈN TÍN HIỆU TỔNG */}
            <header className="mb-8 flex items-center justify-between border-b-4 border-gray-900 pb-4 bg-white p-4 shadow-lg">
                <div>
                    <h1 className="text-2xl font-black">{MOCK_TRUTH.project}</h1>
                    <p className="text-sm text-gray-500">DỮ LIỆU ĐỊNH TÍNH (DETERMINISTIC) - {MOCK_TRUTH.last_update}</p>
                </div>
                <div className="flex flex-col items-end">
                    <span className="text-6xl font-black text-red-600 animate-pulse">
                        {MOCK_TRUTH.status}
                    </span>
                    <span className="text-lg font-bold">NGUY CƠ LỖ CAO</span>
                </div>
            </header>

            {/* GRID 2x2: BỐN KHỐI SỐNG CÒN */}
            <div className="grid grid-cols-2 gap-6">

                {/* KHỐI 1: TỔNG QUAN RỦI RO */}
                <section className="border-4 border-gray-900 bg-white p-6 shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
                    <h2 className="mb-4 bg-gray-900 p-2 text-xl font-bold text-white uppercase">1. Tổng Quan Rủi Ro</h2>
                    <div className="space-y-4">
                        <div className="flex justify-between border-b-2 border-dashed border-gray-300 pb-2">
                            <span className="text-lg">TỔNG LỆCH:</span>
                            <span className="text-2xl font-black text-red-600">{MOCK_TRUTH.deviation}</span>
                        </div>
                        <div className="flex justify-between border-b-2 border-dashed border-gray-300 pb-2">
                            <span className="text-lg">PHÁT SINH CHƯA DUYỆT:</span>
                            <span className="text-xl font-bold">{MOCK_TRUTH.financials.unapproved}</span>
                        </div>
                        <p className="mt-4 text-sm bg-red-100 p-2 border-l-4 border-red-600 italic">
                            "Lệch vượt ngưỡng 10%. Cần rà soát lại BOQ gốc ngay lập tức."
                        </p>
                    </div>
                </section>

                {/* KHỐI 2: TOP SAI LỆCH (DEVIATIONS) */}
                <section className="border-4 border-gray-900 bg-white p-6 shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
                    <h2 className="mb-4 bg-gray-900 p-2 text-xl font-bold text-white uppercase">2. Top Sai Lệch</h2>
                    <ul className="space-y-3">
                        {MOCK_TRUTH.risks.map(risk => (
                            <li key={risk.id} className="flex items-center justify-between border-b border-gray-200 pb-1">
                                <span className="font-bold">{risk.item}: {risk.diff}</span>
                                <span className={`px-2 py-1 text-xs font-black text-white ${risk.level === 'ĐỎ' ? 'bg-red-600' : 'bg-yellow-500'}`}>
                                    {risk.level}
                                </span>
                            </li>
                        ))}
                    </ul>
                </section>

                {/* KHỐI 3: AN TOÀN THANH TOÁN */}
                <section className="border-4 border-gray-900 bg-white p-6 shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
                    <h2 className="mb-4 bg-gray-900 p-2 text-xl font-bold text-white uppercase">3. Thanh Toán & Lãi</h2>
                    <div className="space-y-4">
                        <div className="w-full bg-gray-200 h-6 border-2 border-gray-900">
                            <div className="bg-green-500 h-full border-r-2 border-gray-900" style={{ width: '65%' }}></div>
                        </div>
                        <p className="text-xs text-center font-bold uppercase">Đã thu: {MOCK_TRUTH.financials.paid} / {MOCK_TRUTH.financials.contract}</p>
                        <div className="flex justify-between items-center bg-gray-50 p-3 border-2 border-gray-900">
                            <span className="font-bold">LÃI DỰ KIẾN HIỆN TẠI:</span>
                            <span className="text-2xl font-black text-red-600">{MOCK_TRUTH.financials.profit}</span>
                        </div>
                    </div>
                </section>

                {/* KHỐI 4: HÀNH ĐỘNG CẦN LÀM */}
                <section className="border-4 border-gray-900 bg-white p-6 shadow-[8px_8px_0px_0px_rgba(0,0,0,1)]">
                    <h2 className="mb-4 bg-gray-900 p-2 text-xl font-bold text-white uppercase">4. Việc Phải Làm Ngay</h2>
                    <ul className="list-disc list-inside space-y-2">
                        {MOCK_TRUTH.actions.map((action, idx) => (
                            <li key={idx} className="text-sm font-bold border-l-4 border-gray-900 pl-2 hover:bg-gray-100 cursor-pointer">
                                {action}
                            </li>
                        ))}
                    </ul>
                    <button className="mt-6 w-full bg-gray-900 text-white p-3 font-black uppercase hover:bg-gray-700 transition-colors">
                        XUẤT BÁO CÁO GIẢI TRÌNH (WORD/PDF)
                    </button>
                </section>

            </div>

            {/* FOOTER: SYSTEM STATUS */}
            <footer className="mt-12 text-xs flex justify-between text-gray-500 border-t border-gray-300 pt-4">
                <span>IRON CORE: V2.5 (POLARS 0.52)</span>
                <span>ENGINE: CALAMINE 0.32</span>
                <span>BUILD: 2025.12.26.1200</span>
            </footer>
        </div>
    );
};

export default DashboardMockup;
