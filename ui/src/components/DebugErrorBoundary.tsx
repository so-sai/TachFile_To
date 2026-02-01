import { Component, ErrorInfo, ReactNode } from "react";

interface Props {
    children: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
    errorInfo: ErrorInfo | null;
}

class DebugErrorBoundary extends Component<Props, State> {
    public state: State = {
        hasError: false,
        error: null,
        errorInfo: null,
    };

    public static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error, errorInfo: null };
    }

    public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error("Uncaught error:", error, errorInfo);
        this.setState({ errorInfo });
    }

    public render() {
        if (this.state.hasError) {
            return (
                <div className="fixed inset-0 z-50 flex flex-col items-center justify-center bg-red-50 p-6 font-sans text-gray-900">
                    <div className="w-full max-w-2xl rounded-lg border border-red-200 bg-white p-8 shadow-xl">
                        <h1 className="mb-4 text-2xl font-bold text-red-600">
                            🛡️ TECHNICAL FAILURE (AEC-CRASH-01)
                        </h1>
                        <p className="mb-6 text-gray-600">
                            Hệ thống đã dừng lại để bảo vệ dữ liệu.
                        </p>

                        <div className="mb-6 overflow-hidden rounded bg-gray-900 p-4 text-left">
                            <p className="font-mono text-sm text-red-400">
                                {this.state.error?.toString()}
                            </p>
                        </div>

                        <details className="mb-6">
                            <summary className="cursor-pointer text-sm font-medium text-gray-500 hover:text-gray-700">
                                View Stack Trace
                            </summary>
                            <div className="mt-2 max-h-64 overflow-auto rounded bg-gray-100 p-4">
                                <pre className="font-mono text-xs text-gray-700">
                                    {this.state.errorInfo?.componentStack}
                                </pre>
                            </div>
                        </details>

                        <div className="flex gap-4">
                            <button
                                onClick={() => window.location.reload()}
                                className="rounded bg-red-600 px-4 py-2 text-sm font-semibold text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
                            >
                                Reload System
                            </button>
                            <button
                                onClick={() => {
                                    const dump = {
                                        timestamp: new Date().toISOString(),
                                        error: this.state.error?.toString(),
                                        stack: this.state.errorInfo?.componentStack,
                                        ua: navigator.userAgent
                                    };
                                    const blob = new Blob([JSON.stringify(dump, null, 2)], { type: "application/json" });
                                    const url = URL.createObjectURL(blob);
                                    const a = document.createElement("a");
                                    a.href = url;
                                    a.download = `AEC-CRASH-DUMP-${Date.now()}.json`;
                                    a.click();
                                }}
                                className="rounded border border-gray-300 bg-white px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
                            >
                                🛰️ Export System Dump
                            </button>
                        </div>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}

export default DebugErrorBoundary;
