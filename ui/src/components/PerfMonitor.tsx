import { useEffect, useState } from 'react';

/**
 * ⚡ PERFORMANCE MONITOR (DEV ONLY)
 * Tracks FPS and JS Heap usage to ensure Mission 028 targets.
 */
export function PerfMonitor() {
    const [fps, setFps] = useState(0);
    const [memory, setMemory] = useState(0);

    useEffect(() => {
        let frameCount = 0;
        let lastTime = performance.now();

        const loop = () => {
            frameCount++;
            const now = performance.now();
            if (now - lastTime >= 1000) {
                setFps(frameCount);
                frameCount = 0;
                lastTime = now;

                // @ts-ignore - Chrome/Tauri specific
                if (performance.memory) {
                    // @ts-ignore
                    setMemory(Math.round(performance.memory.usedJSHeapSize / 1024 / 1024));
                }
            }
            requestAnimationFrame(loop);
        };

        const id = requestAnimationFrame(loop);
        return () => cancelAnimationFrame(id);
    }, []);

    // Only show in development
    if (import.meta.env.PROD) return null;

    return (
        <div className="fixed bottom-[20px] right-4 bg-black/90 text-[#00FF00] px-3 py-1 font-mono text-[9px] z-[9999] border border-[#00FF00]/50 no-round pointer-events-none select-none">
            <span className={fps < 50 ? 'text-red-500' : ''}>FPS: {fps}</span>
            <span className="mx-2 opacity-30">|</span>
            <span className={memory > 100 ? 'text-yellow-500' : ''}>MEM: {memory}MB</span>
            <span className="mx-2 opacity-30">|</span>
            <span>028_HARDENED</span>
        </div>
    );
}

export default PerfMonitor;
