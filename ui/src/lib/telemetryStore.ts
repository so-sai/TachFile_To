import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';

interface TelemetryPayload {
    cpuUsage: number;
    ramUsageMb: number;
    ffiLockActive: boolean;
    isLimpMode: boolean;
    janitorStatus: string;
}

interface TelemetryState {
    cpu: number;
    ram: number;
    isLimpMode: boolean;
    isFFILocked: boolean;
    janitorStatus: string;
    isLive: boolean;
    init: () => Promise<() => void>;
}

export const useTelemetryStore = create<TelemetryState>((set) => ({
    cpu: 0,
    ram: 0,
    isLimpMode: false,
    isFFILocked: false,
    janitorStatus: 'IDLE',
    isLive: false,
    init: async () => {
        console.log("[Telemetry] Initializing heartbeat listener...");
        const unlisten = await listen<TelemetryPayload>('system:heartbeat', (event) => {
            const p = event.payload;
            set({
                cpu: p.cpuUsage,
                ram: p.ramUsageMb,
                isLimpMode: p.isLimpMode,
                isFFILocked: p.ffiLockActive,
                janitorStatus: p.janitorStatus,
                isLive: true
            });
        });
        return unlisten;
    }
}));
