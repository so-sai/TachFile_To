import React from 'react';
import { useTruthStore } from '../lib/useTruthStore';

const EvidencePane: React.FC = () => {
    const { selectedCellId, evidenceCache, isEvidenceLoading } = useTruthStore();

    const evidence = selectedCellId ? evidenceCache[selectedCellId] : null;

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
