import React, { useEffect } from 'react';
import { useTruthStore, EncodingCandidate } from '../lib/useTruthStore';

const EvidencePane: React.FC = () => {
    const {
        selectedCellId,
        evidenceCache,
        isEvidenceLoading,
        cells,
        encodingCandidates,
        fetchEncodingCandidates,
        applyEncodingRepair
    } = useTruthStore();

    const evidence = selectedCellId ? evidenceCache[selectedCellId] : null;
    const selectedCard = cells.find(c => c.cell_id === selectedCellId);
    const isEncodingError = selectedCard?.verdict === 'Inadmissible' && selectedCard?.reason === 'EncodingCorruption';

    useEffect(() => {
        if (isEncodingError && selectedCard?.source_text) {
            fetchEncodingCandidates(selectedCard.source_text);
        }
    }, [selectedCellId, isEncodingError, selectedCard?.source_text, fetchEncodingCandidates]);

    const handleSelectCandidate = (candidate: EncodingCandidate) => {
        if (selectedCard) {
            applyEncodingRepair(
                selectedCard.row_idx,
                selectedCard.col_idx,
                selectedCard.source_text,
                candidate
            );
        }
    };

    if (!selectedCellId) {
        return (
            <div className="flex-1 flex flex-col items-center justify-center bg-slate-50 select-none text-center p-8">
                <div className="font-bold text-slate-300 text-sm uppercase tracking-wider">
                    Chọn một ô dữ liệu để xem bằng chứng gốc
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 flex flex-col bg-white overflow-hidden">

            {/* CONTENT */}
            <div className="flex-1 relative flex flex-col overflow-auto custom-scrollbar bg-slate-100/50">
                {isEvidenceLoading && !evidence && (
                    <div className="absolute inset-0 z-10 flex items-center justify-center bg-white/80 backdrop-blur-sm font-bold text-xs uppercase tracking-wider text-slate-500">
                        Đang trích xuất hình ảnh...
                    </div>
                )}

                {evidence ? (
                    <div className="flex flex-col w-full min-h-full">
                        {/* THE CROP */}
                        <div className="relative bg-white border-b border-slate-200 p-4 flex justify-center overflow-auto shadow-sm">
                            <img
                                src={`data:image/png;base64,${evidence.image_base64}`}
                                alt="Evidence Crop"
                                className="block max-w-full shadow-lg border border-slate-200"
                                style={{ imageRendering: 'pixelated' }}
                            />
                        </div>

                        {/* METADATA DESCRIPTION */}
                        <div className="w-full bg-white p-4 font-mono text-[10px] text-slate-500 leading-relaxed border-b border-slate-100">
                            <div className="font-bold text-slate-700 mb-2 uppercase tracking-wide flex justify-between">
                                <span>THÔNG TIN TRÍCH XUẤT:</span>
                            </div>
                            <div className="whitespace-pre-wrap">{evidence.metadata}</div>
                        </div>

                        {/* FORENSIC REPAIR OPTIONS */}
                        {isEncodingError && (
                            <div className="w-full bg-amber-50 border-t border-amber-200 p-4 flex flex-col gap-4">
                                <div className="text-amber-800 font-bold text-xs uppercase tracking-wider flex items-center gap-2">
                                    <span>⚠️ PHÁT HIỆN LỖI FONT (MOJIBAKE)</span>
                                </div>

                                <div className="grid grid-cols-1 gap-3">
                                    {/* PRISTINE (RAW) */}
                                    <div className="bg-white border border-amber-200 rounded p-3 flex flex-col gap-1">
                                        <div className="flex justify-between items-center text-[9px] font-bold text-slate-400 uppercase">
                                            <span>DỮ LIỆU GỐC (RAW)</span>
                                        </div>
                                        <div className="text-slate-800 font-mono text-xs break-all">
                                            {selectedCard?.source_text}
                                        </div>
                                    </div>

                                    {/* CANDIDATES */}
                                    <h5 className="text-[10px] font-bold text-slate-500 uppercase mt-2">ĐỀ XUẤT SỬA LỖI:</h5>
                                    {encodingCandidates.map((candidate, idx) => (
                                        <div key={idx} className="bg-white border border-blue-200 rounded p-3 flex flex-col gap-2 hover:border-blue-400 hover:shadow-md transition-all group">
                                            <div className="flex justify-between items-center text-[9px] font-bold text-blue-600 uppercase">
                                                <span>PHƯƠNG ÁN {idx + 1} ({candidate.mode})</span>
                                            </div>
                                            <div className="text-blue-900 font-bold text-sm">
                                                {candidate.text}
                                            </div>
                                            <button
                                                onClick={() => handleSelectCandidate(candidate)}
                                                className="w-full bg-blue-600 text-white font-bold text-xs uppercase py-2 rounded shadow-sm hover:bg-blue-700 active:scale-95 transition-all text-center"
                                            >
                                                CHỌN PHƯƠNG ÁN NÀY
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                ) : (
                    !isEvidenceLoading && (
                        <div className="flex flex-col items-center justify-center h-full text-slate-400 font-bold text-xs uppercase tracking-wider">
                            Không tìm thấy dữ liệu gốc
                        </div>
                    )
                )}
            </div>
        </div>
    );
};

export default EvidencePane;
