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
            <div className="flex-1 flex flex-col items-center justify-center bg-black text-gray-800 select-none no-round">
                <div className="font-black text-3xl uppercase tracking-tighter opacity-10 italic">
                    CHẬU GIÁM ĐỊNH TRỐNG
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 flex flex-col bg-[#050505] overflow-hidden no-round border-t-4 border-[#333]">
            <div className="bg-black text-white p-2 font-black text-[10px] uppercase tracking-widest flex justify-between items-center select-none border-b-2 border-[#333] no-round">
                <span>3. EVIDENCE PANE [BẰNG CHỨNG THỊ GIÁC]</span>
                <span className="text-[10px] text-green-500 font-mono italic">{selectedCellId}</span>
            </div>

            <div className="flex-1 relative flex flex-col overflow-auto custom-scrollbar p-0 items-center bg-[#0a0a0a]">
                {isEvidenceLoading && !evidence && (
                    <div className="absolute inset-0 z-10 flex items-center justify-center bg-black font-black text-[10px] uppercase tracking-widest text-[#555]">
                        TRUY XUẤT VẬT CHỨNG GỐC...
                    </div>
                )}

                {evidence ? (
                    <div className="flex flex-col items-center w-full min-h-full gap-0">
                        {/* THE CROP - SCALE 2.0X AS PER CORE BRIDGE */}
                        <div className="relative border-b-4 border-[#222] bg-white p-0 w-full flex justify-center overflow-auto custom-scrollbar">
                            <img
                                src={`data:image/png;base64,${evidence.image_base64}`}
                                alt="Evidence Crop"
                                className="block"
                                style={{ imageRendering: 'pixelated' }}
                            />
                        </div>

                        {/* METADATA DESCRIPTION */}
                        <div className="w-full bg-[#000] p-4 font-mono text-[10px] text-[#666] uppercase tracking-wider leading-relaxed">
                            <div className="text-yellow-600 font-black mb-2 select-none flex justify-between">
                                <span>SOURCE_METADATA_STREAM:</span>
                                <span>PRISTINE_AUTH_CERTIFICATE</span>
                            </div>
                            <div className="whitespace-pre-wrap">{evidence.metadata}</div>
                        </div>

                        {/* MISSION 029: FORENSIC REPAIR OPTIONS */}
                        {isEncodingError && (
                            <div className="w-full bg-[#080808] border-t-2 border-[#222] p-4 flex flex-col gap-4">
                                <div className="text-red-500 font-black text-[10px] uppercase tracking-widest select-none">
                                    [4. FORENSIC REPAIR OPTIONS - GIẢI MÃ THỊ GIÁC]
                                </div>

                                <div className="grid grid-cols-1 gap-2">
                                    {/* PRISTINE (RAW) */}
                                    <div className="bg-black border border-[#222] p-2 flex flex-col gap-1">
                                        <div className="flex justify-between items-center text-[8px] font-black text-[#444] uppercase tracking-tighter">
                                            <span>RAW_BYTE_STREAM</span>
                                            <span className="text-[#333]">UTF-8 (PRISTINE)</span>
                                        </div>
                                        <div className="text-white font-mono text-xs p-1 bg-[#111] break-all">
                                            {selectedCard?.source_text}
                                        </div>
                                    </div>

                                    {/* CANDIDATES */}
                                    {encodingCandidates.map((candidate, idx) => (
                                        <div key={idx} className="bg-black border border-blue-900/30 p-2 flex flex-col gap-2 hover:border-blue-500 transition-colors group">
                                            <div className="flex justify-between items-center text-[8px] font-black text-blue-500 uppercase tracking-tighter">
                                                <span>INTERPRETATION_LANE_{idx + 1}</span>
                                                <span>{candidate.mode}_DECODER</span>
                                            </div>
                                            <div className="text-blue-100 font-bold text-sm p-1">
                                                {candidate.text}
                                            </div>
                                            <button
                                                onClick={() => handleSelectCandidate(candidate)}
                                                className="w-full bg-blue-900 text-white font-black text-[9px] uppercase py-1 tracking-widest hover:bg-blue-600 active:scale-95 transition-all no-round"
                                            >
                                                SELECT & SEAL AS TRUTH
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}
                    </div>
                ) : (
                    !isEvidenceLoading && (
                        <div className="flex flex-col items-center justify-center h-full text-red-900 font-black uppercase text-[10px] tracking-widest italic">
                            [!] KHÔNG TÌM THẤY BÀN GỐC TÀI LIỆU
                        </div>
                    )
                )}
            </div>

            <div className="p-1 px-4 bg-black border-t border-[#222] text-[8px] text-[#333] font-black uppercase tracking-[0.3em] flex justify-between select-none">
                <span>72 DPI HIGH-CONTRAST RENDER</span>
                <span>INTERNAL_PDF_ENGINE_V2_5</span>
            </div>
        </div>
    );
};

export default EvidencePane;
