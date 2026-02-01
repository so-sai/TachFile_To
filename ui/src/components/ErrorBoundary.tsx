import React, { Component, ErrorInfo } from 'react';

interface Props {
    children: React.ReactNode;
    panelName: string;
}

interface State {
    hasError: boolean;
    error: Error | null;
    errorInfo: ErrorInfo | null;
}

/**
 * 🛡️ FORENSIC ERROR BOUNDARY
 * Strictly isolates panel failures to prevent Application-Wide WSOD.
 * Displays raw technical data for developer/forensic analysis.
 */
class ErrorBoundary extends Component<Props, State> {
    state: State = { hasError: false, error: null, errorInfo: null };

    static getDerivedStateFromError(error: Error): Partial<State> {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        this.setState({ errorInfo });
        // TODO: In the future, send this to a local log file via Rust FFI
        console.error(`[PANEL_TERMINATION] ${this.props.panelName}:`, error, errorInfo);
    }

    render() {
        if (this.state.hasError) {
            return (
                <div className="flex-1 bg-[#200000] text-red-500 p-6 font-mono text-[10px] border-4 border-red-600 select-all no-round relative overflow-hidden">
                    {/* BRUTALIST GRID BACKGROUND EFFECT */}
                    <div className="absolute inset-0 opacity-10 pointer-events-none" style={{ backgroundImage: 'radial-gradient(#ff0000 0.5px, transparent 0.5px)', backgroundSize: '10px 10px' }}></div>

                    <div className="relative z-10">
                        <div className="flex items-center gap-3 mb-4 bg-red-600 text-white px-2 py-1 w-fit">
                            <span className="font-black animate-pulse">PANEL_STATE: TERMINATED</span>
                        </div>

                        <div className="mb-6">
                            <div className="font-black uppercase tracking-[0.2em] text-red-400 mb-1">
                                COMPONENT_PATH: {this.props.panelName}
                            </div>
                            <div className="text-[9px] text-red-700 font-bold">
                                UUID: {crypto.randomUUID().split('-')[0].toUpperCase()} | SEVERITY: FORENSIC_LEVEL_4
                            </div>
                        </div>

                        <div className="bg-black/80 p-4 border-2 border-red-600/30 mb-6 shadow-[8px_8px_0px_rgba(255,0,0,0.1)]">
                            <div className="text-red-600 font-black mb-2 flex justify-between items-center border-b border-red-900 pb-1 uppercase tracking-tighter text-[9px]">
                                <span>{"{>>>}"} EXCEPTION_LOG_DUMP</span>
                                <span>STRATIFIED_ISOLATION: ACTIVE</span>
                            </div>
                            <div className="whitespace-pre-wrap overflow-auto custom-scrollbar max-h-[200px] text-[9px] leading-tight opacity-80">
                                {this.state.error?.stack || this.state.error?.message}
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4 border-t border-red-900 pt-4">
                            <div className="flex flex-col gap-1">
                                <span className="text-red-900 font-black uppercase text-[8px] tracking-[0.3em]">RECOVERY_VECTOR:</span>
                                <button
                                    onClick={() => window.location.reload()}
                                    className="bg-red-600 text-black hover:bg-white hover:text-black p-2 font-black uppercase text-[10px] no-round transition-none"
                                >
                                    SYSTEM_RE-INIT
                                </button>
                            </div>
                            <div className="flex flex-col gap-1 text-right">
                                <span className="text-red-900 font-black uppercase text-[8px] tracking-[0.3em]">LEGAL_STATUS:</span>
                                <div className="text-red-400 font-black text-[10px]">UNSTABLE_TRUTH</div>
                            </div>
                        </div>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

export default ErrorBoundary;
