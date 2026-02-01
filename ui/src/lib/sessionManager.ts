/**
 * 💾 SESSION PERSISTENCE ENGINE
 * Adheres to the 'Zero-State Loss' objective for Mission 028.
 */

const SESSION_KEY = 'tachfileto_forensic_v1';

export interface PersistentSession {
    activeFile: string | null;
    selectedCellId: string | null;
    lastUpdated: string;
}

export const sessionManager = {
    save: (data: Partial<PersistentSession>) => {
        try {
            const existing = sessionManager.load() || {
                activeFile: null,
                selectedCellId: null,
                lastUpdated: new Date().toISOString()
            };

            const updated = {
                ...existing,
                ...data,
                lastUpdated: new Date().toISOString()
            };

            localStorage.setItem(SESSION_KEY, JSON.stringify(updated));
        } catch (e) {
            console.warn('[SESSION_SYNC_FAIL] Storage quota exceeded or disabled', e);
        }
    },

    load: (): PersistentSession | null => {
        try {
            const raw = localStorage.getItem(SESSION_KEY);
            if (!raw) return null;
            return JSON.parse(raw) as PersistentSession;
        } catch {
            return null;
        }
    },

    clear: () => {
        localStorage.removeItem(SESSION_KEY);
    }
};
